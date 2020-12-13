use crate::engine::data_transfer::{
    DataFormat, GlobalInput, GlobalParameters, InputPacker, NoteInput, OutputUnpacker,
};
use crate::registry::Registry;
use array_macro::array;
use julia_helper::{ExecutionEngine, GeneratedCode, TypedArray, Value};
use shared_util::{perf_counter::sections, prelude::*};
use std::collections::HashSet;

/// The MIDI protocol can provide notes at 128 different pitches.
const NUM_MIDI_NOTES: usize = 128;
/// Notes must be silent for at least this amount of time before they will be shut off.
const MIN_SILENT_TIME: f32 = 0.1;
/// Notes must have every sample be of this magnitude or less to be considered silent.
const SILENT_CUTOFF: f32 = 1e-5;

struct CompleteNoteInput {
    input_data: NoteInput,
    silent_samples: usize,
    static_index: usize,
}

pub struct NoteTracker {
    held_notes: [Option<CompleteNoteInput>; NUM_MIDI_NOTES],
    decaying_notes: Vec<CompleteNoteInput>,
    data_format: DataFormat,
    reserved_static_indexes: HashSet<usize>,
}

impl NoteTracker {
    pub fn new(data_format: DataFormat) -> Self {
        Self {
            held_notes: array![None; NUM_MIDI_NOTES],
            decaying_notes: Vec::new(),
            data_format,
            reserved_static_indexes: HashSet::new(),
        }
    }

    pub fn silence_all(&mut self) {
        self.held_notes = array![None; NUM_MIDI_NOTES];
        self.decaying_notes.clear();
    }

    pub fn set_data_format(&mut self, data_format: DataFormat) {
        self.data_format = data_format;
        // Old notes may not be compatible with the new data data_format.
        self.silence_all();
    }

    fn equal_tempered_tuning(index: usize) -> f32 {
        // MIDI note 69 is 440Hz. 12 notes is an octave (double / half frequency).
        440.0 * (2.0f32).powf((index as i32 - 69) as f32 / 12.0)
    }

    pub fn start_note(&mut self, index: usize, velocity: f32) {
        let mut static_index = 0;
        while self.reserved_static_indexes.contains(&static_index) {
            static_index += 1;
        }
        self.reserved_static_indexes.insert(static_index);
        let static_index = static_index;
        if self.held_notes[index].is_some() {
            return;
        }
        self.held_notes[index] = Some(CompleteNoteInput {
            input_data: NoteInput {
                pitch: Self::equal_tempered_tuning(index),
                velocity,
                elapsed_samples: 0,
                elapsed_beats: 0.0,
                start_trigger: true,
                release_trigger: false,
            },
            silent_samples: 0,
            static_index,
        });
    }

    pub fn release_note(&mut self, index: usize) {
        if let Some(mut note) = self.held_notes[index].take() {
            note.input_data.start_trigger = false;
            note.input_data.release_trigger = true;
            self.decaying_notes.push(note);
        }
    }

    fn advance_all_notes(&mut self, global_input: &GlobalInput) {
        let sample_rate = self.data_format.global_params.sample_rate as f32;
        let buffer_len = self.data_format.global_params.buffer_length;
        let min_silent_samples = (MIN_SILENT_TIME * sample_rate) as usize;
        let buffer_beats = global_input.bpm / 60.0 * buffer_len as f32 / sample_rate;
        for index in (0..self.decaying_notes.len()).rev() {
            let note = &mut self.decaying_notes[index];
            if note.silent_samples >= min_silent_samples {
                assert!(self.reserved_static_indexes.remove(&note.static_index));
                self.decaying_notes.remove(index);
            } else {
                note.input_data.elapsed_samples += buffer_len;
                note.input_data.elapsed_beats += buffer_beats;
                note.input_data.start_trigger = false;
                note.input_data.release_trigger = false;
            }
        }
        for note in self.held_notes.iter_mut().filter_map(|o| o.as_mut()) {
            note.input_data.elapsed_samples += buffer_len;
            note.input_data.elapsed_beats += buffer_beats;
            note.input_data.start_trigger = false;
        }
    }

