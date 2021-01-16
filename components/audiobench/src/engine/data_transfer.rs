use jlrs_derive::IntoJulia;
use julia_helper::{DataType, Frame, JlrsResult, JuliaStruct, Value};
use shared_util::prelude::*;
use std::fmt::{Display, Formatter};

use super::controls::Control;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GlobalParameters {
    pub channels: usize,
    pub buffer_length: usize,
    pub sample_rate: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DataFormat {
    pub global_params: GlobalParameters,
    pub dyn_data_types: Vec<()>, // Previously IOType
    pub feedback_data_len: usize,
}

#[derive(Clone, PartialEq)]
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
#[derive(Clone, Copy, Debug)]
pub struct NoteData {
    pub pitch: f32,
    pub velocity: f32,
    pub elapsed_samples: usize,
    pub elapsed_beats: f32,
    pub start_trigger: bool,
    pub release_trigger: bool,
}

/// Represents the data type of a variable which is either an input or output in the generated
/// program. E.G. `IOType::FloatArray(20)` would be the type of `input [20]FLOAT some_data;`.
#[derive(Eq, PartialEq, Clone, Copy)]
pub enum IOType {
    Bool,
    Int,
    Float,
    BoolArray,
    IntArray,
    FloatArray,
}

impl Display for IOType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use IOType::*;
        match self {
            Bool => write!(f, "Bool"),
            Int => write!(f, "Int32"),
            Float => write!(f, "Float32"),
            BoolArray => write!(f, "Vector{{Bool}}"),
            IntArray => write!(f, "Vector{{Int32}}"),
            FloatArray => write!(f, "Vector{{Float32}}"),
        }
    }
}

#[derive(PartialEq, Debug)]
pub enum IOData {
    Bool(bool),
    Int(i32),
    Float(f32),
    BoolArray(Vec<bool>),
    IntArray(Vec<i32>),
    FloatArray(Vec<f32>),
}

impl IOData {
    pub fn as_julia_value<'f>(&self, frame: &mut impl Frame<'f>) -> JlrsResult<Value<'f, 'f>> {
        use IOData::*;
        match self {
            Bool(v) => Value::new(frame, *v),
            Int(v) => Value::new(frame, *v),
            Float(v) => Value::new(frame, *v),
            BoolArray(v) => Value::move_array(frame, v.clone(), (v.len(),)),
            IntArray(v) => Value::move_array(frame, v.clone(), (v.len(),)),
            FloatArray(v) => Value::move_array(frame, v.clone(), (v.len(),)),
        }
    }
}

#[scones::make_constructor]
pub struct DynDataCollector {
    controls: Vec<Rcrc<dyn Control>>,
}

impl DynDataCollector {
    pub fn collect(&self) -> Vec<IOData> {
        let mut result = Vec::new();
        for control in &self.controls {
            result.append(&mut control.borrow().get_parameter_values());
        }
        result
    }
}

#[scones::make_constructor]
pub struct FeedbackDisplayer {}
