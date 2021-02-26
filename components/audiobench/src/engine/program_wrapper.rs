use crate::engine::data_transfer::{FeedbackData, GlobalData, GlobalParameters, IOData, NoteData};
use array_macro::array;
use jlrs_derive::IntoJulia;
use julia_helper::{DataType, ExecutionEngine, GeneratedCode, JuliaStruct, TypedArray, Value};
use std::collections::HashSet;

/// The MIDI protocol can provide notes at 128 different pitches.
const NUM_MIDI_NOTES: usize = 128;
/// Notes must be silent for at least this amount of time before they will be shut off.
const MIN_SILENT_TIME: f32 = 0.1;
/// Notes must have every sample be of this magnitude or less to be considered silent.
const SILENT_CUTOFF: f32 = 1e-5;

#[repr(C)]
#[derive(Clone, Copy, JuliaStruct, IntoJulia)]
#[jlrs(julia_type = "Main.Registry.Factory.Lib.NoteInput")]
struct NoteInput {
    pub pitch: f32,
    pub velocity: f32,
    pub elapsed_time: f32,
    pub elapsed_beats: f32,
    pub start_trigger: bool,
    pub release_trigger: bool,
}

impl NoteInput {
    fn from(other: &NoteData, params: &GlobalParameters, pitch_mul: f32) -> Self {
        Self {
            pitch: other.pitch * pitch_mul,
            velocity: other.velocity,
            elapsed_time: other.elapsed_samples as f32 / params.sample_rate as f32,
            elapsed_beats: other.elapsed_beats,
            start_trigger: other.start_trigger,
            release_trigger: other.release_trigger,
        }
    }
}

#[derive(Debug)]
struct CompleteNoteData {
    data: NoteData,
    silent_samples: usize,
    static_index: usize,
}

pub struct NoteTracker {
    held_notes: [Option<CompleteNoteData>; NUM_MIDI_NOTES],
    decaying_notes: Vec<CompleteNoteData>,
    reserved_static_indexes: HashSet<usize>,
}

impl NoteTracker {
    pub fn new() -> Self {
        Self {
            held_notes: array![None; NUM_MIDI_NOTES],
            decaying_notes: Vec::new(),
            reserved_static_indexes: HashSet::new(),
        }
    }

    pub fn silence_all(&mut self) {
        self.held_notes = array![None; NUM_MIDI_NOTES];
        self.decaying_notes.clear();
        self.reserved_static_indexes.clear();
    }

    fn equal_tempered_tuning(index: usize) -> f32 {
        // MIDI note 69 is 440Hz. 12 notes is an octave (double / half frequency).
        440.0 * (2.0f32).powf((index as i32 - 69) as f32 / 12.0)
    }

