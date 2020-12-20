pub(crate) mod config;
mod engine;
mod gui;
mod registry;

use gui::graphics::GrahpicsWrapper;
pub use gui::graphics::GraphicsFunctions;
use gui::{Gui, MouseMods};

use shared_util::prelude as util;

pub struct Instance {
    engine: Option<engine::Engine>,
    registry: registry::Registry,
    pub graphics_fns: GraphicsFunctions,
    gui: Option<Gui>,
    critical_error: Option<String>,
    structure_error: Option<String>,
    silence: Vec<f32>,
}

fn copy_to_clipboard(text: String) {
    use clipboard::ClipboardProvider;
    let mut clipboard: clipboard::ClipboardContext = clipboard::ClipboardProvider::new().unwrap();
    clipboard.set_contents(text).unwrap();
}

impl Instance {
    pub fn new() -> Self {
        let mut critical_error = None;

        let (mut registry, registry_load_result) = registry::Registry::new();
        if let Err(err) = registry_load_result {
            copy_to_clipboard(err.clone());
            critical_error = Some(format!(
                "The following error report has been copied to your clipboard:\n\n{}",
                err
            ));
        }
        let engine = if critical_error.is_none() {
            match engine::Engine::new(&mut registry) {
                Ok(engine) => Some(engine),
                Err(err) => {
                    copy_to_clipboard(err.clone());
                    critical_error = Some(format!(
                        "The following error report has been copied to your clipboard:\n\n{}",
                        err
                    ));
                    None
                }
            }
        } else {
            None
        };

        Self {
            engine,
            registry,
            graphics_fns: GraphicsFunctions::placeholders(),
            gui: None,
            critical_error,
            structure_error: None,
            silence: Vec::new(),
        }
    }
}

impl Instance {
    fn set_critical_error(&mut self, error: String) {
        if self.critical_error.is_some() {
            return;
        }
        copy_to_clipboard(error.clone());
        self.critical_error = Some(format!(
            "The following error report has been copied to your clipboard:\n\n{}",
            error
        ));
    }

    fn set_structure_error(&mut self, error: String) {
        if self.critical_error.is_some() || self.structure_error.is_some() {
            return;
        }
        copy_to_clipboard(error.clone());
        self.structure_error = Some(format!(
            "The following error report has been copied to your clipboard:\n\n{}",
            error
        ));
    }

    fn perform_action(&mut self, action: gui::action::InstanceAction) {
        match action {
            gui::action::InstanceAction::Sequence(actions) => {
                for action in actions {
                    self.perform_action(action);
                }
            }
            gui::action::InstanceAction::ReloadAutoconDynData => {
                self.engine.as_mut().map(|e| e.reload_autocon_dyn_data());
            }
            gui::action::InstanceAction::ReloadStaticonDynData => {
                self.engine.as_mut().map(|e| e.reload_staticon_dyn_data());
            }
            gui::action::InstanceAction::ReloadStructure => {
                if let Some(e) = self.engine.as_mut() {
                    let res = e.recompile();
                    if let Err(err) = res {
                        if let Some(gui) = &mut self.gui {
                            gui.display_error(err.clone());
                        }
                        self.set_structure_error(err);
                    } else {
                        if let Some(gui) = &mut self.gui {
                            gui.clear_status();
                        }
                        self.structure_error = None;
                    }
                }
            }
            gui::action::InstanceAction::RenamePatch(name) => {
                self.engine.as_mut().map(|e| e.rename_current_patch(name));
            }
            gui::action::InstanceAction::SavePatch(mut callback) => {
                if let Some(engine) = self.engine.as_mut() {
                    engine.save_current_patch(&self.registry);
                    callback(engine.borrow_current_patch());
                }
                self.gui
                    .as_mut()
                    .map(|g| g.display_success("Saved successfully!".to_owned()));
            }
            gui::action::InstanceAction::NewPatch(mut callback) => {
                if let Some(e) = self.engine.as_mut() {
                    callback(e.new_patch(&mut self.registry));
                }
            }
            gui::action::InstanceAction::LoadPatch(patch, mut callback) => {
                if let Some(e) = self.engine.as_mut() {
                    let res = e.load_patch(&self.registry, patch);
                    if res.is_ok() {
                        callback();
                    }
                    if let Some(gui) = &mut self.gui {
                        if let Err(err) = res {
                            gui.display_error(err);
                        }
                        gui.on_patch_change(&self.registry);
                    }
                }
            }
            gui::action::InstanceAction::SimpleCallback(mut callback) => {
                (callback)();
            }
            gui::action::InstanceAction::CopyPatchToClipboard => {
                if let Some(e) = self.engine.as_mut() {
                    use clipboard::ClipboardProvider;
                    let patch_data = e.serialize_current_patch(&self.registry);
                    let mut clipboard: clipboard::ClipboardContext =
                        clipboard::ClipboardProvider::new().unwrap();
                    clipboard.set_contents(patch_data).unwrap();
                    if let Some(gui) = &mut self.gui {
                        gui.display_success("Patch data copied to clipboard!".to_owned());
                    }
                }
            }
            gui::action::InstanceAction::PastePatchFromClipboard(mut callback) => {
                if let Some(e) = self.engine.as_mut() {
                    use clipboard::ClipboardProvider;
                    let mut clipboard: clipboard::ClipboardContext =
                        clipboard::ClipboardProvider::new().unwrap();
                    let data = clipboard.get_contents().unwrap();
                    // We use the URL-safe dataset, so letters, numbers, - and _.
                    // is_digit(36) checks for numbers and a-z case insensitive.
                    let data: String = data
                        .chars()
                        .filter(|character| {
                            character.is_digit(36) || *character == '-' || *character == '_'
                        })
                        .collect();
                    let err = match e.new_patch_from_clipboard(&mut self.registry, data.as_bytes())
                    {
                        Ok(patch) => {
                            callback(patch);
                            None
                        }
                        Err(err) => Some(err),
                    };
                    if let Some(gui) = &mut self.gui {
                        if let Some(err) = err {
                            gui.display_error(err);
                        } else {
                            gui.display_success("Patch data loaded from clipboard! (Click the save button if you want to keep it)".to_owned());
                        }
                        gui.on_patch_change(&self.registry);
                    }
                }
            }
        }
    }

