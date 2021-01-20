use crate::{
    engine::controls::AnyControl,
    gui::top_level::graph::ModuleGraph as ModuleGraphWidget,
    registry::{module_template::ModuleTemplate, yaml::YamlNode},
};
use shared_util::prelude::*;
use std::collections::{HashMap, HashSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum JackType {
    Pitch,
    Waveform,
    Audio,
    Trigger,
}

impl JackType {
    pub fn from_yaml(yaml: &YamlNode) -> Result<Self, String> {
        let names = vec!["pitch", "waveform", "audio", "trigger"];
        let values = vec![Self::Pitch, Self::Waveform, Self::Audio, Self::Trigger];
        Ok(values[yaml.parse_enumerated(&names[..])?])
    }

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
}

#[derive(Clone, Debug)]
pub struct IOJack {
    typ: JackType,
    icon_index: usize,
    custom_icon_index: Option<usize>,
    code_name: String,
    label: String,
    tooltip: String,
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
}

#[derive(Debug)]
pub struct Module {
    pub template: Rcrc<ModuleTemplate>,
    pub controls: Vec<AnyControl>,
    pub pos: (f32, f32),
}

impl Module {
    pub fn create(template: Rcrc<ModuleTemplate>) -> Self {
        let controls = template
            .borrow()
            .default_controls
            .imc(|(_, c)| c.deep_clone());
        Self {
            template,
            controls,
            pos: (0.0, 0.0),
        }
    }

    /// Removes all inputs and controls. Use this before removing a module to avoid memory leaks.
    /// It is still required to manually remove references to this module that exist in other
    /// modules.
    pub fn sever(&mut self) {
        self.controls.clear();
    }

    pub fn sever_connections_with(&mut self, other: &Rcrc<Module>) {
        let template_ref = self.template.borrow();
        unimplemented!();
    }
}

pub struct ModuleGraph {
    modules: Vec<Rcrc<Module>>,
    pub current_widget: Option<Rc<ModuleGraphWidget>>,
}

impl ModuleGraph {
    pub fn new() -> Self {
        Self {
            modules: Vec::new(),
            current_widget: None,
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
            for control in &module_ref.controls {
                let ptr = control.as_dyn_ptr();
                let control_ref = ptr.borrow();
                for sauce in control_ref.get_connected_automation() {
                    dependencies.insert(self.index_of_module(&sauce.module).ok_or(())?);
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
