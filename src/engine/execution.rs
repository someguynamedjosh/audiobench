use crate::engine::registry::Registry;

#[repr(C)]
pub struct InputData {
    global_pitch: f32,
    global_velocity: f32,
    global_note_time: [f32; 512],
}

#[repr(C)]
pub struct OutputData {
    global_audio_out: [f32; 1024],
}

pub struct ExecEnvironment {
    compiler: nodespeak::Compiler,
    program: Option<nodespeak::llvmir::structure::Program>,
    input: InputData,
    output: OutputData,
    static_data: Option<nodespeak::llvmir::structure::StaticData>,
}

impl ExecEnvironment {
    pub fn new(registry: &Registry) -> Self {
        let mut compiler = nodespeak::Compiler::new();
        // TODO: Add a mechanism to change sources.
        compiler.add_source("<node graph>".to_owned(), "".to_owned());
        for (name, content) in registry.borrow_scripts().iter() {
            compiler.add_source(name.to_owned(), content.to_owned());
        }

        let input = InputData {
            global_pitch: 440.0,
            global_velocity: 440.0,
            global_note_time: [0.0; 512],
        };
        let output = OutputData {
            global_audio_out: [0.0; 1024],
        };
        Self {
            compiler,
            program: None,
            input,
            output,
            static_data: None,
        }
    }

    pub fn compile(&mut self, source: String) -> Result<(), String> {
        self.compiler.add_source("<node graph>".to_owned(), source);
        self.program = Some(self.compiler.compile("<node graph>")?);
        unsafe {
            self.static_data = Some(self.program.as_ref().unwrap().create_static_data()?);
        }
        Ok(())
    }

    pub fn execute(&mut self) -> Result<&[f32], String> {
        if let Some(program) = &self.program {
            unsafe {
                program
                    .execute_data(
                        &mut self.input,
                        &mut self.output,
                        self.static_data.as_mut().unwrap(),
                    )
                    .map_err(|s| s.to_owned())?;
            }
            Ok(&self.output.global_audio_out)
        } else {
            Err("Program not compiled.".to_owned())
        }
    }
}
