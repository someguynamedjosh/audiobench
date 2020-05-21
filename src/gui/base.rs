use crate::engine::parts as ep;
use crate::engine::registry::Registry;
use crate::gui::action::{GuiAction, InstanceAction, MouseAction};
use crate::gui::graphics::GrahpicsWrapper;
use crate::gui::{audio_widgets, other_widgets};
use crate::util::*;
use enumflags2::BitFlags;

#[derive(BitFlags, Copy, Clone)]
#[repr(u8)]
pub enum InteractionHint {
    LeftClick = 0x1,
    RightClick = 0x2,
    LeftClickAndDrag = 0x4,
    DoubleClick = 0x8,
}

#[derive(Clone)]
pub struct Tooltip {
    pub text: String,
    pub interaction: BitFlags<InteractionHint>,
}

impl Default for Tooltip {
    fn default() -> Tooltip {
        Tooltip {
            text: "".to_owned(),
            interaction: Default::default(),
        }
    }
}

pub struct MouseMods {
    pub right_click: bool,
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

        let mut graph = audio_widgets::ModuleGraph::create(registry, graph_ref);
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
    pub fn on_mouse_move(
        &mut self,
        registry: &Registry,
        new_pos: (i32, i32),
    ) -> Option<InstanceAction> {
        self.mouse_pos = new_pos;
        let new_tooltip = if self.mouse_down {
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
                let gui_action = self.mouse_action.on_drag(delta);
                self.click_position = new_pos;
                return gui_action
                    .map(|action| self.perform_action(registry, action))
                    .flatten();
            }
            // TODO: Tooltips while dragging.
            None
        } else {
            // TODO: Module library tooltips?
            match self.current_screen {
                MODULE_GRAPH_SCREEN => self.graph.get_tooltip_at(new_pos),
                _ => None
            }
        };
        if let Some(tooltip) = new_tooltip {
            self.menu_bar.set_tooltip(tooltip);
        } else {
            self.menu_bar.set_tooltip(Tooltip::default());
        }
        None
    }

    fn perform_action(&mut self, registry: &Registry, action: GuiAction) -> Option<InstanceAction> {
        match action {
            GuiAction::OpenMenu(menu) => self.graph.open_menu(menu),
            GuiAction::SwitchScreen(new_index) => self.current_screen = new_index,
            GuiAction::AddModule(module) => {
                self.graph.add_module(registry, module);
                return Some(InstanceAction::ReloadStructure);
            }
            GuiAction::Elevate(action) => return Some(action),
        }
        None
    }

    pub fn on_mouse_up(&mut self, registry: &Registry) -> Option<InstanceAction> {
        let mouse_action = std::mem::replace(&mut self.mouse_action, MouseAction::None);
        let gui_action = if self.dragged {
            let drop_target = self.graph.get_drop_target_at(self.mouse_pos);
            mouse_action.on_drop(drop_target)
        } else {
            mouse_action.on_click()
        };
        self.dragged = false;
        self.mouse_down = false;
        gui_action
            .map(|action| self.perform_action(registry, action))
            .flatten()
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
