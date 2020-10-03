use super::{DataPacker, DataUnpacker};
use inkwell::{
    context::Context,
    execution_engine::{ExecutionEngine, JitFunction},
    module::Module,
    OptimizationLevel,
};
use ouroboros::self_referencing;
use std::fmt::{self, Debug, Formatter};

pub struct StaticData {
    data: Vec<u8>,
}

// Disgusting, it's covered in references to itself.
#[self_referencing(chain_hack)]
struct IncestuousData {
    context: Box<Context>,
    #[borrows(context)]
    module: Box<Module<'this>>,
    #[borrows(module)]
    execution_engine: Box<ExecutionEngine<'this>>,
    #[borrows(execution_engine)]
    function: JitFunction<'this, unsafe extern "C" fn(*mut u8, *mut u8, *mut u8) -> u32>,
    #[borrows(execution_engine)]
    static_init: JitFunction<'this, unsafe extern "C" fn(*mut u8) -> u32>,
}

pub struct Program {
    idata: IncestuousData,
    in_packer: DataPacker,
    out_unpacker: DataUnpacker,
    static_size: usize,
    error_descriptions: Vec<String>,
}

impl Debug for Program {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        writeln!(formatter, "error codes:")?;
        for (code, description) in self.error_descriptions.iter().enumerate() {
            writeln!(formatter, "  {}: {}", code, description)?;
        }
        let content = self
            .idata
            .with_module_contents(|module| module.print_to_string());
        write!(formatter, "LLVM IR Code:\n{}", content.to_string())?;
        write!(formatter, "")
    }
}

impl Program {
    pub fn new(
        context: Context,
        module_builder: impl for<'this> FnOnce(&'this Context) -> Box<Module<'this>>,
        in_packer: DataPacker,
        out_unpacker: DataUnpacker,
        static_size: usize,
        error_descriptions: Vec<String>,
    ) -> Self {
        let idata = IncestuousDataBuilder {
            context: Box::new(context),
            module_builder: module_builder,
            execution_engine_builder: |module| {
                // For some reason no matter how much I crank it up it doesn't go faster. I'm not
                // sure if it is doing vectorization. It was the same level of performance before
                // I implemented rolled-up array-wise operators.
                Box::new(
                    module
                        .create_jit_execution_engine(OptimizationLevel::Less)
                        .unwrap(),
                )
            },
            function_builder: |execution_engine| unsafe {
                execution_engine.get_function("main").unwrap()
            },
            static_init_builder: |execution_engine| unsafe {
                execution_engine.get_function("static_init").unwrap()
            },
        }
        .build();
        Self {
            idata,
            in_packer,
            out_unpacker,
            static_size,
            error_descriptions,
        }
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

    pub fn borrow_input_packer_mut(&mut self) -> &mut DataPacker {
        &mut self.in_packer
    }

    pub fn borrow_output_unpacker(&self) -> &DataUnpacker {
        &self.out_unpacker
    }

    pub unsafe fn create_static_data(&self) -> Result<StaticData, &str> {
        let mut data = StaticData {
            data: vec![0; self.static_size],
        };
        let error_code = self
            .idata
            .with(|idata| idata.static_init.call(data.data.as_mut_ptr()));
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
            .with(|idata| idata.static_init.call(data.data.as_mut_ptr()));
        self.parse_error_code(error_code)
    }

    pub unsafe fn execute(
        &mut self,
        static_data: &mut StaticData,
    ) -> Result<(), &str> {
        let input_data = self.in_packer.borrow_packed_data();
        let output_data = self.out_unpacker.borrow_packed_data();
        let error_code = self.idata.with(|idata| {
            idata.function.call(
                input_data.as_mut_ptr(),
                static_data.data.as_mut_ptr(),
                output_data.as_mut_ptr(),
            )
        });
        self.parse_error_code(error_code)
    }
}
