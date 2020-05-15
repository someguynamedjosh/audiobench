use crate::engine::registry::Registry;
use crate::engine::parts as ep;
use crate::gui::graphics::GrahpicsWrapper;
use crate::gui::{audio_widgets, other_widgets};
use crate::util::*;

pub struct MouseMods {
    pub right_click: bool,
}

pub enum MouseAction {
    None,
    ManipulateControl(Rcrc<ep::Control>),
    ManipulateLane(Rcrc<ep::Control>, usize),
    ManipulateLaneStart(Rcrc<ep::Control>, usize),
    ManipulateLaneEnd(Rcrc<ep::Control>, usize),
    MoveModule(Rcrc<ep::Module>),
    PanOffset(Rcrc<(i32, i32)>),
    ConnectInput(Rcrc<ep::Module>, usize),
    ConnectOutput(Rcrc<ep::Module>, usize),
    OpenMenu(Box<audio_widgets::KnobEditor>),
}

impl MouseAction {
    pub fn is_none(&self) -> bool {
        if let Self::None = self {
            true
        } else {
            false
        }
    }

    fn on_drag(&mut self, delta: (i32, i32)) {
        match self {
            Self::ManipulateControl(control) => {
                let delta = delta.0 - delta.1;
                // How many pixels the user must drag across to cover the entire range of the knob.
                const DRAG_PIXELS: f32 = 200.0;
                let delta = delta as f32 / DRAG_PIXELS;

                let mut control_ref = control.borrow_mut();
                let range = control_ref.range;
                let delta = delta * (range.1 - range.0) as f32;
                control_ref.value = (control_ref.value + delta).clam(range.0, range.1);
                for lane in &mut control_ref.automation {
                    lane.range.0 = (lane.range.0 + delta).clam(range.0, range.1);
                    lane.range.1 = (lane.range.1 + delta).clam(range.0, range.1);
                }
            }
            Self::ManipulateLane(control, lane_index) => {
                let delta = delta.0 - delta.1;
                // How many pixels the user must drag across to cover the entire range of the knob.
                const DRAG_PIXELS: f32 = 200.0;
                let delta = delta as f32 / DRAG_PIXELS;

                let mut control_ref = control.borrow_mut();
                let range = control_ref.range;
                let delta = delta * (range.1 - range.0) as f32;
                let lane = &mut control_ref.automation[*lane_index];
                lane.range.0 = (lane.range.0 + delta).clam(range.0, range.1);
                lane.range.1 = (lane.range.1 + delta).clam(range.0, range.1);
            }
            Self::ManipulateLaneStart(control, lane_index) => {
                let delta = delta.0 - delta.1;
                // How many pixels the user must drag across to cover the entire range of the knob.
                const DRAG_PIXELS: f32 = 200.0;
                let delta = delta as f32 / DRAG_PIXELS;

                let mut control_ref = control.borrow_mut();
                let range = control_ref.range;
                let delta = delta * (range.1 - range.0) as f32;
                let lane = &mut control_ref.automation[*lane_index];
                lane.range.0 = (lane.range.0 + delta).clam(range.0, range.1);
            }
            Self::ManipulateLaneEnd(control, lane_index) => {
                let delta = delta.0 - delta.1;
                // How many pixels the user must drag across to cover the entire range of the knob.
                const DRAG_PIXELS: f32 = 200.0;
                let delta = delta as f32 / DRAG_PIXELS;

                let mut control_ref = control.borrow_mut();
                let range = control_ref.range;
                let delta = delta * (range.1 - range.0) as f32;
                let lane = &mut control_ref.automation[*lane_index];
                lane.range.1 = (lane.range.1 + delta).clam(range.0, range.1);
            }
            Self::MoveModule(module) => {
                let mut module_ref = module.borrow_mut();
                module_ref.pos.0 += delta.0;
                module_ref.pos.1 += delta.1;
            }
            Self::PanOffset(offset) => {
                let mut offset_ref = offset.borrow_mut();
                offset_ref.0 += delta.0;
                offset_ref.1 += delta.1;
            }
            _ => (),
        }
    }

