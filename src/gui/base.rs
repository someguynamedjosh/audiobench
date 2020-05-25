use crate::engine::parts as ep;
use crate::engine::registry::Registry;
use crate::gui::action::{GuiAction, InstanceAction, MouseAction};
use crate::gui::graphics::GrahpicsWrapper;
use crate::gui::{audio_widgets, other_widgets};
use crate::util::*;
use enumflags2::BitFlags;
use std::time::{Duration, Instant};

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
    pub shift: bool,
    pub precise: bool,
}

#[derive(Eq, PartialEq, Clone, Copy)]
pub enum GuiScreen {
    NoteGraph,
    ModuleCatalog,
}

impl GuiScreen {
    fn all() -> Vec<GuiScreen> {
        vec![Self::NoteGraph, Self::ModuleCatalog]
    }

    pub fn get_icon_name(&self) -> &'static str {
        match self {
            Self::NoteGraph => "base:note",
            Self::ModuleCatalog => "base:add",
        }
    }

    pub fn get_tooltip_text(&self) -> &'static str {
        match self {
            Self::NoteGraph => "Note graph: Edit the module graph used to synthesize notes",
            Self::ModuleCatalog => "Module catalog: Add new modules to the current graph",
        }
    }
}

pub struct Gui {
    size: (i32, i32),
    current_screen: GuiScreen,
    menu_bar: other_widgets::MenuBar,
    graph: audio_widgets::ModuleGraph,
    module_catalog: other_widgets::ModuleCatalog,

    mouse_action: MouseAction,
    click_position: (i32, i32),
    mouse_pos: (i32, i32),
    mouse_down: bool,
    dragged: bool,
    last_click: Instant,
}

impl Gui {
    pub fn new(registry: &Registry, graph_ref: Rcrc<ep::ModuleGraph>) -> Self {
        let size = (640, 480);
        let y = other_widgets::MenuBar::HEIGHT;

        let mut graph = audio_widgets::ModuleGraph::create(registry, graph_ref);
        graph.pos.1 = y;
        let module_catalog =
            other_widgets::ModuleCatalog::create(registry, (0, y), (size.0, size.1 - y));

        Self {
            size,
            current_screen: GuiScreen::NoteGraph,
            menu_bar: other_widgets::MenuBar::create(registry, GuiScreen::all()),
            graph,
            module_catalog,

            mouse_action: MouseAction::None,
            click_position: (0, 0),
            mouse_pos: (0, 0),
            mouse_down: false,
            dragged: false,
            last_click: Instant::now() - Duration::from_secs(100),
        }
    }

    pub fn draw(&self, g: &mut GrahpicsWrapper) {
        match self.current_screen {
            GuiScreen::NoteGraph => self.graph.draw(g, self),
            GuiScreen::ModuleCatalog => self.module_catalog.draw(g),
        }
        self.menu_bar.draw(self.size.0, self.current_screen, g);
    }

    pub fn on_mouse_down(&mut self, pos: (i32, i32), mods: &MouseMods) {
        if pos.1 <= other_widgets::MenuBar::HEIGHT {
            self.mouse_action = self.menu_bar.respond_to_mouse_press(pos, mods);
        } else {
            self.mouse_action = match self.current_screen {
                GuiScreen::NoteGraph => self.graph.respond_to_mouse_press(pos, mods),
                GuiScreen::ModuleCatalog => self.module_catalog.respond_to_mouse_press(pos, mods),
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
        mods: &MouseMods,
    ) -> Option<InstanceAction> {
        let mut retval = None;
        self.mouse_pos = new_pos;
        let mut new_tooltip = None;
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
                let (gui_action, tooltip) = self.mouse_action.on_drag(delta, mods);
                new_tooltip = tooltip;
                self.click_position = new_pos;
                retval = gui_action
                    .map(|action| self.perform_action(registry, action))
                    .flatten();
            }
        }
        if new_tooltip.is_none() {
            new_tooltip = self.menu_bar.get_tooltip_at(new_pos);
        }
        if new_tooltip.is_none() {
            // TODO: Module library tooltips?
            new_tooltip = match self.current_screen {
                GuiScreen::NoteGraph => self.graph.get_tooltip_at(new_pos),
                GuiScreen::ModuleCatalog => self.module_catalog.get_tooltip_at(new_pos),
            }
        }
        if let Some(tooltip) = new_tooltip {
            self.menu_bar.set_tooltip(tooltip);
        } else {
            self.menu_bar.set_tooltip(Tooltip::default());
        }
        retval
    }

    fn perform_action(&mut self, registry: &Registry, action: GuiAction) -> Option<InstanceAction> {
        match action {
            GuiAction::OpenMenu(menu) => self.graph.open_menu(menu),
            GuiAction::SwitchScreen(new_index) => self.current_screen = new_index,
            GuiAction::AddModule(module) => {
                self.graph.add_module(registry, module);
                self.current_screen = GuiScreen::NoteGraph;
                return Some(InstanceAction::ReloadStructure);
            }
            GuiAction::Elevate(action) => return Some(action),
        }
        None
    }

    const DOUBLE_CLICK_TIME: Duration = Duration::from_millis(500);
    pub fn on_mouse_up(&mut self, registry: &Registry) -> Option<InstanceAction> {
        let mouse_action = std::mem::replace(&mut self.mouse_action, MouseAction::None);
        let gui_action = if self.dragged {
            let drop_target = self.graph.get_drop_target_at(self.mouse_pos);
            mouse_action.on_drop(drop_target)
        } else {
            if self.last_click.elapsed() < Self::DOUBLE_CLICK_TIME {
                mouse_action.on_double_click()
            } else {
                self.last_click = Instant::now();
                mouse_action.on_click()
            }
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
