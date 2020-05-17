use crate::util::*;
use std::collections::HashSet;

#[derive(Clone, Debug)]
pub struct AutomationLane {
    pub connection: (Rcrc<Module>, usize),
    pub range: (f32, f32),
}

#[derive(Clone, Debug)]
pub struct Control {
    pub code_name: String,
    pub range: (f32, f32),
    pub value: f32,
    pub automation: Vec<AutomationLane>,
}

impl Control {
    pub fn create(code_name: String, min: f32, max: f32, default: f32) -> Self {
        Self {
            code_name,
            range: (min, max),
            value: default,
            automation: Vec::new(),
        }
    }
}

#[derive(Clone, Debug)]
pub enum InputConnection {
    Wire(Rcrc<Module>, usize),
    Default,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JackType {
    Time,
    Pitch,
    Waveform,
    Audio,
    Trigger,
}

impl JackType {
    pub fn from_str(input: &str) -> Result<Self, ()> {
        match input {
            "time" => Ok(Self::Time),
            "pitch" => Ok(Self::Pitch),
            "waveform" => Ok(Self::Waveform),
            "audio" => Ok(Self::Audio),
            "trigger" => Ok(Self::Trigger),
            _ => Err(()),
        }
    }

    pub fn icon_name(&self) -> &'static str {
        match self {
            Self::Time => "base:time",
            Self::Pitch => "base:pitch",
            Self::Waveform => "base:waveform",
            Self::Audio => "base:audio",
            Self::Trigger => "base:trigger",
        }
    }

    pub fn default_value(&self) -> &'static str {
        match self {
            Self::Time => "global_note_time",
            Self::Pitch => "global_pitch",
            Self::Waveform => "FlatWaveform",
            Self::Audio => "0.0",
            Self::Trigger => "global_release_trigger",
        }
    }
}

#[derive(Clone, Debug)]
pub struct IOJack {
    typ: JackType,
    icon_index: usize,
    code_name: String,
    label: String,
}

impl IOJack {
    pub fn create(typ: JackType, icon_index: usize, code_name: String, label: String) -> Self {
        Self {
            typ,
            icon_index,
            code_name,
            label,
        }
    }

    pub fn get_type(&self) -> JackType {
        self.typ
    }

    pub fn get_icon_index(&self) -> usize {
        self.icon_index
    }

    pub fn borrow_label(&self) -> &str {
        &self.label
    }

    pub fn borrow_code_name(&self) -> &str {
        &self.code_name
    }
}

#[derive(Debug)]
pub struct Module {
    pub template: Rcrc<ModuleTemplate>,
    pub controls: Vec<Rcrc<Control>>,
    pub pos: (i32, i32),
    pub inputs: Vec<InputConnection>,
    pub feedback_data: Option<Rcrc<Vec<f32>>>,
}

impl Clone for Module {
    fn clone(&self) -> Self {
        // gui_outline should point to the same data, but controls should point to unique copies
        // of the controls.
        Self {
            template: Rc::clone(&self.template),
            controls: self
                .controls
                .iter()
                .map(|control_ref| rcrc((*control_ref.borrow()).clone()))
                .collect(),
            pos: self.pos,
            inputs: self.inputs.clone(),
            feedback_data: None,
        }
    }
}

impl Module {
    pub fn create(template: Rcrc<ModuleTemplate>, controls: Vec<Rcrc<Control>>) -> Self {
        let num_inputs = template.borrow().inputs.len();
        Self {
            template,
            controls,
            pos: (0, 0),
            inputs: vec![InputConnection::Default; num_inputs],
            feedback_data: None,
        }
    }
}

#[derive(Debug)]
pub struct ModuleGraph {
    modules: Vec<Rcrc<Module>>,
}

impl ModuleGraph {
    pub fn new() -> Self {
        Self {
            modules: Vec::new(),
        }
    }

    pub fn add_module(&mut self, module: Rcrc<Module>) {
        self.modules.push(module);
    }

    pub fn adopt_module(&mut self, module: Module) {
        self.modules.push(rcrc(module));
    }

    pub fn borrow_modules(&self) -> &[Rcrc<Module>] {
        &self.modules[..]
    }

    pub fn index_of_module(&self, module: &Rcrc<Module>) -> Option<usize> {
        self.modules
            .iter()
            .position(|other| Rc::ptr_eq(module, other))
    }

    pub fn compute_execution_order(&self) -> Result<Vec<usize>, ()> {
        let mut execution_order = Vec::new();
        struct ModuleRepr {
            dependencies: Vec<usize>,
            satisfied: bool,
        }
        let mut module_reprs = Vec::new();
        for module in self.modules.iter() {
            let module_ref = module.borrow();
            let mut dependencies = HashSet::new();
            for input in &module_ref.inputs {
                if let InputConnection::Wire(module_ref, _) = &input {
                    dependencies.insert(self.index_of_module(module_ref).ok_or(())?);
                }
            }
            for control in &module_ref.controls {
                let control_ref = control.borrow();
                for lane in &control_ref.automation {
                    dependencies.insert(self.index_of_module(&lane.connection.0).ok_or(())?);
                }
            }
            let flat_dependencies = dependencies.iter().cloned().collect();
            module_reprs.push(ModuleRepr {
                dependencies: flat_dependencies,
                satisfied: false,
            });
        }
        let mut progress = true;
        while progress {
            progress = false;
            for index in 0..module_reprs.len() {
                if module_reprs[index].satisfied {
                    continue;
                }
                // Dependencies met if there is no dependency that is not satisfied.
                let dependencies_met = !module_reprs[index]
                    .dependencies
                    .iter()
                    .any(|depi| !module_reprs[*depi].satisfied);
                if dependencies_met {
                    execution_order.push(index);
                    module_reprs[index].satisfied = true;
                    progress = true;
                }
            }
        }
        if execution_order.len() == module_reprs.len() {
            Ok(execution_order)
        } else {
            Err(())
        }
    }
}

#[derive(Debug)]
pub struct ModuleTemplate {
    pub label: String,
    pub code_resource: String,
    pub size: (i32, i32),
    pub widget_outlines: Vec<WidgetOutline>,
    pub inputs: Vec<IOJack>,
    pub outputs: Vec<IOJack>,
    pub feedback_data_len: usize,
}

#[derive(Debug)]
pub enum WidgetOutline {
    Knob {
        control_index: usize,
        grid_pos: (i32, i32),
        label: String,
    },
}

pub enum FeedbackDataRequirement {
    None,
    Control { control_index: usize },
    Custom { code_name: String, size: usize },
}

impl FeedbackDataRequirement {
    pub fn size(&self) -> usize {
        match self {
            Self::None => 0,
            Self::Control { .. } => 1,
            Self::Custom { size, .. } => *size,
        }
    }
}

impl WidgetOutline {
    pub fn get_feedback_data_requirement(&self) -> FeedbackDataRequirement {
        match self {
            Self::Knob { control_index, .. } => FeedbackDataRequirement::Control {
                control_index: *control_index,
            },
        }
    }
}
