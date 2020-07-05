use super::parts::{Control, Module};
use crate::registry::Registry;
use crate::util::*;
use array_macro::array;
use nodespeak::llvmir::structure::{Program, StaticData};
use nodespeak::Compiler;

const NUM_MIDI_NOTES: usize = 128;
// Notes must be silent for at least this amount of time before they will be shut off.
const MIN_SILENT_TIME: f32 = 0.1;
// Notes must have every sample be of this magnitude or less to be considered silent.
const SILENT_CUTOFF: f32 = 1e-5;

// This packages changes made by the user to knobs and automation into a format that can be read
// by the nodespeak parameter, so that trivial changes don't necessitate a recompile.
pub(super) struct AuxDataCollector {
    ordered_controls: Vec<Rcrc<Control>>,
    data_length: usize,
}

impl AuxDataCollector {
    pub(super) fn new(ordered_controls: Vec<Rcrc<Control>>, data_length: usize) -> Self {
        Self {
            ordered_controls,
            data_length,
        }
    }

    pub(super) fn collect_data(&self) -> Vec<f32> {
        let mut data = Vec::with_capacity(self.data_length);
        for control in &self.ordered_controls {
            let control_ref = control.borrow();
            if control_ref.automation.len() == 0 {
                data.push(control_ref.value);
            } else {
                let num_lanes = control_ref.automation.len();
                let multiplier = 1.0 / num_lanes as f32;
                for lane in &control_ref.automation {
                    // algebraic simplification of remapping value [-1, 1] -> [0, 1] -> [min, max]
                    let a = (lane.range.1 - lane.range.0) / 2.0;
                    let b = a + lane.range.0;
                    data.push(a * multiplier);
                    data.push(b * multiplier);
                }
            }
        }
        debug_assert!(data.len() == self.data_length);
        data
    }
}

pub(super) struct FeedbackDisplayer {
    ordered_modules: Vec<Rcrc<Module>>,
    data_length: usize,
}

impl FeedbackDisplayer {
    pub(super) fn new(ordered_modules: Vec<Rcrc<Module>>, data_length: usize) -> Self {
        Self {
            ordered_modules,
            data_length,
        }
    }

    pub(super) fn display_feedback(&mut self, feedback_data: &[f32]) {
        assert!(feedback_data.len() == self.data_length);
        let mut data_pos = 0;
        for module in &self.ordered_modules {
            let module_ref = module.borrow_mut();
            let module_data_length = module_ref.template.borrow().feedback_data_len;
            if let Some(data_ptr) = &module_ref.feedback_data {
                let slice = &feedback_data[data_pos..data_pos + module_data_length];
                data_ptr.borrow_mut().clone_from_slice(slice);
            }
            data_pos += module_data_length;
        }
        debug_assert!(data_pos == self.data_length);
    }
}

pub(super) struct HostData {
    // MIDI specifies each MIDI Channel has 128 controls.
    pub(super) controller_values: [f32; 128],
    // The pitch wheel is seperate from other controls due to its higher precision.
    pub(super) pitch_wheel_value: f32,
    pub(super) bpm: f32,
    pub(super) song_time: f32,
    pub(super) song_beats: f32,
}

impl HostData {
    pub(super) fn new() -> Self {
        Self {
            controller_values: [0.0; 128],
            pitch_wheel_value: 0.0,
            bpm: 120.0,
            song_time: 0.0,
            song_beats: 0.0,
        }
    }
}

pub(super) struct NoteData {
    pitch: f32,
    velocity: f32,
    elapsed_samples: usize,
    elapsed_beats: f32,
    silent_samples: usize,
    start_trigger: bool,
    release_trigger: bool,
    static_data: StaticData,
}

#[derive(Clone, PartialEq, Eq)]
pub(super) struct HostFormat {
    pub(super) sample_rate: usize,
    pub(super) buffer_len: usize,
}

#[derive(Clone, PartialEq, Eq)]
pub(super) struct DataFormat {
    pub(super) sample_rate: usize,
    pub(super) buffer_len: usize,
    pub(super) aux_data_len: usize,
    pub(super) feedback_data_len: usize,
}

pub(super) struct InputPacker {
    packed_data: Vec<f32>,
    fixed_inputs_len: usize,
    aux_data_len: usize,
    format: DataFormat,
}

// global_pitch: FLOAT
// global_velocity: FLOAT
// global_note_status: FLOAT
// global_bpm: FLOAT
// global_note_time: [BL]FLOAT
// global_note_beats: [BL]FLOAT
// global_song_time: [BL]FLOAT
// global_song_beats: [BL]FLOAT
// global_midi_controls: [128]FLOAT
// global_aux_data: [starting_aux_data.len()]FLOAT
impl InputPacker {
    pub(super) fn new(format: DataFormat) -> Self {
        let mut result = Self {
            packed_data: Vec::new(),
            fixed_inputs_len: 0,
            aux_data_len: 0,
            format: format.clone(),
        };
        result.set_data_format(format);
        result
    }

