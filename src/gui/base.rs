use crate::engine::parts as ep;
use crate::engine::registry::Registry;
use crate::gui::graphics::GrahpicsWrapper;
use crate::gui::{audio_widgets, other_widgets};
use crate::util::*;

pub struct MouseMods {
    pub right_click: bool,
}

// Describes an action the GUI object should perform. Prevents passing a bunch of arguments to
// MouseAction functions for each action that needs to modify something in the GUI.
pub enum GuiAction {
    OpenMenu(Box<audio_widgets::KnobEditor>),
    SwitchScreen(usize),
    AddModule(ep::Module),
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
    SwitchScreen(usize),
    AddModule(ep::Module),
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

    fn on_drop(self, target: DropTarget) -> Option<GuiAction> {
        match self {
            Self::ConnectInput(in_module, in_index) => {
                let mut in_ref = in_module.borrow_mut();
                let template_ref = in_ref.template.borrow();
                let in_type = template_ref.inputs[in_index].get_type();
                drop(template_ref);
                if let DropTarget::Output(out_module, out_index) = target {
                    let out_type =
                        out_module.borrow().template.borrow().outputs[out_index].get_type();
                    if in_type == out_type {
                        in_ref.inputs[in_index] = ep::InputConnection::Wire(out_module, out_index);
                    }
                } else {
                    in_ref.inputs[in_index] = ep::InputConnection::Default;
                }
            }
            Self::ConnectOutput(out_module, out_index) => {
                let out_type = out_module.borrow().template.borrow().inputs[out_index].get_type();
                if let DropTarget::Input(in_module, in_index) = target {
                    let mut in_ref = in_module.borrow_mut();
                    let in_type = in_ref.template.borrow().inputs[in_index].get_type();
                    if in_type == out_type {
                        in_ref.inputs[in_index] = ep::InputConnection::Wire(out_module, out_index);
                    }
                } else if let DropTarget::Control(control) = target {
                    if out_type == ep::JackType::Audio {
                        let mut control_ref = control.borrow_mut();
                        let range = control_ref.range;
                        control_ref.automation.push(ep::AutomationLane {
                            connection: (out_module, out_index),
                            range,
                        });
                    }
                }
            }
            _ => (),
        }
        None
    }

    fn on_click(self) -> Option<GuiAction> {
        match self {
            Self::OpenMenu(menu) => return Some(GuiAction::OpenMenu(menu)),
            Self::SwitchScreen(screen_index) => return Some(GuiAction::SwitchScreen(screen_index)),
            Self::AddModule(module) => return Some(GuiAction::AddModule(module)),
            _ => (),
        }
        None
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

const MODULE_GRAPH_SCREEN: usize = 0;
const MODULE_LIBRARY_SCREEN: usize = 1;

pub struct Gui {
    size: (i32, i32),
    current_screen: usize,
    menu_bar: other_widgets::MenuBar,
    graph: audio_widgets::ModuleGraph,
    module_library: other_widgets::ModuleLibrary,

    mouse_action: MouseAction,
    click_position: (i32, i32),
    mouse_pos: (i32, i32),
    mouse_down: bool,
    dragged: bool,
}

impl Gui {
    pub fn new(registry: &Registry, graph_ref: Rcrc<ep::ModuleGraph>) -> Self {
        let size = (640, 480);
        let y = other_widgets::MenuBar::HEIGHT;

        let mut graph = audio_widgets::ModuleGraph::create(graph_ref);
        graph.pos.1 = y;
        let module_library =
            other_widgets::ModuleLibrary::create(registry, (0, y), (size.0, size.1 - y));

        Self {
            size,
            current_screen: MODULE_GRAPH_SCREEN,
            menu_bar: other_widgets::MenuBar::create(registry),
            graph,
            module_library,

            mouse_action: MouseAction::None,
            click_position: (0, 0),
            mouse_pos: (0, 0),
            mouse_down: false,
            dragged: false,
        }
    }

    pub fn draw(&self, g: &mut GrahpicsWrapper) {
        match self.current_screen {
            MODULE_GRAPH_SCREEN => self.graph.draw(g, self),
            MODULE_LIBRARY_SCREEN => self.module_library.draw(g),
            _ => unreachable!(),
        }
        self.menu_bar.draw(self.size.0, self.current_screen, g);
    }

    pub fn on_mouse_down(&mut self, pos: (i32, i32), mods: &MouseMods) {
        if pos.1 <= other_widgets::MenuBar::HEIGHT {
            self.mouse_action = self.menu_bar.respond_to_mouse_press(pos, mods);
        } else {
            self.mouse_action = match self.current_screen {
                MODULE_GRAPH_SCREEN => self.graph.respond_to_mouse_press(pos, mods),
                MODULE_LIBRARY_SCREEN => self.module_library.respond_to_mouse_press(pos, mods),
                _ => unreachable!(),
            };
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

    fn perform_action(&mut self, action: GuiAction) {
        match action {
            GuiAction::OpenMenu(menu) => self.graph.open_menu(menu),
            GuiAction::SwitchScreen(new_index) => self.current_screen = new_index,
            GuiAction::AddModule(module) => {
                self.graph.add_module(module);
            }
        }
    }

    pub fn on_mouse_up(&mut self) {
        let mouse_action = std::mem::replace(&mut self.mouse_action, MouseAction::None);
        let gui_action = if self.dragged {
            let drop_target = self.graph.get_drop_target_at(self.mouse_pos);
            mouse_action.on_drop(drop_target)
        } else {
            mouse_action.on_click()
        };
        gui_action.map(|action| self.perform_action(action));
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
