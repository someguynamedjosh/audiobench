use crate::gui::constants::*;
use crate::gui::audio_widgets;
use crate::util::*;
use std::collections::HashSet;

fn format_decimal(value: f32) -> String {
    let digits = 8i32;
    let digits = match value {
        v if v <= 0.0 => digits,
        _ => digits - (value.abs().log10().min(digits as f32 - 1.0) as i32),
    };
    let digits = digits as usize;
    format!("{:.*}", digits, value)
}

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
}

impl IOJack {
    pub fn create(typ: JackType, icon_index: usize, code_name: String) -> Self {
        Self {
            typ,
            icon_index,
            code_name,
        }
    }

    pub fn get_type(&self) -> JackType {
        self.typ
    }

    pub fn get_icon_index(&self) -> usize {
        self.icon_index
    }
}

#[derive(Debug)]
pub struct Module {
    pub gui_outline: Rcrc<GuiOutline>,
    pub controls: Vec<Rcrc<Control>>,
    pub pos: (i32, i32),
    pub inputs: Vec<InputConnection>,
    pub input_jacks: Vec<IOJack>,
    pub output_jacks: Vec<IOJack>,
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
            input_jacks: self.input_jacks.clone(),
            output_jacks: self.output_jacks.clone(),
            internal_id: self.internal_id.clone(),
            code_resource: self.code_resource.clone(),
        }
    }
}

impl Module {
    pub fn create(
        gui_outline: Rcrc<GuiOutline>,
        controls: Vec<Rcrc<Control>>,
        input_jacks: Vec<IOJack>,
        output_jacks: Vec<IOJack>,
        internal_id: String,
        code_resource: String,
    ) -> Self {
        Self {
            gui_outline,
            controls,
            pos: (0, 0),
            inputs: vec![InputConnection::Default; input_jacks.len()],
            input_jacks,
            output_jacks,
            internal_id,
            code_resource,
        }
    }

    fn instantiate_widget(&self, outline: &WidgetOutline) -> audio_widgets::Knob {
        fn convert_grid_pos(grid_pos: &(i32, i32)) -> (i32, i32) {
            (MODULE_IO_WIDTH + coord(grid_pos.0), coord(grid_pos.1))
        }
        match outline {
            WidgetOutline::Knob {
                control_index,
                grid_pos,
                label,
            } => audio_widgets::Knob::create(
                Rc::clone(&self.controls[*control_index]),
                convert_grid_pos(grid_pos),
                label.clone(),
            ),
        }
    }

    pub fn build_gui(self_rcrc: Rcrc<Self>) -> audio_widgets::Module {
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

        audio_widgets::Module::create(self_rcrc, size, label, control_widgets)
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

    pub fn build_gui(self_rcrc: Rcrc<Self>) -> audio_widgets::ModuleGraph {
        let self_ref = self_rcrc.borrow();
        let module_widgets = self_ref
            .modules
            .iter()
            .map(|module| Module::build_gui(Rc::clone(module)))
            .collect();
        drop(self_ref);
        audio_widgets::ModuleGraph::create(self_rcrc, module_widgets)
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

    fn generate_code_for_lane(&self, lane: &AutomationLane) -> String {
        let (target_min, target_max) = lane.range;
        // algebraic simplification of remapping value [-1, 1] -> [0, 1] -> [min, max]
        let a = (target_max - target_min) / 2.0;
        let b = a + target_min;
        let mod_index = self.index_of_module(&lane.connection.0).unwrap_or(3999999);
        format!(
            "module_{}_output_{} * {} + {}",
            mod_index,
            lane.connection.1,
            format_decimal(a),
            format_decimal(b)
        )
    }

    fn generate_code_for_control(&self, control: &Rcrc<Control>) -> String {
        let control_ref = control.borrow();
        if control_ref.automation.len() == 0 {
            format_decimal(control_ref.value)
        } else {
            let mut code = self.generate_code_for_lane(&control_ref.automation[0]);
            for lane in &control_ref.automation[1..] {
                code.push_str(" + ");
                code.push_str(&self.generate_code_for_lane(lane));
            }
            code
        }
    }

    fn generate_code_for_input(&self, input: &InputConnection, typ: JackType) -> String {
        match input {
            InputConnection::Wire(module, output_index) => format!(
                "module_{}_output_{}",
                self.index_of_module(&module).unwrap_or(2999999),
                output_index
            ),
            InputConnection::Default => typ.default_value().to_owned(),
        }
    }

    pub fn generate_code(&self, buffer_length: i32, sample_rate: i32) -> Result<String, ()> {
        let mut code = "".to_owned();
        code.push_str(&format!(
            "INT BUFFER_LENGTH = {};\n",
            buffer_length, 
        ));
        code.push_str(&format!(
            "FLOAT SAMPLE_RATE = {}.0;\n",
            sample_rate, 
        ));
        code.push_str(concat!(
            "input FLOAT global_pitch, global_velocity, global_note_status;\n",
            "input [BUFFER_LENGTH][1]FLOAT global_note_time;\n",
            "output [BUFFER_LENGTH][2]FLOAT global_audio_out;\n",
            "[BUFFER_LENGTH]BOOL global_release_trigger = FALSE;\n",
            "if global_note_status == 1.0 { global_release_trigger[0] = TRUE; }\n",
            "macro FlatWaveform(buffer_pos, phase):(value) { FLOAT value = 0.0; }\n",
            "\n",
        ));

        for (index, module) in self.modules.iter().enumerate() {
            code.push_str(&format!("macro module_{}(\n", index));
            let module_ref = module.borrow();
            for input in module_ref.input_jacks.iter() {
                code.push_str(&format!("    {}, \n", input.code_name));
            }
            for control in module_ref.controls.iter() {
                let control_ref = control.borrow();
                code.push_str(&format!("    {}, \n", control_ref.code_name));
            }
            code.push_str("):(\n");
            for output in module_ref.output_jacks.iter() {
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
            for (input, jack) in module_ref.inputs.iter().zip(module_ref.input_jacks.iter()) {
                code.push_str(&format!(
                    "    {}, // {}\n",
                    self.generate_code_for_input(input, jack.typ),
                    &jack.code_name
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
            for output_index in 0..module_ref.output_jacks.len() {
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
