use crate::gui::constants::*;
use crate::gui::widgets;
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
pub struct IOTab {
    pub connection: Option<(Rcrc<Module>, usize)>,
    code_name: String,
}

impl IOTab {
    pub fn create(code_name: String) -> Self {
        Self {
            connection: None,
            code_name,
        }
    }
}

#[derive(Debug)]
pub struct Module {
    pub gui_outline: Rcrc<GuiOutline>,
    pub controls: Vec<Rcrc<Control>>,
    pub pos: (i32, i32),
    pub inputs: Vec<IOTab>,
    pub outputs: Vec<IOTab>,
    pub internal_id: String,
    pub code_resource: String,
}

impl Clone for Module {
    fn clone(&self) -> Self {
        // gui_outline should point to the same data, but controls should point to unique copies
        // of the controls.
        Self {
            gui_outline: Rc::clone(&self.gui_outline),
            controls: self
                .controls
                .iter()
                .map(|control_ref| rcrc((*control_ref.borrow()).clone()))
                .collect(),
            pos: self.pos,
            inputs: self.inputs.clone(),
            outputs: self.outputs.clone(),
            internal_id: self.internal_id.clone(),
            code_resource: self.code_resource.clone(),
        }
    }
}

impl Module {
    pub fn create(
        gui_outline: Rcrc<GuiOutline>,
        controls: Vec<Rcrc<Control>>,
        inputs: Vec<IOTab>,
        outputs: Vec<IOTab>,
        internal_id: String,
        code_resource: String,
    ) -> Self {
        Self {
            gui_outline,
            controls,
            pos: (0, 0),
            inputs,
            outputs,
            internal_id,
            code_resource,
        }
    }

    fn instantiate_widget(
        &self,
        outline: &WidgetOutline,
    ) -> widgets::Knob {
        fn convert_grid_pos(grid_pos: &(i32, i32)) -> (i32, i32) {
            (MODULE_IO_WIDTH + coord(grid_pos.0), coord(grid_pos.1))
        }
        match outline {
            WidgetOutline::Knob {
                control_index,
                grid_pos,
                label,
            } => widgets::Knob::create(
                Rc::clone(&self.controls[*control_index]),
                convert_grid_pos(grid_pos),
                label.clone(),
            ),
        }
    }

    pub fn build_gui(self_rcrc: Rcrc<Self>) -> widgets::Module {
        let self_ref = self_rcrc.borrow();
        let outline = self_ref.gui_outline.borrow();
        let label = outline.label.clone();
        let size = outline.size.clone();
        let control_widgets = outline
            .widget_outlines
            .iter()
            .map(|wo| self_ref.instantiate_widget(wo))
            .collect();
        drop(outline);
        drop(self_ref);

        widgets::Module::create(self_rcrc, size, label, control_widgets)
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

    pub fn build_gui(self_rcrc: Rcrc<Self>) -> widgets::ModuleGraph {
        let self_ref = self_rcrc.borrow();
        let module_widgets = self_ref
            .modules
            .iter()
            .map(|module| Module::build_gui(Rc::clone(module)))
            .collect();
        widgets::ModuleGraph::create(module_widgets)
    }

    fn index_of_module(&self, module: &Rcrc<Module>) -> Option<usize> {
        self.modules
            .iter()
            .position(|other| Rc::ptr_eq(module, other))
    }

    fn compute_execution_order(&self) -> Result<Vec<usize>, ()> {
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
                if let Some((module_ref, _)) = &input.connection {
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

    fn generate_code_for_control(&self, control: &Rcrc<Control>) -> String {
        let control_ref = control.borrow();
        if control_ref.automation.len() == 0 {
            format!("{:01.1}", control_ref.value)
        } else {
            // TODO:
            let mut code = "0.0".to_owned();
            code
        }
    }

    fn generate_code_for_input(&self, input: &IOTab) -> String {
        if let Some((module, output_index)) = &input.connection {
            format!(
                "module_{}_output_{}",
                self.index_of_module(&module).unwrap_or(999999),
                output_index
            )
        } else {
            "0.0".to_owned()
        }
    }

    pub fn generate_code(&self, samples_per_channel: i32) -> Result<String, ()> {
        let mut code = "".to_owned();
        code.push_str(&format!(
            "INT SAMPLES_PER_CHANNEL = {};\n",
            samples_per_channel
        ));
        code.push_str(concat!(
            "input FLOAT global_pitch, global_velocity;\n",
            "output [SAMPLES_PER_CHANNEL][2]FLOAT global_audio_out;\n",
            "\n",
        ));

        for (index, module) in self.modules.iter().enumerate() {
            code.push_str(&format!("macro module_{}(\n", index));
            let module_ref = module.borrow();
            for input in module_ref.inputs.iter() {
                code.push_str(&format!("    {}, \n", input.code_name));
            }
            for control in module_ref.controls.iter() {
                let control_ref = control.borrow();
                code.push_str(&format!("    {}, \n", control_ref.code_name));
            }
            code.push_str("):(\n");
            for output in module_ref.outputs.iter() {
                code.push_str(&format!("    {}, \n", output.code_name));
            }
            code.push_str(") {\n");
            code.push_str(&format!("    include \"{}\";\n", module_ref.code_resource));
            code.push_str("}\n\n");
        }

        let execution_order = self.compute_execution_order()?;
        for index in execution_order {
            let module_ref = self.modules[index].borrow();
            code.push_str(&format!("module_{}(\n", index));
            for input in &module_ref.inputs {
                code.push_str(&format!(
                    "    {}, // {}\n",
                    self.generate_code_for_input(input),
                    &input.code_name
                ));
            }
            for control in &module_ref.controls {
                code.push_str(&format!(
                    "    {}, // {}\n",
                    self.generate_code_for_control(control),
                    &control.borrow().code_name
                ));
            }
            code.push_str("):(\n");
            for output_index in 0..module_ref.outputs.len() {
                code.push_str(&format!(
                    "    AUTO module_{}_output_{},\n",
                    index, output_index,
                ));
            }
            code.push_str(");\n\n");
        }

        Ok(code)
    }
}

#[derive(Debug)]
pub struct GuiOutline {
    pub label: String,
    pub size: (i32, i32),
    pub widget_outlines: Vec<WidgetOutline>,
}

#[derive(Debug)]
pub enum WidgetOutline {
    Knob {
        control_index: usize,
        grid_pos: (i32, i32),
        label: String,
    },
}
