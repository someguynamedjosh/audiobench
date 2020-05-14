use crate::engine::registry::Registry;

pub struct ExecEnvironment {
    compiler: nodespeak::Compiler,
    program: Option<nodespeak::llvmir::structure::Program>,
    input: Vec<f32>,
    output: Vec<f32>,
    buffer_length: usize,
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

        Self {
            compiler,
            program: None,
            input: Vec::new(),
            output: Vec::new(),
            buffer_length: 0,
            static_data: None,
        }
    }

    pub fn compile(&mut self, source: String, buffer_length: usize) -> Result<(), String> {
        self.compiler.add_source("<node graph>".to_owned(), source);
        self.program = Some(self.compiler.compile("<node graph>")?);
        // global_pitch: FLOAT
        // global_velocity: FLOAT
        // global_note_time: [BL]FLOAT
        self.input = vec![0.0; 2 + buffer_length];
        // global_audio_out: [BL][2]FLOAT
        self.output = vec![0.0; 2 * buffer_length];
        self.buffer_length = buffer_length;
        unsafe {
            self.static_data = Some(self.program.as_ref().unwrap().create_static_data()?);
        }
        Ok(())
    }

    pub fn execute(&mut self) -> Result<&[f32], String> {
        // global_pitch
        self.input[0] = 440.0;
        // global_velocity
        self.input[1] = 1.0;
        if let Some(program) = &self.program {
            unsafe {
                program
                    .execute_raw(
                        vec_as_raw(&mut self.input),
                        vec_as_raw(&mut self.output),
                        self.static_data.as_mut().unwrap(),
                    )
                    .map_err(|s| s.to_owned())?;
            }
            Ok(&self.output[0..self.buffer_length * 2])
        } else {
            Err("Program not compiled.".to_owned())
        }
    }
}

fn vec_as_raw<T: Sized>(input: &mut Vec<T>) -> &mut [u8] {
    unsafe {
        let out_len = input.len() * std::mem::size_of::<T>();
        std::slice::from_raw_parts_mut(input.as_mut_ptr() as *mut u8, out_len)
    }
}
