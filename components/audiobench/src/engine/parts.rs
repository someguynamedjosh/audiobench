use super::static_controls::Staticon;
use crate::registry::module_template::ModuleTemplate;
use shared_util::prelude::*;
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug)]
pub struct AutomationLane {
    pub connection: (Rcrc<Module>, usize),
    pub range: (f32, f32),
}

#[derive(Clone, Debug)]
pub struct Autocon {
    pub code_name: String,
    pub range: (f32, f32),
    pub default: f32,
    pub value: f32,
    pub automation: Vec<AutomationLane>,
    pub suffix: String,
}

impl Autocon {
    pub fn create(code_name: String, min: f32, max: f32, default: f32, suffix: String) -> Self {
        Self {
            code_name,
            range: (min, max),
            default,
            value: default,
            automation: Vec::new(),
            suffix,
        }
    }

    pub fn sever_connections_with(&mut self, module: &Rcrc<Module>) {
        for index in (0..self.automation.len()).rev() {
            if std::ptr::eq(
                self.automation[index].connection.0.as_ref(),
                module.as_ref(),
            ) {
                self.automation.remove(index);
            }
        }
    }
}

#[derive(Clone, Debug)]
pub enum InputConnection {
    Wire(Rcrc<Module>, usize),
    Default(usize),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JackType {
    Pitch,
    Waveform,
    Audio,
    Trigger,
}

struct DefaultInputDescription {
    name: &'static str,
    code: &'static str,
    icon: &'static str,
}

#[derive(Clone, Debug)]
pub struct DefaultInput {
    pub name: &'static str,
    pub code: &'static str,
    pub icon: usize,
}

impl JackType {
    pub fn from_str(input: &str) -> Result<Self, ()> {
        match input {
            "pitch" => Ok(Self::Pitch),
            "waveform" => Ok(Self::Waveform),
            "audio" => Ok(Self::Audio),
            "trigger" => Ok(Self::Trigger),
            _ => Err(()),
        }
    }

    pub fn icon_name(&self) -> &'static str {
        match self {
            Self::Pitch => "Factory:pitch",
            Self::Waveform => "Factory:waveform",
            Self::Audio => "Factory:audio",
            Self::Trigger => "Factory:trigger",
        }
    }

    pub fn ns_type(&self) -> &'static str {
        match self {
            Self::Pitch => "<STEREO_BUFFER>",
            Self::Waveform => "MACRO",
            Self::Audio => "<STEREO_BUFFER>",
            Self::Trigger => "<TRIGGER_BUFFER>",
        }
    }

    fn default_option_descriptions(&self) -> &'static [DefaultInputDescription] {
        match self {
            Self::Pitch => &[DefaultInputDescription {
                name: "Note Pitch",
                code: "global_pitch",
                icon: "Factory:note",
            }],
            Self::Waveform => &[
                DefaultInputDescription {
                    name: "Silence",
                    code: "FlatWaveform",
                    // TODO: Better icon.
                    icon: "Factory:nothing",
                },
                DefaultInputDescription {
                    name: "Ramp Up",
                    code: "RampUpWaveform",
                    icon: "Factory:ramp_up",
                },
                DefaultInputDescription {
                    name: "Ramp Down",
                    code: "RampDownWaveform",
                    icon: "Factory:ramp_down",
                },
                DefaultInputDescription {
                    name: "Sine Wave",
                    code: "SineWaveform",
                    icon: "Factory:sine_wave",
                },
            ],
            Self::Audio => &[DefaultInputDescription {
                name: "Silence",
                code: "0.0",
                icon: "Factory:nothing",
            }],
            Self::Trigger => &[
                DefaultInputDescription {
                    name: "Note Start",
                    code: "global_start_trigger",
                    icon: "Factory:note_down",
                },
                DefaultInputDescription {
                    name: "Note Release",
                    code: "global_release_trigger",
                    icon: "Factory:note_up",
                },
                DefaultInputDescription {
                    name: "Never",
                    code: "FALSE",
                    icon: "Factory:nothing",
                },
            ],
        }
    }

    fn default_options(&self, icon_indexes: &HashMap<String, usize>) -> Vec<DefaultInput> {
        self.default_option_descriptions()
            .iter()
            .map(|desc| DefaultInput {
                name: desc.name,
                code: desc.code,
                // The factory library should have all the listed icons.
                icon: *icon_indexes.get(desc.icon).unwrap(),
            })
            .collect()
    }

    pub fn get_num_defaults(&self) -> usize {
        self.default_option_descriptions().len()
    }
}

