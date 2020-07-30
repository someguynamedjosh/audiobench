use crate::engine::parts as ep;
use crate::gui;
use crate::gui::action::{GuiAction, InstanceAction, MouseAction};
use crate::gui::constants::*;
use crate::gui::graphics::GrahpicsWrapper;
use crate::registry::save_data::Patch;
use crate::registry::Registry;
use crate::util::*;
use enumflags2::BitFlags;
use std::time::{Duration, Instant};

#[derive(BitFlags, Copy, Clone)]
#[repr(u8)]
pub enum InteractionHint {
    LeftClick = 0x1,
    RightClick = 0x2,
    Scroll = 0x40,
    LeftClickAndDrag = 0x4,
    DoubleClick = 0x8,
    Alt = 0x10,
    Shift = 0x20,
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

#[derive(Clone)]
pub struct Status {
    pub text: String,
    pub color: (u8, u8, u8),
}

impl Status {
    fn success(text: String) -> Self {
        Self {
            text,
            color: COLOR_SUCCESS,
        }
    }

    fn error(text: String) -> Self {
        Self {
            text,
            color: COLOR_ERROR,
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
    LibraryBrowser,
    PatchBrowser,
    NoteGraph,
    ModuleBrowser,
}

impl GuiScreen {
    fn all() -> Vec<GuiScreen> {
        vec![
            Self::LibraryBrowser,
            Self::PatchBrowser,
            Self::NoteGraph,
            Self::ModuleBrowser,
        ]
    }

    pub fn get_icon_name(&self) -> &'static str {
        match self {
            Self::LibraryBrowser => "factory:library",
            Self::PatchBrowser => "factory:patch_browser",
            Self::NoteGraph => "factory:note",
            Self::ModuleBrowser => "factory:add",
        }
    }

    pub fn get_tooltip_text(&self) -> &'static str {
        match self {
            Self::LibraryBrowser => {
                "Info / Library Browser: View current Audiobench version and installed libraries"
            }
            Self::PatchBrowser => "Patch Browser: Save and load patches",
            Self::NoteGraph => "Note Graph: Edit the module graph used to synthesize notes",
            Self::ModuleBrowser => "Module Browser: Add new modules to the current graph",
        }
    }
}

pub struct Gui {
    size: (f32, f32),
    current_screen: GuiScreen,
    menu_bar: gui::ui_widgets::MenuBar,
    library_browser: gui::ui_widgets::LibraryBrowser,
    patch_browser: gui::ui_widgets::PatchBrowser,
    graph: gui::graph::ModuleGraph,
    module_browser: gui::ui_widgets::ModuleBrowser,

    mouse_action: MouseAction,
    click_position: (f32, f32),
    mouse_pos: (f32, f32),
    mouse_down: bool,
    dragged: bool,
    last_click: Instant,
    focused_text_field: Option<Rcrc<gui::ui_widgets::TextField>>,
}

impl Gui {
    pub fn new(
        registry: &Registry,
        current_patch: &Rcrc<Patch>,
        graph_ref: Rcrc<ep::ModuleGraph>,
    ) -> Self {
        let size = (640.0, 480.0);
        let y = gui::ui_widgets::MenuBar::HEIGHT;
        let screen_size = (size.0, size.1 - y);

        let library_browser =
            gui::ui_widgets::LibraryBrowser::create(registry, (0.0, y), screen_size);
        let patch_browser =
            gui::ui_widgets::PatchBrowser::create(current_patch, registry, (0.0, y), screen_size);
        let mut graph = gui::graph::ModuleGraph::create(registry, graph_ref, screen_size);
        graph.pos.1 = y;
        let module_browser =
            gui::ui_widgets::ModuleBrowser::create(registry, (0.0, y), (size.0, size.1 - y));

        Self {
            size,
            current_screen: GuiScreen::NoteGraph,
            menu_bar: gui::ui_widgets::MenuBar::create(registry, GuiScreen::all()),
            library_browser,
            patch_browser,
            graph,
            module_browser,

            mouse_action: MouseAction::None,
            click_position: (0.0, 0.0),
            mouse_pos: (0.0, 0.0),
            mouse_down: false,
            dragged: false,
            last_click: Instant::now() - Duration::from_secs(100),
            focused_text_field: None,
        }
    }

    pub fn display_success(&mut self, text: String) {
        self.menu_bar.set_status(Status::success(text));
    }

    pub fn display_error(&mut self, text: String) {
        self.menu_bar.set_status(Status::error(text));
    }

    pub fn clear_status(&mut self) {
        self.menu_bar.clear_status();
    }

    pub fn draw(&self, g: &mut GrahpicsWrapper) {
        match self.current_screen {
            GuiScreen::LibraryBrowser => self.library_browser.draw(g),
            GuiScreen::PatchBrowser => self.patch_browser.draw(g),
            GuiScreen::NoteGraph => self.graph.draw(g, self),
            GuiScreen::ModuleBrowser => self.module_browser.draw(g),
        }
        self.menu_bar.draw(self.size.0, self.current_screen, g);
    }

