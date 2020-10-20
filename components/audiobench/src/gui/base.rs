use crate::engine::parts as ep;
use crate::gui;
use crate::gui::action::{GuiRequest, InstanceRequest, MouseAction};
use crate::gui::constants::*;
use crate::gui::graphics::{GrahpicsWrapper, HAlign, VAlign};
use crate::gui::ui_widgets::UiTab;
use crate::registry::save_data::Patch;
use crate::registry::Registry;
use enumflags2::BitFlags;
use shared_util::prelude::*;
use std::time::{Duration, Instant};

#[derive(BitFlags, Copy, Clone)]
#[repr(u8)]
pub enum InteractionHint {
    LeftClick = 0x1,
    RightClick = 0x2,
    Scroll = 0x40,
    LeftClickAndDrag = 0x4,
    DoubleClick = 0x8,
    PrecisionModifier = 0x10,
    SnappingModifier = 0x20,
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
    pub snap: bool,
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
                "Info/Library Browser: View current Audiobench version and installed libraries"
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
    graph: Rcrc<gui::graph::ModuleGraph>,
    module_browser: gui::ui_widgets::ModuleBrowser,

    mouse_action: Option<Box<dyn MouseAction>>,
    click_position: (f32, f32),
    mouse_pos: (f32, f32),
    mouse_down: bool,
    dragged: bool,
    last_click: Instant,
    focused_text_field: Option<Rcrc<gui::ui_widgets::TextField>>,
    update_check_complete: bool,
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
        let graph = rcrc(graph);
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

            mouse_action: None,
            click_position: (0.0, 0.0),
            mouse_pos: (0.0, 0.0),
            mouse_down: false,
            dragged: false,
            last_click: Instant::now() - Duration::from_secs(100),
            focused_text_field: None,
            update_check_complete: false,
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

    pub fn draw(
        &mut self,
        g: &mut GrahpicsWrapper,
        registry: &mut Registry,
        currently_compiling: bool,
    ) {
        if !self.update_check_complete {
            // Returns false when update checker is complete.
            if !registry.poll_update_checker() {
                self.update_check_complete = true;
                if registry.any_updates_available() {
                    self.display_success(
                        "Updates are available! Go to the Info/Libraries tab to view them"
                            .to_owned(),
                    );
                }
            }
        }
        match self.current_screen {
            GuiScreen::LibraryBrowser => self.library_browser.draw(g, &*registry),
            GuiScreen::PatchBrowser => self.patch_browser.draw(g),
            GuiScreen::NoteGraph => self.graph.draw(g, self),
            GuiScreen::ModuleBrowser => self.module_browser.draw(g),
        }
        self.menu_bar.draw(self.size.0, self.current_screen, g);
        if currently_compiling {
            g.set_color(&COLOR_WARNING);
            g.fill_rounded_rect(
                GRID_P,
                gui::ui_widgets::MenuBar::HEIGHT + GRID_P,
                100.0,
                FONT_SIZE + GRID_P * 2.0,
                CORNER_SIZE,
            );
            g.set_color(&COLOR_FG1);
            g.write_text(
                FONT_SIZE,
                GRID_P * 2.0,
                gui::ui_widgets::MenuBar::HEIGHT + GRID_P,
                100.0 - GRID_P * 2.0,
                FONT_SIZE + GRID_P * 2.0,
                HAlign::Center,
                VAlign::Center,
                1,
                "Compiling...",
            );
        }
    }

    pub fn on_patch_change(&mut self, registry: &Registry) {
        self.graph.borrow_mut().rebuild(registry);
    }

    pub fn on_mouse_down(
        &mut self,
        registry: &Registry,
        pos: (f32, f32),
        mods: &MouseMods,
    ) -> Vec<InstanceRequest> {
        self.menu_bar.clear_status();
        let mut ret = Vec::new();
        if let Some(field) = self.focused_text_field.take() {
            let requests = field.borrow_mut().defocus();
            self.do_requests(registry, requests, &mut ret);
        };
        if pos.1 <= gui::ui_widgets::MenuBar::HEIGHT {
            self.mouse_action = self.menu_bar.respond_to_mouse_press(pos, mods);
        } else {
            self.mouse_action = match self.current_screen {
                GuiScreen::LibraryBrowser => self.library_browser.respond_to_mouse_press(pos, mods),
                GuiScreen::PatchBrowser => self.patch_browser.respond_to_mouse_press(pos, mods),
                GuiScreen::NoteGraph => self.graph.respond_to_mouse_press(pos, mods),
                GuiScreen::ModuleBrowser => self.module_browser.respond_to_mouse_press(pos, mods),
            };
        }
        self.mouse_down = true;
        self.click_position = pos;
        ret
    }