#[derive(Clone, Debug)]
pub struct IOJack {
    typ: JackType,
    icon_index: usize,
    custom_icon_index: Option<usize>,
    code_name: String,
    label: String,
    tooltip: String,
    default_options: Vec<DefaultInput>,
}

impl IOJack {
    pub fn create(
        icon_indexes: &HashMap<String, usize>,
        typ: JackType,
        icon_index: usize,
        custom_icon_index: Option<usize>,
        code_name: String,
        label: String,
        tooltip: String,
    ) -> Self {
        Self {
            typ,
            icon_index,
            custom_icon_index,
            code_name,
            label,
            tooltip,
            default_options: typ.default_options(icon_indexes),
        }
    }

    pub fn get_type(&self) -> JackType {
        self.typ
    }

    pub fn get_icon_index(&self) -> usize {
        self.icon_index
    }

    pub fn get_custom_icon_index(&self) -> Option<usize> {
        self.custom_icon_index
    }

    pub fn borrow_label(&self) -> &str {
        &self.label
    }

    pub fn borrow_code_name(&self) -> &str {
        &self.code_name
    }

    pub fn borrow_tooltip(&self) -> &str {
        &self.tooltip
    }

    pub fn borrow_default_options(&self) -> &[DefaultInput] {
        &self.default_options[..]
    }
}

#[derive(Debug)]
pub struct Module {
    pub template: Rcrc<ModuleTemplate>,
    pub autocons: Vec<Rcrc<Autocon>>,
    pub staticons: Vec<Rcrc<Staticon>>,
    pub pos: (f32, f32),
    pub inputs: Vec<InputConnection>,
    pub feedback_data: Option<Rcrc<Vec<f32>>>,
}

impl Clone for Module {
    fn clone(&self) -> Self {
        // gui_outline should point to the same data, but controls should point to unique copies
        // of the controls.
        Self {
            template: Rc::clone(&self.template),
            autocons: self
                .autocons
                .imc(|control_ref| rcrc((*control_ref.borrow()).clone())),
            staticons: self
                .staticons
                .imc(|control_ref| rcrc((*control_ref.borrow()).clone())),
            pos: self.pos,
            inputs: self.inputs.clone(),
            feedback_data: None,
        }
    }
}

impl Module {
    pub fn create(
        template: Rcrc<ModuleTemplate>,
        autocons: Vec<Rcrc<Autocon>>,
        staticons: Vec<Rcrc<Staticon>>,
        default_inputs: Vec<usize>,
    ) -> Self {
        Self {
            template,
            autocons,
            staticons,
            pos: (0.0, 0.0),
            inputs: default_inputs
                .into_iter()
                .map(|i| InputConnection::Default(i))
                .collect(),
            feedback_data: None,
        }
    }

    /// Removes all inputs and controls. Use this before removing a module to avoid memory leaks.
    /// It is still required to manually remove references to this module that exist in other
    /// modules.
    pub fn sever(&mut self) {
        self.inputs.clear();
        self.autocons.clear();
        self.staticons.clear();
        self.feedback_data = None;
    }

    pub fn sever_connections_with(&mut self, other: &Rcrc<Module>) {
        let template_ref = self.template.borrow();
        for (index, input) in self.inputs.iter_mut().enumerate() {
            if let InputConnection::Wire(module, ..) = input {
                if std::ptr::eq(module.as_ref(), other.as_ref()) {
                    *input = InputConnection::Default(template_ref.default_inputs[index]);
                }
            }
        }
        for control in &mut self.autocons {
            control.borrow_mut().sever_connections_with(other);
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

    pub fn set_modules(&mut self, modules: Vec<Rcrc<Module>>) {
        for module in &self.modules {
            module.borrow_mut().sever();
        }
        self.modules = modules;
    }

    pub fn remove_module(&mut self, module: &Rcrc<Module>) {
        let index = self
            .modules
            .iter()
            .position(|e| std::ptr::eq(e.as_ref(), module.as_ref()))
            .unwrap();
        let module_rc = Rc::clone(&self.modules[index]);
        for module in &self.modules {
            module.borrow_mut().sever_connections_with(&module_rc);
        }
        self.modules.remove(index);
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
            for control in &module_ref.autocons {
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
