use crate::{
    engine::{controls::Control, parts::Module},
    gui::top_level::graph::ModuleGraph,
};
use shared_util::prelude::*;
use std::fmt::{Display, Formatter};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

#[derive(Clone, Debug, Default)]
pub struct FeedbackData {
    pub widget_feeback: Vec<Vec<f32>>,
    pub output_view: Vec<Vec<f32>>,
    pub output_view_module_index: usize,
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
pub struct FeedbackDisplayer {
    /// Module the widget is in and the index of the widget in that module.
    widget_selectors: Vec<(Rcrc<Module>, usize)>,
}

impl FeedbackDisplayer {
    pub fn display(&self, data: FeedbackData, on: Rc<ModuleGraph>) {
        if data.widget_feeback.len() != self.widget_selectors.len() {
            return;
        }
        for (index, (module, widget_index)) in self.widget_selectors.iter().enumerate() {
            let module_widget = on.get_widget_for_module(module).unwrap();
            module_widget.take_feedback_data(data.widget_feeback[index].clone(), *widget_index);
        }
        let real_graph_ptr: Rcrc<crate::engine::parts::ModuleGraph> = on.get_real_graph();
        let real_graph = real_graph_ptr.borrow();
        if data.output_view_module_index < real_graph.borrow_modules().len() {
            let module = &real_graph.borrow_modules()[data.output_view_module_index];
            let module_widget = on.get_widget_for_module(module).unwrap();
            module_widget.take_output_view_data(data.output_view);
        }
    }
}