    pub(super) fn set_timing_input(&mut self, start_index: usize, factory: f32, increment: f32) {
        let data = &mut self.packed_data[start_index..self.format.buffer_len + start_index];
        let mut value = factory;
        for index in 0..self.format.buffer_len {
            data[index] = value;
            value += increment;
        }
    }

    pub(super) fn set_data_format(&mut self, format: DataFormat) {
        self.fixed_inputs_len = 5 + 4 * format.buffer_len + 128;
        self.aux_data_len = format.aux_data_len;
        self.packed_data
            .resize(self.fixed_inputs_len + self.aux_data_len, 0.0);
        self.format = format;
    }

    pub(super) fn set_host_data(&mut self, host_data: &HostData) {
        self.packed_data[4] = host_data.bpm;
        let midi_controls_start = 5 + 4 * self.format.buffer_len;
        let midi_controls_data =
            &mut self.packed_data[midi_controls_start..midi_controls_start + 128];
        for index in 0..128 {
            midi_controls_data[index] = host_data.controller_values[index];
        }
        self.set_timing_input(
            5 + 2 * self.format.buffer_len,
            host_data.song_time,
            1.0 / self.format.sample_rate as f32,
        );
        self.set_timing_input(
            5 + 3 * self.format.buffer_len,
            host_data.song_beats,
            host_data.bpm / 60.0 / self.format.sample_rate as f32,
        );
    }

    pub(super) fn set_note_data(
        &mut self,
        note_data: &NoteData,
        host_data: &HostData,
        update_feedback: bool,
    ) {
        self.packed_data[0] = note_data.pitch;
        self.packed_data[1] = note_data.velocity;
        self.packed_data[2] = if note_data.start_trigger {
            2.0
        } else if note_data.release_trigger {
            1.0
        } else {
            0.0
        };
        self.packed_data[3] = if update_feedback { 1.0 } else { 0.0 };
        let elapsed_seconds = note_data.elapsed_samples as f32 / self.format.sample_rate as f32;
        self.set_timing_input(5, elapsed_seconds, 1.0 / self.format.sample_rate as f32);
        self.set_timing_input(
            5 + self.format.buffer_len,
            elapsed_seconds * host_data.bpm / 60.0,
            1.0 / self.format.sample_rate as f32,
        );
    }

    pub(super) fn set_aux_data(&mut self, new_data: &[f32]) {
        assert!(new_data.len() == self.format.aux_data_len);
        (&mut self.packed_data[self.fixed_inputs_len..]).copy_from_slice(new_data);
    }

    pub(super) fn borrow_raw_mut(&mut self) -> &mut [u8] {
        self.packed_data.as_raw_mut()
    }
}

pub(super) struct OutputUnpacker {
    packed_data: Vec<f32>,
    audio_len: usize,
    feedback_data_len: usize,
    format: DataFormat,
}

impl OutputUnpacker {
    pub(super) fn new(format: DataFormat) -> Self {
        let mut result = Self {
            packed_data: Vec::new(),
            audio_len: 0,
            feedback_data_len: 0,
            format: format.clone(),
        };
        result.set_data_format(format);
        result
    }

    pub(super) fn set_data_format(&mut self, format: DataFormat) {
        self.audio_len = format.buffer_len * 2;
        self.feedback_data_len = format.feedback_data_len;
        self.packed_data
            .resize(self.audio_len + self.feedback_data_len, 0.0);
        self.format = format;
    }

    pub(super) fn borrow_audio(&self) -> &[f32] {
        &self.packed_data[0..self.audio_len]
    }

    pub(super) fn borrow_feedback_data(&self) -> &[f32] {
        &self.packed_data[self.audio_len..]
    }

    pub(super) fn borrow_raw_mut(&mut self) -> &mut [u8] {
        self.packed_data.as_raw_mut()
    }
}

pub(super) struct NoteTracker {
    held_notes: [Option<NoteData>; NUM_MIDI_NOTES],
    decaying_notes: Vec<NoteData>,
    format: DataFormat,
}

impl NoteTracker {
    pub(super) fn new(format: DataFormat) -> Self {
        Self {
            held_notes: array![None; NUM_MIDI_NOTES],
            decaying_notes: Vec::new(),
            format,
        }
    }

    pub(super) fn silence_all(&mut self) {
        self.held_notes = array![None; NUM_MIDI_NOTES];
        self.decaying_notes.clear();
    }

    pub(super) fn set_data_format(&mut self, format: DataFormat) {
        self.format = format;
    }

    pub(super) fn equal_tempered_tuning(index: usize) -> f32 {
        // MIDI note 69 is 440Hz. 12 notes is an octave (double / half frequency).
        440.0 * (2.0f32).powf((index as i32 - 69) as f32 / 12.0)
    }

    pub(super) fn start_note(&mut self, static_data: StaticData, index: usize, velocity: f32) {
        if self.held_notes[index].is_some() {
            return;
        }
        self.held_notes[index] = Some(NoteData {
            pitch: Self::equal_tempered_tuning(index),
            velocity,
            elapsed_samples: 0,
            elapsed_beats: 0.0,
            silent_samples: 0,
            start_trigger: true,
            release_trigger: false,
            static_data,
        });
    }