    pub fn perf_report(&self) -> String {
        self.engine
            .as_ref()
            .map(|engine| engine.perf_counter_report())
            .unwrap_or("Engine failed to initialize.".to_owned())
    }

    pub fn get_num_icons(&self) -> usize {
        self.registry.get_num_icons()
    }

    pub fn borrow_icon_data(&self, icon_index: usize) -> &[u8] {
        self.registry.borrow_icon_data(icon_index)
    }

    pub fn set_global_params(&mut self, buffer_length: usize, sample_rate: usize) {
        if let Some(engine) = self.engine.as_mut() {
            engine.set_global_params(buffer_length, sample_rate)
        } else {
            self.silence.resize(buffer_length as usize * 2, 0.0);
        }
    }

    pub fn serialize_patch(&self) -> String {
        self.engine
            .as_ref()
            .map(|e| e.serialize_current_patch(&self.registry))
            .unwrap_or_default()
    }

    pub fn deserialize_patch(&mut self, serialized: &[u8]) {
        let patch = match registry::save_data::Patch::load_readable(
            "External Preset".to_owned(),
            serialized,
            &self.registry,
        ) {
            Ok(patch) => patch,
            Err(message) => {
                self.set_critical_error(format!(
                    "ERROR: Failed to load the patch you were working on, caused by:\n{}",
                    message
                ));
                return;
            }
        };
        let patch = util::rcrc(patch);
        if let Some(engine) = &mut self.engine {
            if let Err(message) = engine.load_patch(&self.registry, patch) {
                self.set_critical_error(format!(
                    "ERROR: Failed to load the patch you were working on, caused by:\n{}",
                    message
                ));
            }
        }
        if let Some(gui) = &mut self.gui {
            gui.on_patch_change(&self.registry);
        }
    }

    pub fn start_note(&mut self, index: usize, velocity: f32) {
        self.engine.as_mut().map(|e| e.start_note(index, velocity));
    }

    pub fn release_note(&mut self, index: usize) {
        self.engine.as_mut().map(|e| e.release_note(index));
    }

    pub fn set_pitch_wheel(&mut self, value: f32) {
        self.engine.as_mut().map(|e| e.set_pitch_wheel(value));
    }

    pub fn set_control(&mut self, index: usize, value: f32) {
        self.engine.as_mut().map(|e| e.set_control(index, value));
    }

    pub fn set_bpm(&mut self, bpm: f32) {
        self.engine.as_mut().map(|e| e.set_bpm(bpm));
    }

    pub fn set_elapsed_time(&mut self, time: f32) {
        self.engine.as_mut().map(|e| e.set_elapsed_time(time));
    }

    pub fn set_elapsed_beats(&mut self, beats: f32) {
        self.engine.as_mut().map(|e| e.set_elapsed_beats(beats));
    }

    pub fn render_audio(&mut self) -> Vec<f32> {
        if let Some(engine) = self.engine.as_mut() {
            engine.render_audio()
        } else {
            self.silence.clone()
        }
    }