    pub fn on_patch_change(&mut self, registry: &Registry) {
        self.graph.rebuild(registry);
    }

    pub fn on_mouse_down(
        &mut self,
        registry: &Registry,
        pos: (f32, f32),
        mods: &MouseMods,
    ) -> Option<InstanceAction> {
        self.menu_bar.clear_status();
        let mut ret = None;
        if let Some(field) = self.focused_text_field.take() {
            if let Some(action) = field.borrow_mut().defocus().on_click() {
                ret = self.perform_action(registry, action);
            }
        };
        if pos.1 <= gui::ui_widgets::MenuBar::HEIGHT {
            self.mouse_action = self.menu_bar.respond_to_mouse_press(pos, mods);
        } else {
            self.mouse_action = match self.current_screen {
                GuiScreen::LibraryBrowser => MouseAction::None,
                GuiScreen::PatchBrowser => self.patch_browser.respond_to_mouse_press(pos, mods),
                GuiScreen::NoteGraph => self.graph.respond_to_mouse_press(pos, mods),
                GuiScreen::ModuleBrowser => self.module_browser.respond_to_mouse_press(pos, mods),
            };
        }
        self.mouse_down = true;
        self.click_position = pos;
        ret
    }

    /// Minimum number of pixels the mouse must move before dragging starts.
    const MIN_DRAG_DELTA: f32 = 4.0;
    pub fn on_mouse_move(
        &mut self,
        registry: &Registry,
        new_pos: (f32, f32),
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
                let fdelta = (delta.0 as f32, delta.1 as f32);
                let (gui_action, tooltip) = self.mouse_action.on_drag(fdelta, mods);
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
            new_tooltip = match self.current_screen {
                GuiScreen::LibraryBrowser => None,
                GuiScreen::PatchBrowser => self.patch_browser.get_tooltip_at(new_pos),
                GuiScreen::NoteGraph => self.graph.get_tooltip_at(new_pos),
                GuiScreen::ModuleBrowser => self.module_browser.get_tooltip_at(new_pos),
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
            GuiAction::Sequence(actions) => {
                return Some(InstanceAction::Sequence(
                    actions
                        .into_iter()
                        .filter_map(|action| self.perform_action(registry, action))
                        .collect(),
                ));
            }
            GuiAction::OpenMenu(menu) => self.graph.open_menu(menu),
            GuiAction::SwitchScreen(new_index) => self.current_screen = new_index,
            GuiAction::AddModule(module) => {
                self.graph.add_module(registry, module);
                self.current_screen = GuiScreen::NoteGraph;
                return Some(InstanceAction::ReloadStructure);
            }
            GuiAction::RemoveModule(module) => {
                self.graph.remove_module(&module);
                return Some(InstanceAction::ReloadStructure);
            }
            GuiAction::FocusTextField(field) => {
                field.borrow_mut().focus();
                self.focused_text_field = Some(field);
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

    pub fn on_scroll(&mut self, registry: &Registry, delta: f32) -> Option<InstanceAction> {
        if let GuiScreen::NoteGraph = self.current_screen {
            return self
                .graph
                .on_scroll(delta)
                .map(|a| self.perform_action(registry, a))
                .flatten();
        } else if let GuiScreen::PatchBrowser = self.current_screen {
            return self
                .patch_browser
                .on_scroll(self.mouse_pos, delta)
                .map(|a| self.perform_action(registry, a))
                .flatten();
        } else if let GuiScreen::LibraryBrowser = self.current_screen {
            return self
                .library_browser
                .on_scroll(delta)
                .map(|a| self.perform_action(registry, a))
                .flatten();
        }
        None
    }

    pub fn on_key_press(&mut self, registry: &Registry, key: u8) -> Option<InstanceAction> {
        // For some reason JUCE gives CR for enter instead of LF.
        let key = if key == 13 { 10 } else { key };
        if let Some(field) = &self.focused_text_field {
            let mut field = field.borrow_mut();
            match key {
                0x8 | 0x7F => {
                    // Bksp / Del
                    if field.text.len() > 0 {
                        let last = field.text.len() - 1;
                        field.text = field.text[..last].to_owned();
                    }
                }
                0x1B | 0xA => {
                    // Esc / Enter
                    let action = field.defocus();
                    drop(field);
                    self.focused_text_field = None;
                    if let Some(action) = action.on_click() {
                        return self.perform_action(registry, action);
                    }
                }
                _ => {
                    field.text.push(key as char);
                }
            }
        }
        None
    }

    pub(super) fn is_dragging(&self) -> bool {
        self.dragged
    }

    pub(super) fn borrow_current_mouse_action(&self) -> &MouseAction {
        &self.mouse_action
    }

    pub(super) fn get_current_mouse_pos(&self) -> (f32, f32) {
        self.mouse_pos
    }
}
