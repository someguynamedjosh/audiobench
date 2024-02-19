use std::collections::HashSet;

use array_macro::array;

use crate::engine::data_transfer::{FeedbackData, GlobalData, GlobalParameters, IOData, NoteData};

/// The MIDI protocol can provide notes at 128 different pitches.
const NUM_MIDI_NOTES: usize = 128;
/// Notes must be silent for at least this amount of time before they will be
/// shut off.
const MIN_SILENT_TIME: f32 = 0.1;
/// Notes must have every sample be of this magnitude or less to be considered
/// silent.
const SILENT_CUTOFF: f32 = 1e-5;

#[repr(C)]
#[derive(Clone, Copy)]
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
    dummy_note: Option<CompleteNoteData>,
    held_notes: [Option<CompleteNoteData>; NUM_MIDI_NOTES],
    decaying_notes: Vec<CompleteNoteData>,
    reserved_static_indexes: HashSet<usize>,
    static_indexes_to_reset: Vec<usize>,
}

impl NoteTracker {
    pub fn new() -> Self {
        Self {
            dummy_note: None,
            held_notes: array![None; NUM_MIDI_NOTES],
            decaying_notes: Vec::new(),
            reserved_static_indexes: HashSet::new(),
            static_indexes_to_reset: Vec::new(),
        }
    }

    fn reserve_static_index(&mut self) -> usize {
        let mut static_index = 0;
        while self.reserved_static_indexes.contains(&static_index) {
            static_index += 1;
        }
        self.reserved_static_indexes.insert(static_index);
        self.static_indexes_to_reset.push(static_index);
        static_index
    }

    pub fn start_dummy_note(&mut self) {
        if self.dummy_note.is_none() {
            let static_index = self.reserve_static_index();
            self.dummy_note = Some(CompleteNoteData {
                data: NoteData {
                    pitch: 440.0,
                    velocity: 1.0,
                    elapsed_samples: 0,
                    elapsed_beats: 0.0,
                    start_trigger: true,
                    release_trigger: false,
                },
                silent_samples: 0,
                static_index,
            });
        }
    }

    pub fn stop_dummy_note(&mut self) {
        if let Some(note) = self.dummy_note.take() {
            self.reserved_static_indexes.remove(&note.static_index);
        }
    }

    pub fn set_dummy_note_active(&mut self, should_be_active: bool) {
        if should_be_active {
            // Only sets up the dummy note if it was previouisly None.
            self.start_dummy_note()
        } else {
            self.stop_dummy_note()
        }
    }

    pub fn silence_all(&mut self) {
        self.stop_dummy_note();
        self.held_notes = array![None; NUM_MIDI_NOTES];
        self.decaying_notes.clear();
        self.reserved_static_indexes.clear();
    }

    fn equal_tempered_tuning(index: usize) -> f32 {
        // MIDI note 69 is 440Hz. 12 notes is an octave (double / half frequency).
        440.0 * (2.0f32).powf((index as i32 - 69) as f32 / 12.0)
    }

    pub fn start_note(&mut self, index: usize, velocity: f32) {
        if self.held_notes[index].is_some() {
            return;
        }
        let static_index = self.reserve_static_index();
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
        if let Some(note) = &mut self.dummy_note {
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
        if let Some(note) = &self.dummy_note {
            Some(note.static_index)
        } else {
            None
        }
    }

    fn active_notes_mut(&mut self) -> impl Iterator<Item = &mut CompleteNoteData> {
        let dummy_iter = self.dummy_note.iter_mut();
        let held_iter = self.held_notes.iter_mut().filter_map(|o| o.as_mut());
        let decaying_iter = self.decaying_notes.iter_mut();
        dummy_iter.chain(held_iter.chain(decaying_iter))
    }
}

pub(super) struct AudiobenchExecutor {
    parameters: GlobalParameters,
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

    pub fn new(parameters: &GlobalParameters) -> Result<Self, String> {
        let mut this = AudiobenchExecutor {
            // This is a quick and dirty way of getting the executor to rebuild when we use
            // change_parameters for the first time.
            parameters: GlobalParameters {
                channels: 999,
                buffer_length: 999,
                sample_rate: 999,
            },
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
        Ok(())
    }


    fn reset_static_data(&mut self, index: usize) -> Result<(), String> {
        // todo!()
        Ok(())
    }

    // Runs the main function once to make sure everything is compiled.
    pub fn preheat(&mut self, notes: &mut NoteTracker, dyn_data: &[IOData]) -> Result<(), String> {
        let was_dummy_note_active = notes.dummy_note.is_some();
        notes.start_dummy_note();
        for index in std::mem::take(&mut notes.static_indexes_to_reset) {
            self.reset_static_data(index)?;
        }
        let note = notes.dummy_note.as_ref().unwrap();
        let note_input = NoteInput::from(&note.data, &self.parameters, 1.0);
        let static_index = note.static_index;
        let global_data = GlobalData::new();
        // todo!();
        notes.set_dummy_note_active(was_dummy_note_active);
        Ok(())
    }

    /// This handles everything from global setup, note iteration, program
    /// execution, note teardown, and finally global teardown. Returns true
    /// if feedback data was updated. View index is which module's outputs
    /// should be retrieved.
    pub fn execute(
        &mut self,
        do_feedback: bool,
        view_index: usize,
        global_data: &GlobalData,
        notes: &mut NoteTracker,
        dyn_data: &[IOData],
        audio_output: &mut [f32],
    ) -> Result<Option<FeedbackData>, String> {
        for index in std::mem::take(&mut notes.static_indexes_to_reset) {
            self.reset_static_data(index)?;
        }

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
        let mut is_dummy = notes.dummy_note.is_some();
        for note in notes.active_notes_mut() {
            let note_input = NoteInput::from(&note.data, &self.parameters, pitch_mul);
            let static_index = note.static_index;
            let do_feedback = feedback_note == Some(static_index);
            todo!("Execute");
        }

        notes.advance_all_notes(&self.parameters, global_data);
        Ok(feedback_data)
    }
}
