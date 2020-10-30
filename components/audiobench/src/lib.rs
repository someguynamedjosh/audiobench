pub(crate) mod config;
mod engine;
mod gui;
mod registry;
mod scui_config;

use gui::graphics::GrahpicsWrapper;
pub use gui::graphics::GraphicsFunctions;
use gui::Gui;
use scui::MouseMods;

use shared_util::prelude as util;

pub struct Instance {
    engine: Option<engine::Engine>,
    registry: registry::Registry,
    pub graphics_fns: util::Rc<GraphicsFunctions>,
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
            graphics_fns: util::Rc::new(GraphicsFunctions::placeholders()),
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
        self.structure_error = Some(error);
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

    pub fn set_host_format(&mut self, buffer_length: usize, sample_rate: usize) {
        if let Some(engine) = self.engine.as_mut() {
            engine.set_host_format(buffer_length, sample_rate)
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
            // gui.on_patch_change(&self.registry);
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

    pub fn set_song_time(&mut self, time: f32) {
        self.engine.as_mut().map(|e| e.set_song_time(time));
    }

    pub fn set_song_beats(&mut self, beats: f32) {
        self.engine.as_mut().map(|e| e.set_song_beats(beats));
    }

    pub fn render_audio(&mut self) -> &[f32] {
        if let Some(engine) = self.engine.as_mut() {
            engine.render_audio()
        } else {
            &self.silence[..]
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
            self.gui = Some(gui::new_gui());
        }
    }

    pub fn draw_ui(&mut self, data: *mut i8, icon_store: *mut i8) {
        let mut g = GrahpicsWrapper::new(util::Rc::clone(&self.graphics_fns), data, icon_store);
        if let Some(err) = &self.critical_error {
            g.set_color(&(0, 0, 0));
            g.draw_rect(0, (640, 480));
            g.set_color(&(255, 255, 255));
            g.draw_console_text((640, 480), err);
        // If there is no critical error, then the engine initialized successfully.
        } else if let Some(err) = self.engine.as_ref().unwrap().clone_critical_error() {
            g.set_color(&(0, 0, 0));
            g.draw_rect(0, (640, 480));
            g.set_color(&(255, 255, 255));
            g.draw_console_text((640, 480), &err);
            // This way we don't have to copy it in the future.
            self.set_critical_error(err.clone());
        } else {
            // If the engine has new feedback data (from audio being played) then copy it over before
            // we render the UI so it will show up in the UI.
            let engine_ref = self.engine.as_mut().unwrap();
            engine_ref.display_new_feedback_data();
            let is_compiling = engine_ref.is_currently_compiling();
            g.set_color(&gui::constants::COLOR_BG0);
            g.clear();
            if let Some(gui) = &mut self.gui {
                gui.draw(&mut g);
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

    pub fn mouse_down(&mut self, x: f32, y: f32, right_click: bool, snap: bool, precise: bool) {
        if let Some(gui) = &mut self.gui {
            let mods = MouseMods {
                right_click,
                snap,
                precise,
            };
            let requests = gui.on_mouse_down(&mods);
            // This is a pretty hacky way of keeping the error on the screen but it works.
            if let Some(err) = &self.structure_error {
                // gui.display_error(err.to_owned());
            }
        } else if self.critical_error.is_none() {
            debug_assert!(false, "mouse_down called, but no GUI exists.");
            eprintln!("WARNING: mouse_down called, but no GUI exists.");
        }
    }

    pub fn mouse_move(&mut self, x: f32, y: f32, right_click: bool, snap: bool, precise: bool) {
        if let Some(gui) = &mut self.gui {
            let mods = MouseMods {
                right_click,
                snap,
                precise,
            };
            gui.on_mouse_move((x, y).into(), &mods);
        } else if self.critical_error.is_none() {
            debug_assert!(false, "mouse_move called, but no GUI exists.");
            eprintln!("WARNING: mouse_move called, but no GUI exists.");
        }
    }

    pub fn mouse_up(&mut self) {
        if let Some(gui) = &mut self.gui {
            let requests = gui.on_mouse_up();
        } else if self.critical_error.is_none() {
            debug_assert!(false, "mouse_up called, but no GUI exists.");
            eprintln!("WARNING: mouse_up called, but no GUI exists.");
        }
    }

    pub fn scroll(&mut self, delta: f32) {
        if let Some(gui) = &mut self.gui {
            gui.on_scroll(delta);
        } else if self.critical_error.is_none() {
            debug_assert!(false, "mouse_up called, but no GUI exists.");
            eprintln!("WARNING: mouse_up called, but no GUI exists.");
        }
    }

    pub fn key_press(&mut self, key: char) {
        if let Some(gui) = &mut self.gui {
            gui.on_key_press(key);
        } else if self.critical_error.is_none() {
            debug_assert!(false, "mouse_up called, but no GUI exists.");
            eprintln!("WARNING: mouse_up called, but no GUI exists.");
        }
    }
}
