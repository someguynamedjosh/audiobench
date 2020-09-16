use llvm_sys::core::*;
use llvm_sys::execution_engine::*;
use llvm_sys::prelude::*;
use llvm_sys::target::*;
use std::fmt::{self, Debug, Formatter};
use std::mem::{self, MaybeUninit};
use std::ptr;

pub struct StaticData {
    data: Vec<u8>,
}

pub struct Program {
    execution_engine: LLVMExecutionEngineRef,
    function: extern "C" fn(*mut u8, *mut u8, *mut u8) -> u32,
    static_init: extern "C" fn(*mut u8) -> u32,
    context: LLVMContextRef,
    module: LLVMModuleRef,
    in_size: usize,
    out_size: usize,
    static_size: usize,
    error_descriptions: Vec<String>,
}

impl Debug for Program {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        writeln!(formatter, "error codes:")?;
        for (code, description) in self.error_descriptions.iter().enumerate() {
            writeln!(formatter, "  {}: {}", code, description)?;
        }
        unsafe {
            let content = LLVMPrintModuleToString(self.module);
            write!(
                formatter,
                "LLVM IR Code:{}",
                std::ffi::CStr::from_ptr(content).to_string_lossy()
            )?;
            LLVMDisposeMessage(content);
        }
        write!(formatter, "")
    }
}

impl Drop for Program {
    fn drop(&mut self) {
        unsafe {
            // Disposing the execution engine also disposes the module.
            LLVMDisposeExecutionEngine(self.execution_engine);
            LLVMContextDispose(self.context);
        }
    }
}

impl Program {
    /// After this, the prrogram will handle dropping the module and context automatically.
    pub fn new(
        context: LLVMContextRef,
        module: LLVMModuleRef,
        in_type: LLVMTypeRef,
        out_type: LLVMTypeRef,
        static_type: LLVMTypeRef,
        error_descriptions: Vec<String>,
    ) -> Self {
        let execution_engine = unsafe {
            let mut ee_ref = MaybeUninit::uninit();
            let mut creation_error = ptr::null_mut();
            LLVMLinkInMCJIT();
            assert!(
                LLVM_InitializeNativeTarget() != 1,
                "Failed to initialize native target."
            );
            assert!(
                LLVM_InitializeNativeAsmPrinter() != 1,
                "Failed to initialize native asm."
            );
            // This takes ownership of the module so disposing the EE disposes the module.
            LLVMCreateExecutionEngineForModule(ee_ref.as_mut_ptr(), module, &mut creation_error);

            ee_ref.assume_init()
        };
        let function = unsafe {
            let func_addr =
                LLVMGetFunctionAddress(execution_engine, b"main\0".as_ptr() as *const _);
            mem::transmute(func_addr)
        };
        let static_init = unsafe {
            let func_addr =
                LLVMGetFunctionAddress(execution_engine, b"static_init\0".as_ptr() as *const _);
            mem::transmute(func_addr)
        };
        let (in_size, out_size, static_size) = unsafe {
            let target_data = LLVMGetExecutionEngineTargetData(execution_engine);
            let in_size = LLVMSizeOfTypeInBits(target_data, in_type) / 8;
            let out_size = LLVMSizeOfTypeInBits(target_data, out_type) / 8;
            let static_size = LLVMSizeOfTypeInBits(target_data, static_type) / 8;
            (in_size as usize, out_size as usize, static_size as usize)
        };
        Self {
            execution_engine,
            function,
            static_init,
            context,
            module,
            in_size,
            out_size,
            static_size,
            error_descriptions,
        }
    }

    fn assert_size(&self, in_size: usize, out_size: usize, static_size: usize) {
        assert!(
            self.in_size == in_size,
            "Expected {}, got {}.",
            self.in_size,
            in_size
        );
        assert!(
            self.out_size == out_size,
            "Expected {}, got {}.",
            self.out_size,
            out_size
        );
        assert!(
            self.static_size == static_size,
            "Expected {}, got {}.",
            self.static_size,
            static_size
        );
    }

    fn parse_error_code(&self, error_code: u32) -> Result<(), &str> {
        if error_code == 0 {
            Ok(())
        } else if (error_code as usize) < self.error_descriptions.len() && error_code > 0 {
            Err(&self.error_descriptions[error_code as usize])
        } else {
            Err("Invalid non-success error code")
        }
    }

    pub unsafe fn create_static_data(&self) -> Result<StaticData, &str> {
        let mut data = StaticData {
            data: vec![0; self.static_size],
        };
        let error_code = (self.static_init)(data.data.as_mut_ptr());
        self.parse_error_code(error_code)?;
        Ok(data)
    }

    pub unsafe fn reinit_static_data(&self, data: &mut StaticData) -> Result<(), &str> {
        assert!(
            self.static_size == data.data.len(),
            "Expected {}, got {}.",
            self.static_size,
            data.data.len()
        );
        let error_code = (self.static_init)(data.data.as_mut_ptr());
        self.parse_error_code(error_code)
    }

    pub unsafe fn execute_data<T: Sized, U: Sized>(
        &self,
        input_data: &mut T,
        output_data: &mut U,
        static_data: &mut StaticData,
    ) -> Result<(), &str> {
        self.assert_size(
            mem::size_of::<T>(),
            mem::size_of::<U>(),
            static_data.data.len(),
        );
        let error_code = (self.function)(
            input_data as *mut T as *mut u8,
            static_data.data.as_mut_ptr(),
            output_data as *mut U as *mut u8,
        );
        self.parse_error_code(error_code)
    }

    pub unsafe fn execute_raw(
        &self,
        input_data: &mut [u8],
        output_data: &mut [u8],
        static_data: &mut StaticData,
    ) -> Result<(), &str> {
        self.assert_size(input_data.len(), output_data.len(), static_data.data.len());
        let error_code = (self.function)(
            input_data.as_mut_ptr(),
            static_data.data.as_mut_ptr(),
            output_data.as_mut_ptr(),
        );
        self.parse_error_code(error_code)
    }
}