    fn recommend_note_for_feedback(&self) -> Option<usize> {
        let mut youngest_time = std::usize::MAX;
        for note in self.held_notes.iter().filter_map(|o| o.as_ref()) {
            youngest_time = youngest_time.min(note.input_data.elapsed_samples);
        }
        // If there are no held notes, it is okay to display a decaying note insteaad.
        if youngest_time == std::usize::MAX {
            for note in &self.decaying_notes {
                youngest_time = youngest_time.min(note.input_data.elapsed_samples);
            }
        }
        let mut index = 0;
        for note in self.held_notes.iter().filter_map(|o| o.as_ref()) {
            if note.input_data.elapsed_samples == youngest_time {
                return Some(index);
            }
            index += 1;
        }
        for note in &self.decaying_notes {
            if note.input_data.elapsed_samples == youngest_time {
                return Some(index);
            }
            index += 1;
        }
        None
    }

    fn active_notes_mut(&mut self) -> impl Iterator<Item = &mut CompleteNoteInput> {
        let held_iter = self.held_notes.iter_mut().filter_map(|o| o.as_mut());
        let decaying_iter = self.decaying_notes.iter_mut();
        held_iter.chain(decaying_iter)
    }
}

pub(super) struct AudiobenchExecutor {
    base: ExecutionEngine,
    parameters: GlobalParameters,
    registry_source: GeneratedCode,
    loaded: bool,
}

impl AudiobenchExecutor {
    pub fn new(registry: &Registry, parameters: &GlobalParameters) -> Result<Self, String> {
        let mut base = ExecutionEngine::new();
        let mut registry_source = GeneratedCode::new();
        registry_source.append("module Registry\n", "generated");
        for (lib_name, _) in &Vec::<(String, i32)>::new() { //registry.borrow_library_infos() {
            registry_source.append(&format!("\nmodule {}\n", lib_name), "generated");
            for file_content in registry.borrow_general_scripts_from_library(lib_name) {
                registry_source.append_clip(file_content);
            }
            for (mod_name, file_content) in registry.borrow_module_scripts_from_library(lib_name) {
                registry_source.append(&format!("module {}\n", mod_name), "generated");
                registry_source.append_clip(file_content);
                registry_source.append(&format!("\nend # module {}\n", mod_name), "generated");
            }
            registry_source.append(&format!("\nend # module {}\n", lib_name), "generated");
        }
        registry_source.append("\nend # module Registry\n", "generated");
        println!("{}", registry_source.as_str());
        let mut res = AudiobenchExecutor {
            base,
            parameters: parameters.clone(),
            registry_source,
            loaded: false,
        };
        res.change_parameters(parameters)?;
        Ok(res)
    }

    pub fn change_parameters(&mut self, parameters: &GlobalParameters) -> Result<(), String> {
        self.loaded = false;
        let mut parameter_code = format!(
            concat!(
                "module Parameters\n",
                "    const channels = {}\n",
                "    const buffer_length = {}\n",
                "    const sample_rate = {}f0\n",
                "end\n",
            ),
            parameters.channels, parameters.buffer_length, parameters.sample_rate
        );
        let parameter_code =
            GeneratedCode::from_unique_source("Generated:parameters.jl", &parameter_code);
        self.base.add_global_code(parameter_code)?;
        // Redefine the registry module because it may have been previously compiled with old
        // parameters.
        self.base.add_global_code(self.registry_source.clone())?;
        // Redefine the Generated module to be blank because it may have been previously compiled
        // with old parameters.
        let blank_mod = GeneratedCode::from_unique_source("Generated", "module Generated end");
        self.base.add_global_code(blank_mod)?;
        Ok(())
    }

