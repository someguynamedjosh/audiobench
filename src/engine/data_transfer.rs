use crate::engine::data_format::{DataPacker, DataUnpacker, IODataPtr, IOType, OwnedIOData};

#[derive(Clone, PartialEq, Eq)]
pub struct HostFormat {
    pub sample_rate: usize,
    pub buffer_len: usize,
}

#[derive(Clone, PartialEq, Eq)]
pub struct DataFormat {
    pub host_format: HostFormat,
    pub autocon_dyn_data_len: usize,
    pub staticon_dyn_data_types: Vec<IOType>,
    pub feedback_data_len: usize,
}

pub struct HostData {
    // MIDI specifies each MIDI Channel has 128 controls.
    pub controller_values: [f32; 128],
    // The pitch wheel is seperate from other controls due to its higher precision.
    pub pitch_wheel_value: f32,
    pub bpm: f32,
    pub song_time: f32,
    pub song_beats: f32,
}

impl HostData {
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

pub struct NoteData {
    pub pitch: f32,
    pub velocity: f32,
    pub elapsed_samples: usize,
    pub elapsed_beats: f32,
    pub start_trigger: bool,
    pub release_trigger: bool,
}

pub struct InputPacker {
    real_packer: DataPacker,
    data_format: DataFormat,
}

impl InputPacker {
    // If the pitch wheel is within the deadzone, it will read as zero instead of its actual value.
    // I added this because my pitch wheel is utter crap.
    const PITCH_WHEEL_DEADZONE: f32 = 0.1;
    // The extra division makes it so the ends of the pitch wheel still reach the specified value
    // considering the deadzone.
    // Range: +- perfect fifth, todo: make adjustable
    const PITCH_WHEEL_RANGE: f32 = (7.0 / 12.0) / (1.0 - Self::PITCH_WHEEL_DEADZONE);

    const GPI_PITCH: usize = 0;
    const GPI_VELOCITY: usize = 1;
    const GPI_NOTE_STATUS: usize = 2;
    const GPI_SHOULD_UPDATE: usize = 3;
    const GPI_BPM: usize = 4;
    const GPI_NOTE_TIME: usize = 5;
    const GPI_NOTE_BEATS: usize = 6;
    const GPI_SONG_TIME: usize = 7;
    const GPI_SONG_BEATS: usize = 8;
    const GPI_MIDI_CONTROLS: usize = 9;
    const GPI_AUTOCON_DYN_DATA: usize = 10;
    const GPI_STATICON_DYN_DATA_START: usize = 11;
    fn make_data_packer(data_format: &DataFormat) -> DataPacker {
        let buf_len = data_format.host_format.buffer_len;
        let mut parameters = vec![
            IOType::Float,                                        // global_pitch
            IOType::Float,                                        // global_velocity
            IOType::Float,                                        // global_note_status
            IOType::Float,                                        // global_should_update
            IOType::Float,                                        // global_bpm
            IOType::FloatArray(buf_len),                          // global_note_time
            IOType::FloatArray(buf_len),                          // global_note_beats
            IOType::FloatArray(buf_len),                          // global_song_time
            IOType::FloatArray(buf_len),                          // global_song_beats
            IOType::FloatArray(128),                              // global_midi_controls
            IOType::FloatArray(data_format.autocon_dyn_data_len), // global_autocon_dyn_data
        ];
        parameters.append(&mut data_format.staticon_dyn_data_types.clone());
        DataPacker::new(parameters)
    }

    pub fn new(data_format: DataFormat) -> Self {
        Self {
            real_packer: Self::make_data_packer(&data_format),
            data_format,
        }
    }

    pub fn set_data_format(&mut self, format: DataFormat) {
        self.data_format = format;
        self.real_packer = Self::make_data_packer(&self.data_format);
    }

    pub fn borrow_data_format(&self) -> &DataFormat {
        &self.data_format
    }

    pub fn borrow_packed_data_mut(&mut self) -> &mut [u8] {
        self.real_packer.borrow_packed_data()
    }

    fn set_timing_input(&mut self, index: usize, start: f32, increment: f32) {
        let data: Vec<_> = (0..self.data_format.host_format.buffer_len)
            .map(|index| start + increment * index as f32)
            .collect();
        self.real_packer
            .set_parameter(index, IODataPtr::FloatArray(&data[..]));
    }

    pub fn set_host_data(&mut self, host_data: &HostData) {
        self.real_packer
            .set_parameter(Self::GPI_BPM, IODataPtr::Float(host_data.bpm));
        self.set_timing_input(
            Self::GPI_SONG_TIME,
            host_data.song_time,
            1.0 / self.data_format.host_format.sample_rate as f32,
        );
        self.set_timing_input(
            Self::GPI_SONG_BEATS,
            host_data.song_beats,
            host_data.bpm / 60.0 / self.data_format.host_format.sample_rate as f32,
        );
        self.real_packer.set_parameter(
            Self::GPI_MIDI_CONTROLS,
            IODataPtr::FloatArray(&host_data.controller_values[..]),
        );
    }

