use crate::engine::execution::{AuxMidiData, ExecEnvironment};
use crate::util::*;
use array_macro::array;

struct ActiveVoice {
    pitch: f32,
    velocity: f32,
    elapsed_samples: usize,
    elapsed_beats: f32,
    silent_samples: usize,
    start_trigger: bool,
    release_trigger: bool,
    static_data: nodespeak::llvmir::structure::StaticData,
}

const NUM_MIDI_NOTES: usize = 128;
// Notes must be silent for at least this amount of time before they will be shut off.
const MIN_SILENT_TIME: f32 = 0.1;
// Notes must have every sample be of this magnitude or less to be considered silent.
const SILENT_CUTOFF: f32 = 1e-5;

pub struct NoteManager {
    held_notes: [Option<ActiveVoice>; NUM_MIDI_NOTES],
    decaying_notes: Vec<ActiveVoice>,
    async_error: Option<String>,
}

impl NoteManager {
    pub fn new() -> Self {
        Self {
            held_notes: array![None; NUM_MIDI_NOTES],
            decaying_notes: Vec::new(),
            async_error: None,
        }
    }

    pub fn silence_all(&mut self) {
        for note_index in 0..NUM_MIDI_NOTES {
            self.held_notes[note_index] = None;
        }
        self.decaying_notes.clear();
    }

    // Optionally returns a vector containing feedback data collected while rendering notes.
    pub fn render_all_notes(
        &mut self,
        executor: &mut ExecEnvironment,
        aux_midi_data: &AuxMidiData,
        render_into: &mut [f32],
        mut collect_feedback_data: bool,
    ) -> Result<Option<Vec<f32>>, String> {
        // Pitch offset range is +- 2 semitones
        let pitch_multiplier = (2.0f32).powf(aux_midi_data.pitch_wheel_value * 2.0 / 12.0);
        let mut feedback_data = None;

        let buffer_length = executor.get_current_buffer_length();
        let sample_rate = executor.get_current_sample_rate();
        let time_per_sample = 1.0 / sample_rate as f32;
        assert!(render_into.len() == buffer_length * 2);
        for i in 0..buffer_length * 2 {
            render_into[i] = 0.0;
        }

        let mut shortest_voice_duration = std::usize::MAX;
        for note in self.held_notes.iter() {
            if let Some(voice) = note {
                shortest_voice_duration = shortest_voice_duration.min(voice.elapsed_samples);
            }
        }
        // If there are no held notes then it is okay to display info about a decaying note.
        if shortest_voice_duration == std::usize::MAX {
            for voice in self.decaying_notes.iter() {
                shortest_voice_duration = shortest_voice_duration.min(voice.elapsed_samples);
            }
        }

        executor.set_aux_midi_data(aux_midi_data);
        for note in self.held_notes.iter_mut() {
            if let Some(voice) = note {
                executor.set_pitch_input(voice.pitch * pitch_multiplier);
                executor.set_velocity_input(voice.velocity.to_range(-1.0, 1.0));
                executor.set_note_status_input(if voice.start_trigger { 2.0 } else { 0.0 });
                voice.start_trigger = false;
                executor.set_note_time_input(voice.elapsed_samples as f32 * time_per_sample);
                executor.set_note_beats_input(voice.elapsed_beats, aux_midi_data);

                let record_feedback_now =
                    if voice.elapsed_samples == shortest_voice_duration && collect_feedback_data {
                        collect_feedback_data = false;
                        executor.set_should_update_input(1.0);
                        true
                    } else {
                        executor.set_should_update_input(0.0);
                        false
                    };

                executor.execute(&mut voice.static_data)?;
                if record_feedback_now {
                    feedback_data = Some(Vec::from(executor.borrow_feedback_data()));
                }
                let voice_audio = executor.borrow_audio_out();
                debug_assert!(voice_audio.len() == render_into.len());
                for i in 0..voice_audio.len() {
                    render_into[i] += voice_audio[i];
                }
                voice.elapsed_samples += buffer_length;
                voice.elapsed_beats +=
                    buffer_length as f32 * aux_midi_data.bpm / 60.0 / sample_rate as f32;
            }
        }

        let min_silent_samples = (MIN_SILENT_TIME / time_per_sample) as usize;
        let mut voice_kill_list = Vec::new();
        for (voice_index, voice) in self.decaying_notes.iter_mut().enumerate() {
            executor.set_pitch_input(voice.pitch * pitch_multiplier);
            executor.set_velocity_input(voice.velocity.to_range(-1.0, 1.0));
            executor.set_note_status_input(if voice.release_trigger { 1.0 } else { 0.0 });
            voice.release_trigger = false;
            executor.set_note_time_input(voice.elapsed_samples as f32 * time_per_sample);

            let record_feedback_now =
                if voice.elapsed_samples == shortest_voice_duration && collect_feedback_data {
                    collect_feedback_data = false;
                    executor.set_should_update_input(1.0);
                    true
                } else {
                    executor.set_should_update_input(0.0);
                    false
                };

            executor.execute(&mut voice.static_data)?;
            if record_feedback_now {
                feedback_data = Some(Vec::from(executor.borrow_feedback_data()));
            }
            let voice_audio = executor.borrow_audio_out();
            let mut all_silent = true;
            for i in 0..voice_audio.len() {
                render_into[i] += voice_audio[i];
                if voice_audio[i].abs() > SILENT_CUTOFF {
                    all_silent = false;
                }
            }
            voice.elapsed_samples += buffer_length;
            if all_silent {
                voice.silent_samples += buffer_length;
                if voice.silent_samples >= min_silent_samples {
                    voice_kill_list.push(voice_index);
                }
            } else {
                voice.silent_samples = 0;
            }
        }

        // Iterate backwards because we are deleting things.
        for note_index in (0..self.decaying_notes.len()).rev() {
            if self.decaying_notes[note_index].silent_samples >= min_silent_samples {
                self.decaying_notes.remove(note_index);
            }
        }
        Ok(feedback_data)
    }

    pub fn note_on(&mut self, executor: &mut ExecEnvironment, note_index: i32, velocity: f32) {
        assert!(note_index < 128);
        let static_data = match executor.create_static_data() {
            Ok(data) => data,
            Err(err) => {
                self.async_error = Some(err);
                return;
            }
        };
        self.held_notes[note_index as usize] = Some(ActiveVoice {
            pitch: equal_tempered_tuning(note_index),
            velocity,
            elapsed_samples: 0,
            elapsed_beats: 0.0,
            silent_samples: 0,
            start_trigger: false,
            release_trigger: false,
            static_data,
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
