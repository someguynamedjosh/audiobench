use crate::engine;
use crate::gui::graphics::GrahpicsWrapper;
use crate::gui::widgets;
use crate::util::*;

pub enum MouseAction {
    None,
    ManipulateControl(Rcrc<engine::Control>),
    MoveModule(Rcrc<engine::Module>),
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
            Self::None => (),
            Self::ManipulateControl(control) => {
                let delta = delta.0 - delta.1;
                // How many pixels the user must drag across to cover the entire range of the knob.
                const DRAG_PIXELS: f32 = 200.0;
                let delta = delta as f32 / DRAG_PIXELS;

                let mut control_ref = control.borrow_mut();
                let range = control_ref.range;
                let delta = delta * (range.1 - range.0) as f32;
                control_ref.value = (control_ref.value + delta).clam(range.0, range.1);
            }
            Self::MoveModule(module) => {
                let mut module_ref = module.borrow_mut();
                module_ref.pos.0 += delta.0;
                module_ref.pos.1 += delta.1;
            }
        }
    }
}

pub struct Gui {
    root_widget: widgets::ModuleGraph,
    mouse_action: MouseAction,
    click_position: (i32, i32),
    mouse_down: bool,
    dragged: bool,
}

impl Gui {
    pub fn new(root: widgets::ModuleGraph) -> Self {
        Self {
            root_widget: root,
            mouse_action: MouseAction::None,
            click_position: (0, 0),
            mouse_down: false,
            dragged: false,
        }
    }

    pub fn draw(&self, g: &mut GrahpicsWrapper) {
        self.root_widget.draw(g);
    }

    pub fn on_mouse_down(&mut self, pos: (i32, i32)) {
        self.mouse_action = self.root_widget.respond_to_mouse_press(pos);
        self.mouse_down = true;
        self.click_position = pos;
    }

    /// Minimum number of pixels the mouse must move before dragging starts.
    const MIN_DRAG_DELTA: i32 = 4;
    pub fn on_mouse_move(&mut self, new_pos: (i32, i32)) {
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
        self.dragged = false;
        self.mouse_down = false;
    }
}