    pub fn on_mouse_move(
        &mut self,
        registry: &Registry,
        new_pos: (f32, f32),
        mods: &MouseMods,
    ) -> Vec<InstanceRequest> {
        let mut ret = Vec::new();
        self.mouse_pos = new_pos;
        if self.mouse_down {
            let delta = (
                new_pos.0 - self.click_position.0,
                new_pos.1 - self.click_position.1,
            );
            if !self.dragged {
                let distance = delta.0.abs() + delta.1.abs();
                if distance > DRAG_DEADZONE {
                    self.dragged = true;
                }
            }
            if self.dragged {
                let fdelta = (delta.0 as f32, delta.1 as f32);
                if let Some(ma) = &mut self.mouse_action {
                    let requests = ma.on_drag(fdelta, mods);
                    self.click_position = new_pos;
                    self.do_requests(registry, requests, &mut ret);
                }
            }
        } else {
            let tooltip = match self.current_screen {
                GuiScreen::LibraryBrowser => self.library_browser.get_tooltip_at(new_pos),
                GuiScreen::PatchBrowser => self.patch_browser.get_tooltip_at(new_pos),
                GuiScreen::NoteGraph => self.graph.get_tooltip_at(new_pos),
                GuiScreen::ModuleBrowser => self.module_browser.get_tooltip_at(new_pos),
            };
            if let Some(tooltip) = tooltip {
                self.menu_bar.set_tooltip(tooltip);
            } else if let Some(tooltip) = self.menu_bar.get_tooltip_at(new_pos) {
                self.menu_bar.set_tooltip(tooltip);
            } else {
                self.menu_bar.set_tooltip(Default::default());
            }
        }
        ret
    }

    fn do_requests(
        &mut self,
        registry: &Registry,
        requests: Vec<GuiRequest>,
        output: &mut Vec<InstanceRequest>,
    ) {
        for request in requests {
            match request {
                GuiRequest::ShowTooltip(tooltip) => self.menu_bar.set_tooltip(tooltip),
                GuiRequest::OpenMenu(..) => unreachable!(),
                GuiRequest::SwitchScreen(new_index) => self.current_screen = new_index,
                GuiRequest::AddModule(module) => {
                    self.graph.borrow_mut().add_module(registry, module);
                    self.current_screen = GuiScreen::NoteGraph;
                    output.push(InstanceRequest::ReloadStructure);
                }
                GuiRequest::RemoveModule(module) => {
                    self.graph.borrow_mut().remove_module(&module);
                    output.push(InstanceRequest::ReloadStructure);
                }
                GuiRequest::FocusTextField(field) => {
                    field.borrow_mut().focus();
                    self.focused_text_field = Some(field);
                }
                GuiRequest::Elevate(request) => output.push(request),
                GuiRequest::OpenWebpage(url) => {
                    if let Err(err) = webbrowser::open(&url) {
                        self.display_error(format!(
                            "Failed to open webpage, see console for details."
                        ));
                        eprintln!(
                            "WARNING: Failed to open web browser, caused by:\nERROR: {}",
                            err
                        );
                    }
                }
            }
        }
    }

    pub fn on_mouse_up(&mut self, registry: &Registry) -> Vec<InstanceRequest> {
        let mouse_action = self.mouse_action.take();
        let mut ret = Vec::new();
        if let Some(ma) = mouse_action {
            let requests = if self.dragged {
                let drop_target = self.graph.get_drop_target_at(self.mouse_pos);
                ma.on_drop(drop_target)
            } else {
                if self.last_click.elapsed() < DOUBLE_CLICK_TIME {
                    ma.on_double_click()
                } else {
                    self.last_click = Instant::now();
                    ma.on_click()
                }
            };
            self.do_requests(registry, requests, &mut ret);
        }
        self.dragged = false;
        self.mouse_down = false;
        ret
    }

    pub fn on_scroll(&mut self, registry: &Registry, delta: f32) -> Vec<InstanceRequest> {
        let requests = if let GuiScreen::NoteGraph = self.current_screen {
            self.graph.on_scroll(delta)
        } else if let GuiScreen::PatchBrowser = self.current_screen {
            self.patch_browser.on_scroll(self.mouse_pos, delta)
        } else if let GuiScreen::LibraryBrowser = self.current_screen {
            self.library_browser.on_scroll(delta)
        } else {
            Vec::new()
        };
        let mut ret = Vec::new();
        self.do_requests(registry, requests, &mut ret);
        ret
    }

    pub fn on_key_press(&mut self, registry: &Registry, key: u8) -> Vec<InstanceRequest> {
        // For some reason JUCE gives CR for enter instead of LF.
        let key = if key == 13 { 10 } else { key };
        let mut ret = Vec::new();
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
                    let requests = field.defocus();
                    drop(field);
                    self.focused_text_field = None;
                    self.do_requests(registry, requests, &mut ret);
                }
                _ => {
                    field.text.push(key as char);
                }
            }
        }
        ret
    }

    pub(super) fn get_current_mouse_pos(&self) -> (f32, f32) {
        self.mouse_pos
    }
}
