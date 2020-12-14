use jlrs_derive::IntoJulia;
use julia_helper::{DataType, Frame, JlrsResult, JuliaStruct, Value};

#[derive(Clone, PartialEq, Eq)]
pub struct GlobalParameters {
    pub channels: usize,
    pub buffer_length: usize,
    pub sample_rate: usize,
}

#[derive(Clone, PartialEq, Eq)]
pub struct DataFormat {
    pub global_params: GlobalParameters,
    pub autocon_dyn_data_len: usize,
    pub staticon_dyn_data_types: Vec<()>, // Previously IOType
    pub feedback_data_len: usize,
}

#[derive(Clone)]
pub struct GlobalData {
    // MIDI specifies each MIDI Channel has 128 controls.
    pub controller_values: [f32; 128],
    // The pitch wheel is seperate from other controls due to its higher precision.
    pub pitch_wheel: f32,
    pub bpm: f32,
    pub elapsed_time: f32,
    pub elapsed_beats: f32,
}

impl GlobalData {
    pub fn new() -> Self {
        Self {
            controller_values: [0.0; 128],
            pitch_wheel: 0.0,
            bpm: 120.0,
            elapsed_time: 0.0,
            elapsed_beats: 0.0,
        }
    }

    pub fn as_julia_values<'f>(
        &self,
        frame: &mut impl Frame<'f>,
    ) -> JlrsResult<Vec<Value<'f, 'f>>> {
        Ok(vec![
            Value::move_array(frame, self.controller_values.to_vec(), (128,))?,
            Value::new(frame, self.pitch_wheel)?,
            Value::new(frame, self.bpm)?,
            Value::new(frame, self.elapsed_time)?,
            Value::new(frame, self.elapsed_beats)?,
        ])
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct NoteData {
    pub pitch: f32,
    pub velocity: f32,
    pub elapsed_samples: usize,
    pub elapsed_beats: f32,
    pub start_trigger: bool,
    pub release_trigger: bool,
}

pub struct InputPacker<'a> {
    // real_packer: &'a mut DataPacker,
    data_format: &'a DataFormat,
}

impl<'a> InputPacker<'a> {
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
    const GPI_elapsed_time: usize = 7;
    const GPI_elapsed_beats: usize = 8;
    const GPI_MIDI_CONTROLS: usize = 9;
    const GPI_AUTOCON_DYN_DATA: usize = 10;
    const GPI_STATICON_DYN_DATA_START: usize = 11;

    pub fn new(data_format: &'a DataFormat) -> Self {
        Self {
            data_format,
        }
    }

    fn set_timing_input(&mut self, index: usize, start: f32, increment: f32) {
        let data: Vec<_> = (0..self.data_format.global_params.buffer_length)
            .map(|index| start + increment * index as f32)
            .collect();
        // self.real_packer
        //     .set_argument(index, IODataPtr::FloatArray(&data[..]));
    }

    pub fn set_global_data(&mut self, global_data: &GlobalData) {
        // self.real_packer
        //     .set_argument(Self::GPI_BPM, IODataPtr::Float(global_data.bpm));
        self.set_timing_input(
            Self::GPI_elapsed_time,
            global_data.elapsed_time,
            1.0 / self.data_format.global_params.sample_rate as f32,
        );
        self.set_timing_input(
            Self::GPI_elapsed_beats,
            global_data.elapsed_beats,
            global_data.bpm / 60.0 / self.data_format.global_params.sample_rate as f32,
        );
        // self.real_packer.set_argument(
        //     Self::GPI_MIDI_CONTROLS,
        //     IODataPtr::FloatArray(&global_data.controller_values[..]),
        // );
    }

    pub fn set_note_data(
        &mut self,
        note_data: &NoteData,
        global_data: &GlobalData,
        update_feedback: bool,
    ) {
        // Pitch wheel value goes from -1.0 to 1.0. At the extreme ends, pitch should be offset by
        // a nice ratio. In the middle, there should be a deadzone where nothing happens. There
        // should be no sudden transition when leaving the deadzone. This math makes all these
        // conditions true.
        let pitch_offset: f32 = if global_data.pitch_wheel.abs() <= Self::PITCH_WHEEL_DEADZONE {
            1.0
        } else {
            // Make sure to offset so there is no sudden transition.
            let wheel_offset = if global_data.pitch_wheel > 0.0 {
                Self::PITCH_WHEEL_DEADZONE
            } else {
                -Self::PITCH_WHEEL_DEADZONE
            };
            2.0f32.powf((global_data.pitch_wheel - wheel_offset) * Self::PITCH_WHEEL_RANGE)
        };
        // self.real_packer.set_argument(
        //     Self::GPI_PITCH,
        //     IODataPtr::Float(note_data.pitch * pitch_offset),
        // );
        // self.real_packer
        //     .set_argument(Self::GPI_VELOCITY, IODataPtr::Float(note_data.velocity));
        // self.real_packer.set_argument(
        //     Self::GPI_NOTE_STATUS,
        //     IODataPtr::Float(if note_data.start_trigger {
        //         2.0
        //     } else if note_data.release_trigger {
        //         1.0
        //     } else {
        //         0.0
        //     }),
        // );
        // self.real_packer.set_argument(
        //     Self::GPI_SHOULD_UPDATE,
        //     IODataPtr::Float(if update_feedback { 1.0 } else { 0.0 }),
        // );
        let sample_rate = self.data_format.global_params.sample_rate as f32;
        let elapsed_seconds = note_data.elapsed_samples as f32 / sample_rate;
        self.set_timing_input(Self::GPI_NOTE_TIME, elapsed_seconds, 1.0 / sample_rate);
        self.set_timing_input(
            Self::GPI_NOTE_BEATS,
            note_data.elapsed_beats,
            global_data.bpm / 60.0 / sample_rate,
        );
    }

    pub fn set_autocon_dyn_data(&mut self, data: &[f32]) {
        // self.real_packer
        //     .set_argument(Self::GPI_AUTOCON_DYN_DATA, IODataPtr::FloatArray(data));
    }

    pub fn set_staticon_dyn_data<T>(&mut self, data: &[T]) { // Previously OwnedIOData
        assert!(self.data_format.staticon_dyn_data_types.len() == data.len());
        for (index, item) in data.iter().enumerate() {
            // let item_ptr = item.borrow();
            // self.real_packer
            //     .set_argument(Self::GPI_STATICON_DYN_DATA_START + index, item_ptr);
        }
    }
}

pub struct OutputUnpacker {
    // real_unpacker: &'a DataUnpacker,
}

impl OutputUnpacker {
    const GPI_AUDIO_OUT: usize = 0;
    const GPI_FEEDBACK_DATA: usize = 1;

    pub fn new() -> Self {
        Self { }
    }

    pub fn borrow_audio_out(&self) -> &[f32] {
        unimplemented!()
    }

    pub fn borrow_feedback_data(&self) -> &[f32] {
        unimplemented!()
    }
}
