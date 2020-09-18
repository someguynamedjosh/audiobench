use inkwell::{
    basic_block::BasicBlock,
    builder::Builder,
    context::Context,
    execution_engine::{ExecutionEngine, JitFunction},
    module::Module,
    targets::{InitializationConfig, Target},
    types::{BasicTypeEnum, PointerType, StructType},
    values::{
        BasicValue, BasicValueEnum, CallSiteValue, FloatValue, FunctionValue, IntValue,
        PointerValue,
    },
    AddressSpace, FloatPredicate, IntPredicate, OptimizationLevel,
};
use std::fmt::{self, Debug, Formatter};
use std::mem::{self, MaybeUninit};
use std::pin::Pin;
use std::ptr;

pub struct StaticData {
    data: Vec<u8>,
}

pub struct Program<'ctx> {
    execution_engine: ExecutionEngine<'ctx>,
    function: JitFunction<'ctx, unsafe extern "C" fn(*mut u8, *mut u8, *mut u8) -> u32>,
    static_init: JitFunction<'ctx, unsafe extern "C" fn(*mut u8) -> u32>,
    context: Box<Context>,
    module: Box<Module<'ctx>>,
    in_size: usize,
    out_size: usize,
    static_size: usize,
    error_descriptions: Vec<String>,
}

impl<'ctx> Debug for Program<'ctx> {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        writeln!(formatter, "error codes:")?;
        for (code, description) in self.error_descriptions.iter().enumerate() {
            writeln!(formatter, "  {}: {}", code, description)?;
        }
        unsafe {
            let content = self.module.print_to_string();
            write!(formatter, "LLVM IR Code:{}", content)?;
        }
        write!(formatter, "")
    }
}

impl<'ctx> Program<'ctx> {
    /// After this, the prrogram will handle dropping the module and context automatically.
    pub fn new(
        context: Box<Context>,
        module: Box<Module<'ctx>>,
        in_size: usize,
        out_size: usize,
        static_size: usize,
        error_descriptions: Vec<String>,
    ) -> Self {
        let execution_engine = module
            .create_jit_execution_engine(OptimizationLevel::Default)
            .unwrap();
        let function = unsafe { execution_engine.get_function("main").unwrap() };
        let static_init = unsafe { execution_engine.get_function("static_init").unwrap() };
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
        let error_code = self.static_init.call(data.data.as_mut_ptr());
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
        let error_code = self.static_init.call(data.data.as_mut_ptr());
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
        let error_code = self.function.call(
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
        let error_code = self.function.call(
            input_data.as_mut_ptr(),
            static_data.data.as_mut_ptr(),
            output_data.as_mut_ptr(),
        );
        self.parse_error_code(error_code)
    }
}