    fn on_drop(&mut self, target: DropTarget) {
        match self {
            Self::ConnectInput(in_module, in_index) => {
                let mut in_ref = in_module.borrow_mut();
                let in_type = in_ref.input_jacks[*in_index].get_type();
                if let DropTarget::Output(out_module, out_index) = target {
                    let out_type = out_module.borrow().output_jacks[out_index].get_type();
                    if in_type == out_type {
                        in_ref.inputs[*in_index] = ep::InputConnection::Wire(out_module, out_index);
                    }
                } else {
                    in_ref.inputs[*in_index] = ep::InputConnection::Default;
                }
            }
            Self::ConnectOutput(out_module, out_index) => {
                let out_type = out_module.borrow().output_jacks[*out_index].get_type();
                if let DropTarget::Input(in_module, in_index) = target {
                    let mut in_ref = in_module.borrow_mut();
                    let in_type = in_ref.input_jacks[in_index].get_type();
                    if in_type == out_type {
                        in_ref.inputs[in_index] =
                            ep::InputConnection::Wire(Rc::clone(out_module), *out_index);
                    }
                } else if let DropTarget::Control(control) = target {
                    if out_type == ep::JackType::Audio {
                        let mut control_ref = control.borrow_mut();
                        let range = control_ref.range;
                        control_ref.automation.push(ep::AutomationLane {
                            connection: (Rc::clone(out_module), *out_index),
                            range,
                        });
                    }
                }
            }
            _ => (),
        }
    }

    fn on_click(&mut self, root_widget: &mut audio_widgets::ModuleGraph) {
        match self {
            Self::OpenMenu(menu) => {
                root_widget.open_menu(menu.clone());
            }
            _ => (),
        }
    }
}

pub enum DropTarget {
    None,
    Control(Rcrc<ep::Control>),
    Input(Rcrc<ep::Module>, usize),
    Output(Rcrc<ep::Module>, usize),
}

impl DropTarget {
    pub fn is_none(&self) -> bool {
        if let Self::None = self {
            true
        } else {
            false
        }
    }
}

enum Screen {
    Graph,
    ModuleLibrary,
}

pub struct Gui {
    current_screen: Screen,
    menu_bar: other_widgets::MenuBar,
    graph: audio_widgets::ModuleGraph,

    mouse_action: MouseAction,
    click_position: (i32, i32),
    mouse_pos: (i32, i32),
    mouse_down: bool,
    dragged: bool,
}

impl Gui {
    pub fn new(registry: &Registry, graph_ref: Rcrc<ep::ModuleGraph>) -> Self {
        Self {
            current_screen: Screen::Graph,
            menu_bar: other_widgets::MenuBar::create(registry),
            graph: ep::ModuleGraph::build_gui(graph_ref),

            mouse_action: MouseAction::None,
            click_position: (0, 0),
            mouse_pos: (0, 0),
            mouse_down: false,
            dragged: false,
        }
    }

    pub fn draw(&self, g: &mut GrahpicsWrapper) {
        g.push_state();
        self.graph.draw(g, self);
        let current_screen_index = match self.current_screen {
            Screen::Graph => 0,
            Screen::ModuleLibrary => 1,
        };
        // TODO: Dynamic width.
        self.menu_bar.draw(640, current_screen_index, g);
        g.pop_state();
    }

    pub fn on_mouse_down(&mut self, pos: (i32, i32), mods: &MouseMods) {
        self.mouse_action = self.menu_bar.respond_to_mouse_press(pos, mods);
        if self.mouse_action.is_none() {
            self.mouse_action = self.graph.respond_to_mouse_press(pos, mods);
        }
        self.mouse_down = true;
        self.click_position = pos;
    }

    /// Minimum number of pixels the mouse must move before dragging starts.
    const MIN_DRAG_DELTA: i32 = 4;
    pub fn on_mouse_move(&mut self, new_pos: (i32, i32)) {
        self.mouse_pos = new_pos;
        if self.mouse_down {
            let delta = (
                new_pos.0 - self.click_position.0,
                new_pos.1 - self.click_position.1,
            );
            if !self.dragged {
                let distance = delta.0.abs() + delta.1.abs();
                if distance > Self::MIN_DRAG_DELTA {
                    self.dragged = true;
                }
            }
            if self.dragged {
                self.mouse_action.on_drag(delta);
                self.click_position = new_pos;
            }
        }
    }

    pub fn on_mouse_up(&mut self) {
        if self.dragged {
            let drop_target = self.graph.get_drop_target_at(self.mouse_pos);
            self.mouse_action.on_drop(drop_target);
        } else {
            self.mouse_action.on_click(&mut self.graph);
        }
        self.mouse_action = MouseAction::None;
        self.dragged = false;
        self.mouse_down = false;
    }

    pub(super) fn is_dragging(&self) -> bool {
        self.dragged
    }

    pub(super) fn borrow_current_mouse_action(&self) -> &MouseAction {
        &self.mouse_action
    }

    pub(super) fn get_current_mouse_pos(&self) -> (i32, i32) {
        self.mouse_pos
    }
}
