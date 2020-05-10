use crate::gui::constants::*;
use crate::gui::widgets;
use crate::util::*;

#[derive(Clone, Debug)]
pub struct AutomationLane {
    pub source: (Rcrc<Module>, usize),
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

#[derive(Debug)]
pub struct Module {
    gui_outline: Rcrc<GuiOutline>,
    controls: Vec<Rcrc<Control>>,
    pub pos: (i32, i32),
    pub num_inputs: usize,
    pub num_outputs: usize,
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
            num_inputs: self.num_inputs,
            num_outputs: self.num_outputs,
            internal_id: self.internal_id.clone(),
            code_resource: self.code_resource.clone(),
        }
    }
}

impl Module {
    pub fn create(
        gui_outline: Rcrc<GuiOutline>,
        controls: Vec<Rcrc<Control>>,
        num_inputs: usize,
        num_outputs: usize,
        internal_id: String,
        code_resource: String,
    ) -> Self {
        Self {
            gui_outline,
            controls,
            pos: (0, 0),
            num_inputs,
            num_outputs,
            internal_id,
            code_resource,
        }
    }

    fn instantiate_widget(&self, outline: &WidgetOutline) -> Rcrc<dyn widgets::Widget> {
        fn convert_grid_pos(grid_pos: &(i32, i32)) -> (i32, i32) {
            (MODULE_IO_WIDTH + coord(grid_pos.0), coord(grid_pos.1))
        }
        match outline {
            WidgetOutline::Knob {
                control_index,
                grid_pos,
                label,
            } => rcrc(widgets::Knob::create(
                Rc::clone(&self.controls[*control_index]),
                convert_grid_pos(grid_pos),
                label.clone(),
            )),
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
            .map(|module| rcrc(Module::build_gui(Rc::clone(module))) as Rcrc<dyn widgets::Widget>)
            .collect();
        widgets::ModuleGraph::create(module_widgets)
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
