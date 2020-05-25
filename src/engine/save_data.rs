use crate::engine::parts as ep;
use crate::util::*;
use std::collections::HashMap;

#[derive(Debug)]
struct SavedAutomationLane {
    module_index: usize,
    output_index: usize,
    range: (f32, f32),
}

#[derive(Debug)]
struct SavedControl {
    value: f32,
    automation_lanes: Vec<SavedAutomationLane>,
}

#[derive(Debug)]
struct SavedComplexControl {
    value: String,
}

#[derive(Debug)]
enum SavedInputConnection {
    Default(usize),
    Output {
        module_index: usize,
        output_index: usize,
    },
}

#[derive(Debug)]
struct SavedModule {
    resource_name: String,
    controls: Vec<SavedControl>,
    complex_controls: Vec<SavedComplexControl>,
    input_connections: Vec<SavedInputConnection>,
    pos: (i32, i32),
}

#[derive(Debug)]
pub struct SavedModuleGraph {
    modules: Vec<SavedModule>,
}

impl SavedModuleGraph {
    fn save_control(
        control: &Rcrc<ep::Control>,
        module_indexes: &HashMap<*const RefCell<ep::Module>, usize>,
    ) -> SavedControl {
        let control_ref = control.borrow();
        let value = control_ref.value;
        let automation_lanes = control_ref
            .automation
            .iter()
            .map(|lane| {
                let module_index = *module_indexes
                    .get(&(&*lane.connection.0.as_ref() as *const _))
                    .unwrap();
                let output_index = lane.connection.1;
                let range = lane.range;
                SavedAutomationLane {
                    module_index,
                    output_index,
                    range,
                }
            })
            .collect();
        SavedControl {
            value,
            automation_lanes,
        }
    }

    fn save_input(
        input: &ep::InputConnection,
        module_indexes: &HashMap<*const RefCell<ep::Module>, usize>,
    ) -> SavedInputConnection {
        match input {
            ep::InputConnection::Wire(module, output_index) => SavedInputConnection::Output {
                module_index: *module_indexes
                    .get(&(&*module.as_ref() as *const _))
                    .unwrap(),
                output_index: *output_index,
            },
            ep::InputConnection::Default(default_index) => {
                SavedInputConnection::Default(*default_index)
            }
        }
    }

    fn save_module(
        module: &Rcrc<ep::Module>,
        module_indexes: &HashMap<*const RefCell<ep::Module>, usize>,
    ) -> SavedModule {
        let mod_ref = module.borrow();
        let template_ref = mod_ref.template.borrow();
        let resource_name = template_ref.resource_name.clone();
        let controls = mod_ref
            .controls
            .iter()
            .map(|control| Self::save_control(control, module_indexes))
            .collect();
        let complex_controls = mod_ref
            .complex_controls
            .iter()
            .map(|control| SavedComplexControl {
                value: control.borrow().value.clone(),
            })
            .collect();
        let input_connections = mod_ref
            .inputs
            .iter()
            .map(|input| Self::save_input(input, module_indexes))
            .collect();
        let pos = mod_ref.pos;
        SavedModule {
            resource_name,
            controls,
            complex_controls,
            input_connections,
            pos,
        }
    }

    pub fn save(graph: &ep::ModuleGraph) -> Self {
        let mut module_indexes: HashMap<*const RefCell<ep::Module>, usize> = HashMap::new();
        for (index, module) in graph.borrow_modules().iter().enumerate() {
            module_indexes.insert(&*module.as_ref(), index);
        }
        let module_indexes = module_indexes;
        let modules = graph
            .borrow_modules()
            .iter()
            .map(|module| Self::save_module(module, &module_indexes))
            .collect();
        Self { modules }
    }
}