    pub fn create_ui(&mut self) {
        if self.gui.is_some() {
            // This is an indicator of a bug in the frontend, but is not in itself a critical error,
            // so we shouldn't panic in release builds.
            debug_assert!(false, "create_gui called when GUI was already created!");
            eprintln!("WARNING: create_gui called when GUI was already created!");
        } else if let Some(engine) = self.engine.as_ref() {
            let graph = util::Rc::clone(engine.borrow_module_graph_ref());
            self.gui = Some(Gui::new(
                &self.registry,
                engine.borrow_current_patch(),
                graph,
            ));
        }
    }

    pub fn draw_ui(&mut self, data: *mut i8, icon_store: *mut i8) {
        let mut g = GrahpicsWrapper::new(&self.graphics_fns, data, icon_store);
        if let Some(err) = &self.critical_error {
            g.set_color(&(0, 0, 0));
            g.fill_rect(0.0, 0.0, 640.0, 480.0);
            g.set_color(&(255, 255, 255));
            g.write_console_text(640.0, 480.0, err);
        // If there is no critical error, then the engine initialized successfully.
        } else if let Some(err) = self.engine.as_ref().unwrap().clone_critical_error() {
            g.set_color(&(0, 0, 0));
            g.fill_rect(0.0, 0.0, 640.0, 480.0);
            g.set_color(&(255, 255, 255));
            g.write_console_text(640.0, 480.0, &err);
            // This way we don't have to copy it in the future.
            self.set_critical_error(err.clone());
        } else {
            // If the engine has new feedback data (from audio being played) then copy it over before
            // we render the UI so it will show up in the UI.
            let engine_ref = self.engine.as_mut().unwrap();
            engine_ref.display_new_feedback_data();
            let is_compiling = engine_ref.is_julia_thread_busy();
            g.set_color(&gui::constants::COLOR_BG0);
            g.clear();
            if let Some(gui) = &mut self.gui {
                gui.draw(&mut g, &mut self.registry, is_compiling);
            } else {
                panic!("draw_ui called before GUI was created!");
            }
        }
    }

    pub fn destroy_ui(&mut self) {
        if self.gui.is_none() && self.critical_error.is_none() {
            // This is an indicator of a bug in the frontend, but is not in itself a critical error,
            // so we shouldn't panic in release builds.
            debug_assert!(false, "destroy_gui called when GUI was already destroyed!");
            eprintln!("WARNING: destroy_gui called when GUI was already destroyed!");
        } else {
            self.gui = None;
        }
    }

    pub fn mouse_down(&mut self, x: f32, y: f32, right_click: bool, shift: bool, precise: bool) {
        if let Some(gui) = &mut self.gui {
            let mods = MouseMods {
                right_click,
                shift,
                precise,
            };
            let action = gui.on_mouse_down(&self.registry, (x, y), &mods);
            // This is a pretty hacky way of keeping the error on the screen but it works.
            if let Some(err) = &self.structure_error {
                gui.display_error(err.to_owned());
            }
            action.map(|a| self.perform_action(a));
        } else if self.critical_error.is_none() {
            debug_assert!(false, "mouse_down called, but no GUI exists.");
            eprintln!("WARNING: mouse_down called, but no GUI exists.");
        }
    }

    pub fn mouse_move(&mut self, x: f32, y: f32, right_click: bool, shift: bool, precise: bool) {
        if let Some(gui) = &mut self.gui {
            let mods = MouseMods {
                right_click,
                shift,
                precise,
            };
            let action = gui.on_mouse_move(&self.registry, (x, y), &mods);
            action.map(|a| self.perform_action(a));
        } else if self.critical_error.is_none() {
            debug_assert!(false, "mouse_move called, but no GUI exists.");
            eprintln!("WARNING: mouse_move called, but no GUI exists.");
        }
    }

    pub fn mouse_up(&mut self) {
        if let Some(gui) = &mut self.gui {
            let action = gui.on_mouse_up(&self.registry);
            action.map(|a| self.perform_action(a));
        } else if self.critical_error.is_none() {
            debug_assert!(false, "mouse_up called, but no GUI exists.");
            eprintln!("WARNING: mouse_up called, but no GUI exists.");
        }
    }

    pub fn scroll(&mut self, delta: f32) {
        if let Some(gui) = &mut self.gui {
            let action = gui.on_scroll(&self.registry, delta);
            action.map(|a| self.perform_action(a));
        } else if self.critical_error.is_none() {
            debug_assert!(false, "mouse_up called, but no GUI exists.");
            eprintln!("WARNING: mouse_up called, but no GUI exists.");
        }
    }

    pub fn key_press(&mut self, key: u8) {
        if let Some(gui) = &mut self.gui {
            let action = gui.on_key_press(&self.registry, key);
            action.map(|a| self.perform_action(a));
        } else if self.critical_error.is_none() {
            debug_assert!(false, "mouse_up called, but no GUI exists.");
            eprintln!("WARNING: mouse_up called, but no GUI exists.");
        }
    }
}