    pub(super) fn release_note(&mut self, index: usize) {
        if let Some(mut note) = self.held_notes[index].take() {
            note.start_trigger = false;
            note.release_trigger = true;
            self.decaying_notes.push(note);
        }
    }

    pub(super) fn advance_all_notes(&mut self, host_data: &HostData) {
        let min_silent_samples = (MIN_SILENT_TIME * self.format.sample_rate as f32) as usize;
        let buffer_beats =
            host_data.bpm / 60.0 * self.format.buffer_len as f32 / self.format.sample_rate as f32;
        for index in (0..self.decaying_notes.len()).rev() {
            let note = &mut self.decaying_notes[index];
            if note.silent_samples >= min_silent_samples {
                self.decaying_notes.remove(index);
            } else {
                note.elapsed_samples += self.format.buffer_len;
                note.elapsed_beats += buffer_beats;
                note.start_trigger = false;
                note.release_trigger = false;
            }
        }
        for note in self.held_notes.iter_mut().filter_map(|o| o.as_mut()) {
            note.elapsed_samples += self.format.buffer_len;
            note.elapsed_beats += buffer_beats;
            note.start_trigger = false;
        }
    }

    pub(super) fn recommend_note_for_feedback(&self) -> Option<usize> {
        let mut youngest_time = std::usize::MAX;
        for note in self.held_notes.iter().filter_map(|o| o.as_ref()) {
            youngest_time = youngest_time.min(note.elapsed_samples);
        }
        // If there are no held notes, it is okay to display a decaying note insteaad.
        if youngest_time == std::usize::MAX {
            for note in &self.decaying_notes {
                youngest_time = youngest_time.min(note.elapsed_samples);
            }
        }
        let mut index = 0;
        for note in self.held_notes.iter().filter_map(|o| o.as_ref()) {
            if note.elapsed_samples == youngest_time {
                return Some(index);
            }
            index += 1;
        }
        for note in &self.decaying_notes {
            if note.elapsed_samples == youngest_time {
                return Some(index);
            }
            index += 1;
        }
        None
    }

    pub(super) fn active_notes_mut(&mut self) -> impl Iterator<Item = &mut NoteData> {
        let held_iter = self.held_notes.iter_mut().filter_map(|o| o.as_mut());
        let decaying_iter = self.decaying_notes.iter_mut();
        held_iter.chain(decaying_iter)
    }
}

pub(super) struct AudiobenchCompiler {
    compiler: Compiler,
}

impl AudiobenchCompiler {
    pub(super) fn new(registry: &Registry) -> Self {
        let mut compiler = Compiler::new();
        compiler.add_source("<note graph>".to_owned(), "".to_owned());
        compiler.add_source("!:lib.ns".to_owned(), include_str!("lib.ns").to_owned());
        for (name, content) in registry.borrow_scripts().iter() {
            compiler.add_source(name.to_owned(), content.to_owned());
        }
        Self { compiler }
    }

    pub(super) fn compile(&mut self, source: String) -> Result<AudiobenchProgram, String> {
        self.compiler.add_source("<note graph>".to_owned(), source);
        Ok(AudiobenchProgram {
            program: self.compiler.compile("<note graph>")?,
        })
    }
}

pub(super) struct AudiobenchProgram {
    program: Program,
}

impl AudiobenchProgram {
    pub(super) fn create_static_data(&mut self) -> Result<StaticData, String> {
        unsafe { self.program.create_static_data().map_err(|e| e.to_owned()) }
    }

    pub(super) fn execute(
        &mut self,
        update_feedback: bool,
        input: &mut InputPacker,
        output: &mut OutputUnpacker,
        host_data: &mut HostData,
        notes: &mut NoteTracker,
        audio_output: &mut [f32],
    ) -> Result<bool, String> {
        let buf_len = input.format.buffer_len;
        let sample_rate = input.format.sample_rate;
        let buf_time = buf_len as f32 / sample_rate as f32;
        assert!(input.format == output.format && output.format == notes.format);
        assert!(audio_output.len() == buf_len * 2);

        for i in 0..buf_len * 2 {
            audio_output[i] = 0.0;
        }
        input.set_host_data(&host_data);
        let feedback_note = if update_feedback {
            notes.recommend_note_for_feedback()
        } else {
            None
        };

        for (index, note) in notes.active_notes_mut().enumerate() {
            input.set_note_data(&note, host_data, feedback_note == Some(index));
            unsafe {
                self.program.execute_raw(
                    input.borrow_raw_mut(),
                    output.borrow_raw_mut(),
                    &mut note.static_data,
                )?;
            }
            let mut silent = true;
            for i in 0..buf_len * 2 {
                audio_output[i] += output.borrow_audio()[i];
                silent &= output.borrow_audio()[i].abs() < SILENT_CUTOFF;
            }
            if silent {
                note.silent_samples += input.format.buffer_len;
            } else {
                note.silent_samples = 0;
            }
        }

        notes.advance_all_notes(host_data);
        host_data.song_time += buf_time;
        host_data.song_beats += buf_time * host_data.bpm / 60.0;
        Ok(feedback_note.is_some())
    }
}
