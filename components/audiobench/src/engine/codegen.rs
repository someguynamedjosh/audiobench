use shared_util::prelude::*;

use crate::{
    engine::{
        controls::AutomationSource,
        data_transfer::{DataFormat, DynDataCollector, FeedbackDisplayer, GlobalParameters},
        parts::*,
    },
    gui::module_widgets::FeedbackMode,
    registry::Registry,
};

pub struct GeneratedCode {}

pub(super) struct CodeGenResult {
    pub code: GeneratedCode,
    pub dyn_data_collector: DynDataCollector,
    pub feedback_displayer: FeedbackDisplayer,
    pub data_format: DataFormat,
}

pub struct AutomationCode {
    ordered_modules: Vec<Rcrc<Module>>,
}

impl AutomationCode {
    pub fn value_of(&self, source: &AutomationSource) -> String {
        let module_index = self
            .ordered_modules
            .iter()
            .position(|mod_ptr| Rc::ptr_eq(mod_ptr, &source.module))
            .unwrap(); // Our list should contain all the modules that exist.
        format!("m{}o{}", module_index, source.output_index)
    }
}

struct CodeGenerator<'a> {
    graph: &'a ModuleGraph,
    execution_order: Vec<usize>,
    dyn_data_types: Vec<()>, // Previously IOType
    dyn_data_parameter_defs: Vec<String>,
    feedback_data_len: usize,
}

pub(super) fn generate_code(
    for_graph: &ModuleGraph,
    global_params: &GlobalParameters,
) -> Result<CodeGenResult, ()> {
    todo!()
}