    pub fn set_note_data(
        &mut self,
        note_data: &NoteData,
        host_data: &HostData,
        update_feedback: bool,
    ) {
        // Pitch wheel value goes from -1.0 to 1.0. At the extreme ends, pitch should be offset by
        // a nice ratio. In the middle, there should be a deadzone where nothing happens. There
        // should be no sudden transition when leaving the deadzone. This math makes all these
        // conditions true.
        let pitch_offset: f32 = if host_data.pitch_wheel_value.abs() <= Self::PITCH_WHEEL_DEADZONE {
            1.0
        } else {
            // Make sure to offset so there is no sudden transition.
            let wheel_offset = if host_data.pitch_wheel_value > 0.0 {
                Self::PITCH_WHEEL_DEADZONE
            } else {
                -Self::PITCH_WHEEL_DEADZONE
            };
            2.0f32.powf((host_data.pitch_wheel_value - wheel_offset) * Self::PITCH_WHEEL_RANGE)
        };
        self.real_packer.set_parameter(
            Self::GPI_PITCH,
            IODataPtr::Float(note_data.pitch * pitch_offset),
        );
        self.real_packer
            .set_parameter(Self::GPI_VELOCITY, IODataPtr::Float(note_data.velocity));
        self.real_packer.set_parameter(
            Self::GPI_NOTE_STATUS,
            IODataPtr::Float(if note_data.start_trigger {
                2.0
            } else if note_data.release_trigger {
                1.0
            } else {
                0.0
            }),
        );
        self.real_packer.set_parameter(
            Self::GPI_SHOULD_UPDATE,
            IODataPtr::Float(if update_feedback { 1.0 } else { 0.0 }),
        );
        let sample_rate = self.data_format.host_format.sample_rate as f32;
        let elapsed_seconds = note_data.elapsed_samples as f32 / sample_rate;
        self.set_timing_input(Self::GPI_NOTE_TIME, elapsed_seconds, 1.0 / sample_rate);
        self.set_timing_input(
            Self::GPI_NOTE_BEATS,
            note_data.elapsed_beats,
            host_data.bpm / 60.0 / sample_rate,
        );
    }

    pub fn set_autocon_dyn_data(&mut self, data: &[f32]) {
        self.real_packer
            .set_parameter(Self::GPI_AUTOCON_DYN_DATA, IODataPtr::FloatArray(data));
    }

    pub fn set_staticon_dyn_data(&mut self, data: &[OwnedIOData]) {
        assert!(self.data_format.staticon_dyn_data_types.len() == data.len());
        for (index, item) in data.iter().enumerate() {
            let item_ptr = item.borrow();
            self.real_packer
                .set_parameter(Self::GPI_STATICON_DYN_DATA_START + index, item_ptr);
        }
    }
}

pub struct OutputUnpacker {
    real_unpacker: DataUnpacker,
    data_format: DataFormat,
}

impl OutputUnpacker {
    const GPI_AUDIO_OUT: usize = 0;
    const GPI_FEEDBACK_DATA: usize = 1;
    fn make_data_unpacker(data_format: &DataFormat) -> DataUnpacker {
        let buf_len = data_format.host_format.buffer_len;
        eprintln!("{}", data_format.feedback_data_len);
        let parameters = vec![
            IOType::FloatArray(buf_len * 2),                   // global_audio_out
            IOType::FloatArray(data_format.feedback_data_len), // global_feedback_data
        ];
        DataUnpacker::new(parameters)
    }

    pub fn new(data_format: DataFormat) -> Self {
        Self {
            real_unpacker: Self::make_data_unpacker(&data_format),
            data_format,
        }
    }

    pub fn set_data_format(&mut self, format: DataFormat) {
        self.data_format = format;
        self.real_unpacker = Self::make_data_unpacker(&self.data_format);
    }

    pub fn borrow_data_format(&self) -> &DataFormat {
        &self.data_format
    }

    pub fn borrow_packed_data_mut(&mut self) -> &mut [u8] {
        self.real_unpacker.borrow_packed_data()
    }

    pub fn borrow_audio_out(&self) -> &[f32] {
        unsafe {
            if let IODataPtr::FloatArray(arr) =
                self.real_unpacker.get_parameter(Self::GPI_AUDIO_OUT)
            {
                arr
            } else {
                unreachable!()
            }
        }
    }

    pub fn borrow_feedback_data(&self) -> &[f32] {
        unsafe {
            if let IODataPtr::FloatArray(arr) =
                self.real_unpacker.get_parameter(Self::GPI_FEEDBACK_DATA)
            {
                arr
            } else {
                unreachable!()
            }
        }
    }
}
