use inkwell::{
    context::Context,
    execution_engine::{ExecutionEngine, JitFunction},
    module::Module,
    OptimizationLevel,
};
use std::fmt::{self, Debug, Formatter};
use std::mem;

pub struct StaticData {
    data: Vec<u8>,
}

// Macro for creating self-referential structs.
rental! {
    mod rented_impl {
        use super::*;

        // Disgusting, it's covered in references to itself.
        #[rental]
        pub struct IncestuousData {
            context: Box<Context>,
            module: Box<Module<'context>>,
            execution_engine: Box<ExecutionEngine<'context>>,
            function: Box<JitFunction<
                'context, unsafe extern "C" fn(*mut u8, *mut u8, *mut u8) -> u32>>,
            static_init: Box<JitFunction<
                'context, unsafe extern "C" fn(*mut u8) -> u32>>,
        }
    }
}

use rented_impl::IncestuousData;

pub struct Program {
    idata: IncestuousData,
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
        let content = self.idata.rent_all(|data| data.module.print_to_string());
        write!(formatter, "LLVM IR Code:{}", content)?;
        write!(formatter, "")
    }
}

impl Program {
    pub fn new(
        context: Context,
        module_creator: impl for<'ctx> FnOnce(&'ctx Context) -> Box<Module<'ctx>>,
        in_size: usize,
        out_size: usize,
        static_size: usize,
        error_descriptions: Vec<String>,
    ) -> Self {
        let idata = IncestuousData::new(
            Box::new(context),
            module_creator,
            |module, _| {
                Box::new(
                    module
                        .create_jit_execution_engine(OptimizationLevel::Default)
                        .unwrap(),
                )
            },
            |execution_engine, _, _| {
                Box::new(unsafe { execution_engine.get_function("main").unwrap() })
            },
            |_, execution_engine, _, _| {
                Box::new(unsafe { execution_engine.get_function("static_init").unwrap() })
            },
        );
        Self {
            idata,
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
        let error_code = self
            .idata
            .rent_all(|idata| idata.static_init.call(data.data.as_mut_ptr()));
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
        let error_code = self
            .idata
            .rent_all(|idata| idata.static_init.call(data.data.as_mut_ptr()));
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
        let error_code = self.idata.rent_all(|idata| {
            idata.function.call(
                input_data as *mut T as *mut u8,
                static_data.data.as_mut_ptr(),
                output_data as *mut U as *mut u8,
            )
        });
        self.parse_error_code(error_code)
    }

    pub unsafe fn execute_raw(
        &self,
        input_data: &mut [u8],
        output_data: &mut [u8],
        static_data: &mut StaticData,
    ) -> Result<(), &str> {
        self.assert_size(input_data.len(), output_data.len(), static_data.data.len());
        let error_code = self.idata.rent_all(|idata| {
            idata.function.call(
                input_data.as_mut_ptr(),
                static_data.data.as_mut_ptr(),
                output_data.as_mut_ptr(),
            )
        });
        self.parse_error_code(error_code)
    }
}
