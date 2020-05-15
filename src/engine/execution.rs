use crate::engine::registry::Registry;
use crate::util::*;
use array_macro::array;

struct ActiveVoice {
    pitch: f32,
    velocity: f32,
    elapsed_samples: usize,
    silent_samples: usize,
    release_trigger: bool,
    static_data: nodespeak::llvmir::structure::StaticData,
}

const NUM_MIDI_NOTES: usize = 128;
// Notes must be silent for at least this amount of time before they will be shut off.
const MIN_SILENT_TIME: f32 = 0.1;
// Notes must have every sample be of this magnitude or less to be considered silent.
const SILENT_CUTOFF: f32 = 1e-5;

pub struct ExecEnvironment {
    compiler: nodespeak::Compiler,
    program: Option<nodespeak::llvmir::structure::Program>,
    input: Vec<f32>,
    output: Vec<f32>,
    accumulator: Vec<f32>,
    buffer_length: usize,
    time_per_sample: f32,
    held_notes: [Option<ActiveVoice>; NUM_MIDI_NOTES],
    decaying_notes: Vec<ActiveVoice>,
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
            accumulator: Vec::new(),
            buffer_length: 0,
            time_per_sample: 0.0,
            held_notes: array![None; NUM_MIDI_NOTES],
            decaying_notes: Vec::new(),
        }
    }

    pub fn compile(
        &mut self,
        source: String,
        buffer_length: usize,
        sample_rate: i32,
    ) -> Result<(), String> {
        self.compiler.add_source("<node graph>".to_owned(), source);
        self.program = Some(self.compiler.compile("<node graph>")?);
        // global_pitch: FLOAT
        // global_velocity: FLOAT
        // global_note_status: FLOAT
        // global_note_time: [BL]FLOAT
        self.input = vec![0.0; 3 + buffer_length];
        // global_audio_out: [BL][2]FLOAT
        self.output = vec![0.0; 2 * buffer_length];
        // Just an audio buffer, nothing special.
        self.accumulator = vec![0.0; 2 * buffer_length];
        self.buffer_length = buffer_length;
        self.time_per_sample = 1.0 / sample_rate as f32;

        let pref = self.program.as_ref().unwrap();
        for held_note in self.held_notes.iter_mut() {
            if let Some(voice) = held_note {
                voice.static_data = unsafe {
                    pref.create_static_data()?
                };
            }
        }
        self.decaying_notes.clear();
        Ok(())
    }

    fn execute_impl(
        program: &nodespeak::llvmir::structure::Program,
        accumulator: &mut Vec<f32>,
        voice: &mut ActiveVoice,
        input: &mut Vec<f32>,
        output: &mut Vec<f32>,
        buffer_length: usize,
        time_per_sample: f32,
    ) -> Result<(), String> {
        // global_pitch
        input[0] = voice.pitch;
        // global_velocity
        input[1] = voice.velocity.to_range(-1.0, 1.0);
        // global_note_status
        input[2] = if voice.release_trigger { 1.0 } else { 0.0 };
        voice.release_trigger = false;
        // global_note_time
        for i in 0..buffer_length {
            input[i + 3] = (i + voice.elapsed_samples) as f32 * time_per_sample;
        }
        unsafe {
            program
                .execute_raw(
                    vec_as_raw(input),
                    vec_as_raw(output),
                    &mut voice.static_data,
                )
                .map_err(|s| s.to_owned())?;
        }
        let mut all_silent = true;
        for i in 0..buffer_length * 2 {
            accumulator[i] += output[i];
            if output[i].abs() > SILENT_CUTOFF {
                all_silent = false;
            }
        }
        voice.elapsed_samples += buffer_length;
        if all_silent {
            voice.silent_samples += buffer_length;
        } else {
            voice.silent_samples = 0;
        }
        Ok(())
    }

    pub fn execute(&mut self) -> Result<&[f32], String> {
        for i in 0..self.buffer_length * 2 {
            self.accumulator[i] = 0.0;
        }
        let program = if let Some(program) = &self.program {
            program
        } else {
            return Err("Program executed before compiled!".to_owned());
        };
        for note in self.held_notes.iter_mut() {
            if let Some(note) = note {
                Self::execute_impl(
                    &program,
                    &mut self.accumulator,
                    note,
                    &mut self.input,
                    &mut self.output,
                    self.buffer_length,
                    self.time_per_sample,
                )?;
            }
        }
        for note in self.decaying_notes.iter_mut() {
            Self::execute_impl(
                &program,
                &mut self.accumulator,
                note,
                &mut self.input,
                &mut self.output,
                self.buffer_length,
                self.time_per_sample,
            )?;
        }
        let min_silent_samples = (MIN_SILENT_TIME / self.time_per_sample) as usize;
        // Iterate backwards because we are deleting things.
        for note_index in (0..self.decaying_notes.len()).rev() {
            if self.decaying_notes[note_index].silent_samples >= min_silent_samples {
                self.decaying_notes.remove(note_index);
            }
        }
        Ok(&self.accumulator[..])
    }

    pub fn note_on(&mut self, note_index: i32, velocity: f32) {
        assert!(note_index < 128);
        debug_assert!(self.program.is_some());
        self.held_notes[note_index as usize] = Some(ActiveVoice {
            pitch: equal_tempered_tuning(note_index),
            velocity,
            elapsed_samples: 0,
            silent_samples: 0,
            release_trigger: false,
            static_data: unsafe {
                self.program
                    .as_ref()
                    .unwrap()
                    .create_static_data()
                    .expect("TODO: Nice error")
            },
        })
    }

    pub fn note_off(&mut self, note_index: i32) {
        assert!(note_index < 128);
        if let Some(mut note) = self.held_notes[note_index as usize].take() {
            note.release_trigger = true;
            self.decaying_notes.push(note);
        }
    }
}

fn equal_tempered_tuning(index: i32) -> f32 {
    // MIDI note 69 is 440Hz. 12 notes is an octave (double / half frequency).
    440.0 * (2.0f32).powf((index - 69) as f32 / 12.0)
}

fn vec_as_raw<T: Sized>(input: &mut Vec<T>) -> &mut [u8] {
    unsafe {
        let out_len = input.len() * std::mem::size_of::<T>();
        std::slice::from_raw_parts_mut(input.as_mut_ptr() as *mut u8, out_len)
    }
}