    pub fn change_generated_code(&mut self, generated_code: GeneratedCode) -> Result<(), String> {
        println!("{}", generated_code.as_str());
        self.base.add_global_code(generated_code)?;
        self.loaded = true;
        Ok(())
    }

    /// If this returns false, you should not call reset_static_data or execute as they are
    /// guaranteed to return errors.
    pub fn is_generated_code_loaded(&self) -> bool {
        self.loaded
    }

    pub fn reset_static_data(&mut self, index: usize) -> Result<(), String> {
        self.base.call_fn(
            &["Main", "Generated", "init_static"],
            |frame, inputs| {
                inputs.push(Value::new(frame, index)?);
                Ok(())
            },
            |_, _| Ok(()),
        )
    }

    /// This handles everything from global setup, note iteration, program execution, note teardown,
    /// and finally global teardown. Returns true if feedback data was updated.
    pub fn execute(
        &mut self,
        update_feedback: bool,
        global_input: &mut GlobalInput,
        notes: &mut NoteTracker,
        audio_output: &mut [f32],
        perf_counter: &mut impl PerfCounter,
    ) -> Result<bool, String> {
        let section = perf_counter.begin_section(&sections::GLOBAL_SETUP);
        let channels = self.parameters.channels;
        let buf_len = self.parameters.buffer_length;
        let sample_rate = self.parameters.sample_rate;
        let buf_time = buf_len as f32 / sample_rate as f32;
        assert!(audio_output.len() == buf_len * channels);

        for i in 0..buf_len * channels {
            audio_output[i] = 0.0;
        }
        let feedback_note = if update_feedback {
            notes.recommend_note_for_feedback()
        } else {
            None
        };
        perf_counter.end_section(section);

        for note in notes.active_notes_mut() {
            let section = perf_counter.begin_section(&sections::NOTE_SETUP);
            let note_input = note.input_data.clone();
            let static_index = note.static_index;
            perf_counter.end_section(section);

            let section = perf_counter.begin_section(&sections::NODESPEAK_EXEC);
            self.base.call_fn(
                &["Main", "Generated", "exec"],
                |frame, inputs| {
                    inputs.append(&mut global_input.as_julia_values(frame)?);
                    inputs.push(Value::new(frame, note_input)?);
                    inputs.push(Value::new(frame, static_index)?);
                    Ok(())
                },
                |frame, output| {
                    perf_counter.end_section(section);
                    let audio = output.get_field(frame, "audio");
                    let audio = match audio {
                        Ok(v) => v,
                        Err(err) => {
                            return Ok(Err(format!(
                                "ERROR: Failed to retrieve audio output, caused by:\n{:?}",
                                err
                            )))
                        }
                    };
                    let audio = match audio.cast::<TypedArray<'_, '_, f32>>() {
                        Ok(v) => v,
                        Err(err) => {
                            return Ok(Err(format!(
                                "ERROR: audio is not expected type, caused by:\n{:?}",
                                err
                            )))
                        }
                    };
                    let audio = audio.inline_data(frame)?.into_slice();
                    let section = perf_counter.begin_section(&sections::NOTE_FINALIZE);
                    let mut silent = true;
                    for i in 0..buf_len * channels {
                        audio_output[i] += audio[i];
                        silent &= audio[i].abs() < SILENT_CUTOFF;
                    }
                    if silent {
                        note.silent_samples += buf_len;
                    } else {
                        note.silent_samples = 0;
                    }
                    perf_counter.end_section(section);
                    Ok(Ok(()))
                },
            )??;
        }

        let section = perf_counter.begin_section(&sections::GLOBAL_FINALIZE);
        notes.advance_all_notes(global_input);
        global_input.elapsed_time += buf_time;
        global_input.elapsed_beats += buf_time * global_input.bpm / 60.0;
        perf_counter.end_section(section);
        Ok(feedback_note.is_some())
    }
}