    pub fn start_note(&mut self, index: usize, velocity: f32) -> usize {
        if let Some(note) = &self.held_notes[index] {
            return note.static_index;
        }
        let mut static_index = 0;
        while self.reserved_static_indexes.contains(&static_index) {
            static_index += 1;
        }
        self.reserved_static_indexes.insert(static_index);
        let static_index = static_index;
        self.held_notes[index] = Some(CompleteNoteData {
            data: NoteData {
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
        static_index
    }

    pub fn release_note(&mut self, index: usize) {
        if let Some(mut note) = self.held_notes[index].take() {
            note.data.start_trigger = false;
            note.data.release_trigger = true;
            self.decaying_notes.push(note);
        }
    }

    fn advance_all_notes(&mut self, global_params: &GlobalParameters, global_data: &GlobalData) {
        let sample_rate = global_params.sample_rate as f32;
        let buffer_len = global_params.buffer_length;
        let min_silent_samples = (MIN_SILENT_TIME * sample_rate) as usize;
        let buffer_beats = global_data.bpm / 60.0 * buffer_len as f32 / sample_rate;
        for index in (0..self.decaying_notes.len()).rev() {
            let note = &mut self.decaying_notes[index];
            if note.silent_samples >= min_silent_samples {
                assert!(self.reserved_static_indexes.remove(&note.static_index));
                self.decaying_notes.remove(index);
            } else {
                note.data.elapsed_samples += buffer_len;
                note.data.elapsed_beats += buffer_beats;
                note.data.start_trigger = false;
                note.data.release_trigger = false;
            }
        }
        for note in self.held_notes.iter_mut().filter_map(|o| o.as_mut()) {
            note.data.elapsed_samples += buffer_len;
            note.data.elapsed_beats += buffer_beats;
            note.data.start_trigger = false;
        }
    }

    fn recommend_note_for_feedback(&self) -> Option<usize> {
        let mut youngest_time = std::usize::MAX;
        for note in self.held_notes.iter().filter_map(|o| o.as_ref()) {
            youngest_time = youngest_time.min(note.data.elapsed_samples);
        }
        // If there are no held notes, it is okay to display a decaying note insteaad.
        if youngest_time == std::usize::MAX {
            for note in &self.decaying_notes {
                youngest_time = youngest_time.min(note.data.elapsed_samples);
            }
        }
        for note in self.held_notes.iter().filter_map(|o| o.as_ref()) {
            if note.data.elapsed_samples == youngest_time {
                return Some(note.static_index);
            }
        }
        for note in &self.decaying_notes {
            if note.data.elapsed_samples == youngest_time {
                return Some(note.static_index);
            }
        }
        None
    }

    fn active_notes_mut(&mut self) -> impl Iterator<Item = &mut CompleteNoteData> {
        // println!(
        //     "{} held notes, {} decaying notes.",
        //     self.held_notes.iter().filter(|i| i.is_some()).count(),
        //     self.decaying_notes.len(),
        // );
        let held_iter = self.held_notes.iter_mut().filter_map(|o| o.as_mut());
        let decaying_iter = self.decaying_notes.iter_mut();
        held_iter.chain(decaying_iter)
    }
}

pub(super) struct AudiobenchExecutor {
    base: ExecutionEngine,
    parameters: GlobalParameters,
    registry_source: GeneratedCode,
    generated_source: GeneratedCode,
    loaded: bool,
}

impl AudiobenchExecutor {
    fn beautify_stack_trace(trace: String) -> String {
        let mut trace = &trace[..];
        let mut result = String::new();
        while let Some(index) = trace.find("exec(") {
            result.push_str(&trace[..index + 5]);
            trace = &trace[index + 5..];
            let end = trace.find(')').unwrap_or(0);
            trace = &trace[end..];
        }
        result.push_str(trace);
        result
    }

    pub fn new(
        registry_source: GeneratedCode,
        parameters: &GlobalParameters,
    ) -> Result<Self, String> {
        let mut base = ExecutionEngine::new();
        base.add_global_code(julia_helper::include_packed_library!("StaticArrays"))
            .unwrap();
        let mut this = AudiobenchExecutor {
            base,
            // This is a quick and dirty way of getting the executor to rebuild when we use
            // change_parameters later.
            parameters: GlobalParameters {
                channels: 999,
                buffer_length: 999,
                sample_rate: 999,
            },
            registry_source,
            generated_source: GeneratedCode::from_unique_source("blank", ""),
            loaded: false,
        };
        this.change_parameters(parameters)?;
        Ok(this)
    }

    pub fn change_parameters(&mut self, parameters: &GlobalParameters) -> Result<(), String> {
        if &self.parameters == parameters {
            return Ok(());
        }
        self.loaded = false;
        self.parameters = parameters.clone();
        let parameter_code = format!(
            concat!(
                "module Parameters\n",
                "    const channels = {}\n",
                "    const buffer_length = {}\n",
                "    const sample_rate = {}f0\n",
                "    export channels, buffer_length, sample_rate\n",
                "end\n",
            ),
            parameters.channels, parameters.buffer_length, parameters.sample_rate
        );
        let parameter_code =
            GeneratedCode::from_unique_source("Generated:parameters.jl", &parameter_code);
        self.base
            .add_global_code(parameter_code)
            .map_err(Self::beautify_stack_trace)?;
        // Redefine the registry module because it may have been previously compiled with old
        // parameters.
        self.base
            .add_global_code(self.registry_source.clone())
            .map_err(Self::beautify_stack_trace)?;
        // Redefine the Generated module to be blank because it may have been previously compiled
        // with old parameters.
        self.base
            .add_global_code(self.generated_source.clone())
            .map_err(Self::beautify_stack_trace)?;
        Ok(())
    }

    pub fn change_generated_code(&mut self, generated_code: GeneratedCode) -> Result<(), String> {
        let mut temp_file = std::env::temp_dir();
        temp_file.push("audiobench_note_graph_code.jl");
        if std::fs::write(temp_file.clone(), generated_code.as_str()).is_err() {
            return Err(format!(
                "ERROR: Failed to open {:?} for writing.",
                temp_file
            ));
        }
        self.generated_source = generated_code.clone();
        self.base
            .add_global_code(generated_code)
            .map_err(Self::beautify_stack_trace)?;
        self.loaded = true;
        Ok(())
    }

    pub fn reset_static_data(&mut self, index: usize) -> Result<(), String> {
        self.base.call_fn(
            &["Main", "Generated", "static_init"],
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
        do_feedback: bool,
        global_data: &GlobalData,
        notes: &mut NoteTracker,
        dyn_data: &[IOData],
        audio_output: &mut [f32],
    ) -> Result<Option<FeedbackData>, String> {
        let channels = self.parameters.channels;
        let buf_len = self.parameters.buffer_length;
        assert!(audio_output.len() == buf_len * channels);

        for i in 0..buf_len * channels {
            audio_output[i] = 0.0;
        }
        let feedback_note = if do_feedback {
            notes.recommend_note_for_feedback()
        } else {
            None
        };
        let mut feedback_data = None;

        let pitch_mul = (2.0f32).powf(global_data.pitch_wheel * 7.0 / 12.0);
        for note in notes.active_notes_mut() {
            let note_input = NoteInput::from(&note.data, &self.parameters, pitch_mul);
            let static_index = note.static_index;
            let do_feedback = feedback_note == Some(static_index);

            let res = self.base.call_fn(
                &["Main", "Generated", "exec"],
                |frame, inputs| {
                    inputs.append(&mut global_data.as_julia_values(frame)?);
                    inputs.push(Value::new(frame, do_feedback)?);
                    inputs.push(Value::new(frame, note_input)?);
                    inputs.push(Value::new(frame, static_index)?);
                    for item in dyn_data {
                        inputs.push(item.as_julia_value(frame)?);
                    }
                    Ok(())
                },
                |frame, output| {
                    // 0-based index, not Julia index.
                    let audio = match output.get_nth_field(frame, 0) {
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

                    if do_feedback {
                        let julia_feedback = match output.get_nth_field(frame, 1) {
                            Ok(v) => v,
                            Err(err) => {
                                return Ok(Err(format!(
                                    "ERROR: Failed to retrieve feedback data, caused by:\n{:?}",
                                    err
                                )))
                            }
                        };
                        let mut native_feedback = FeedbackData::default();
                        for index in 0..julia_feedback.n_fields() {
                            let field = julia_feedback.get_nth_field(frame, index)?;
                            let field = field.cast::<TypedArray<'_, '_, f32>>()?;
                            let field = field.inline_data(frame)?.into_slice();
                            native_feedback.0.push(Vec::from(field));
                        }
                        feedback_data = Some(native_feedback);
                    }

                    Ok(Ok(()))
                },
            );
            res.map_err(Self::beautify_stack_trace)??;
        }

        notes.advance_all_notes(&self.parameters, global_data);
        Ok(feedback_data)
    }
}
