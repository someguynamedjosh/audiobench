use crate::engine::registry::Registry;
use nodespeak::llvmir::structure::{Program, StaticData};

pub struct ExecEnvironment {
    compiler: nodespeak::Compiler,
    program: Option<Program>,
    input: Vec<f32>,
    // How much space the required inputs like pitch and time take up.
    default_inputs_len: usize,
    output: Vec<f32>,
    buffer_length: usize,
    sample_rate: i32,
}

pub struct AuxMidiData {
    // MIDI specifies each MIDI Channel has 128 controls.
    pub controller_values: [f32; 128],
    // The pitch wheel is seperate from other controls due to its higher precision.
    pub pitch_wheel_value: f32,
    pub bpm: f32,
    pub song_time: f32,
    pub song_beats: f32,
}

impl AuxMidiData {
    pub fn new() -> Self {
        Self {
            controller_values: [0.0; 128],
            pitch_wheel_value: 0.0,
            bpm: 120.0,
            song_time: 0.0,
            song_beats: 0.0,
        }
    }
}

impl ExecEnvironment {
    pub fn new(registry: &Registry) -> Self {
        let mut compiler = nodespeak::Compiler::new();
        // TODO: Add a mechanism to change sources.
        compiler.add_source("<node graph>".to_owned(), "".to_owned());
        compiler.add_source("!:lib.ns".to_owned(), include_str!("lib.ns").to_owned());
        for (name, content) in registry.borrow_scripts().iter() {
            compiler.add_source(name.to_owned(), content.to_owned());
        }

        Self {
            compiler,
            program: None,
            input: Vec::new(),
            default_inputs_len: 4 + 0,
            output: Vec::new(),
            buffer_length: 0,
            sample_rate: 1, // Prevent accidental div/0 errors.
        }
    }

    pub fn get_current_buffer_length(&self) -> usize {
        self.buffer_length
    }

    pub fn get_current_sample_rate(&self) -> i32 {
        self.sample_rate
    }

    pub fn set_pitch_input(&mut self, pitch: f32) {
        self.input[0] = pitch;
    }

    pub fn set_velocity_input(&mut self, velocity: f32) {
        self.input[1] = velocity;
    }

    pub fn set_note_status_input(&mut self, note_status: f32) {
        self.input[2] = note_status;
    }

    pub fn set_should_update_input(&mut self, should_update: f32) {
        self.input[3] = should_update;
    }

    fn set_bpm_input(&mut self, bpm: f32) {
        self.input[4] = bpm;
    }

    fn set_timing_input(&mut self, start_index: usize, base: f32, increment: f32) {
        let data = &mut self.input[start_index..self.buffer_length + start_index];
        let mut value = base;
        for index in 0..self.buffer_length {
            data[index] = value;
            value += increment;
        }
    }

    pub fn set_note_time_input(&mut self, base: f32) {
        self.set_timing_input(5, base, 1.0 / self.sample_rate as f32);
    }

    pub fn set_note_beats_input(&mut self, base: f32, aux_midi_data: &AuxMidiData) {
        self.set_timing_input(
            5 + self.buffer_length,
            base,
            aux_midi_data.bpm / 60.0 / self.sample_rate as f32,
        );
    }

    fn set_song_time_input(&mut self, base: f32) {
        self.set_timing_input(
            5 + 2 * self.buffer_length,
            base,
            1.0 / self.sample_rate as f32,
        );
    }

    fn set_song_beats_input(&mut self, base: f32, aux_midi_data: &AuxMidiData) {
        self.set_timing_input(
            5 + 3 * self.buffer_length,
            base,
            aux_midi_data.bpm / 60.0 / self.sample_rate as f32,
        );
    }

    pub fn set_aux_midi_data(&mut self, aux_midi_data: &AuxMidiData) {
        self.set_bpm_input(aux_midi_data.bpm);
        let midi_controls_input =
            &mut self.input[self.buffer_length * 4 + 5..self.buffer_length * 4 + 5 + 128];
        // MIDI standard specifies 128 controls
        for index in 0..128 {
            midi_controls_input[index] = aux_midi_data.controller_values[index];
        }
        self.set_song_time_input(aux_midi_data.song_time);
        self.set_song_beats_input(aux_midi_data.song_beats, aux_midi_data);
    }

    pub fn change_aux_input_data(&mut self, new_aux_input_data: &[f32]) {
        debug_assert!(new_aux_input_data.len() == self.input.len() - self.default_inputs_len);
        self.input[self.default_inputs_len..].clone_from_slice(new_aux_input_data);
    }

    pub fn compile(
        &mut self,
        source: String,
        mut starting_aux_data: Vec<f32>,
        buffer_length: usize,
        sample_rate: i32,
        feedback_data_len: usize,
    ) -> Result<(), String> {
        self.compiler.add_source("<node graph>".to_owned(), source);
        self.program = Some(self.compiler.compile("<node graph>")?);
        // global_pitch: FLOAT
        // global_velocity: FLOAT
        // global_note_status: FLOAT
        // global_bpm: FLOAT
        // global_note_time: [BL]FLOAT
        // global_note_beats: [BL]FLOAT
        // global_song_time: [BL]FLOAT
        // global_song_beats: [BL]FLOAT
        // global_midi_controls: [128]FLOAT
        // global_aux_data: [starting_aux_data.len()][1]FLOAT
        self.default_inputs_len = 5 + 4 * buffer_length + 128;
        self.input = vec![0.0; self.default_inputs_len];
        self.input.append(&mut starting_aux_data);
        // global_audio_out: [BL][2]FLOAT
        self.output = vec![0.0; 2 * buffer_length + feedback_data_len];
        self.buffer_length = buffer_length;
        self.sample_rate = sample_rate;
        Ok(())
    }

    pub fn create_static_data(&mut self) -> Result<StaticData, String> {
        let program = if let Some(program) = &self.program {
            program
        } else {
            return Err("Cannot create static data when program is not compiled.".to_owned());
        };
        Ok(unsafe { program.create_static_data()? })
    }

    pub fn execute(&mut self, static_data: &mut StaticData) -> Result<(), String> {
        let program = if let Some(program) = &self.program {
            program
        } else {
            return Err("Program executed before compiled.".to_owned());
        };
        unsafe {
            program
                .execute_raw(
                    vec_as_raw(&mut self.input),
                    vec_as_raw(&mut self.output),
                    static_data,
                )
                .map_err(|s| s.to_owned())?;
        }
        Ok(())
    }

    pub fn borrow_audio_out(&self) -> &[f32] {
        &self.output[..self.buffer_length * 2]
    }

    pub fn borrow_feedback_data(&self) -> &[f32] {
        &self.output[self.buffer_length * 2..]
    }
}

fn vec_as_raw<T: Sized>(input: &mut Vec<T>) -> &mut [u8] {
    unsafe {
        let out_len = input.len() * std::mem::size_of::<T>();
        std::slice::from_raw_parts_mut(input.as_mut_ptr() as *mut u8, out_len)
    }
}
