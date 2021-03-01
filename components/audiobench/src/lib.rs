pub(crate) mod config;
mod engine;
mod gui;
mod registry;
mod scui_config;

use engine::{AudioThreadEngine, UiThreadEngine};
use gui::graphics::GrahpicsWrapper;
pub use gui::graphics::GraphicsFunctions;
use gui::Gui;
use registry::Registry;
use shared_util::prelude::*;

pub struct ErrorDrawer {
    error: String,
    pub graphics_fns: Rc<GraphicsFunctions>,
}

impl ErrorDrawer {
    pub fn new(error: String) -> Self {
        let graphics_fns = Rc::new(GraphicsFunctions::placeholders());
        use clipboard::ClipboardProvider;
        let mut clipboard: clipboard::ClipboardContext =
            clipboard::ClipboardProvider::new().unwrap();
        clipboard.set_contents(error.clone()).unwrap();
        let error = format!(
            "The following error has been copied to your clipboard:\n\n{}",
            error
        );
        Self {
            error,
            graphics_fns,
        }
    }

    pub fn draw(&self, data: *mut i8, icon_store: *mut i8) {
        let mut g = GrahpicsWrapper::new(Rc::clone(&self.graphics_fns), data, icon_store);
        g.set_color(&(0, 0, 0));
        g.draw_rect((0, 0), (640, 480));
        g.set_color(&(255, 255, 255));
        g.draw_console_text((640, 480), &self.error);
    }
}

pub struct Instance {
    pub registry: Rcrc<Registry>,
    pub ui_engine: Rcrc<UiThreadEngine>,
    pub audio_engine: Rcrc<AudioThreadEngine>,
    pub graphics_fns: Rc<GraphicsFunctions>,
    pub gui: Option<Gui>,
    audio: Vec<f32>,
}

impl Instance {
    pub fn new() -> Result<Self, String> {
        let registry = rcrc(Registry::new()?);
        let (ui_engine, audio_engine) = engine::new_engine(Rc::clone(&registry))?;
        let graphics_fns = Rc::new(GraphicsFunctions::placeholders());
        Ok(Self {
            registry,
            ui_engine,
            audio_engine,
            graphics_fns,
            gui: None,
            audio: Vec::new(),
        })
    }

    pub fn ui_deserialize_patch(&mut self, serialized: &[u8]) -> Result<(), ()> {
        let registry = self.registry.borrow();
        let patch = match registry::save_data::Patch::load_readable(
            "External Preset".to_owned(),
            serialized,
        ) {
            Ok(patch) => patch,
            Err(message) => {
                drop(registry);
                self.ui_engine.borrow_mut().post_error(format!(
                    "ERROR: Failed to load the patch you were working on, caused by:\n{}",
                    message
                ));
                return Err(());
            }
        };
        drop(registry);
        let patch = rcrc(patch);
        self.ui_engine.borrow_mut().load_patch(patch)?;
        Ok(())
    }

    pub fn audio_render_audio(&mut self) -> &[f32] {
        self.audio = self.audio_engine.borrow_mut().render_audio();
        &self.audio[..]
    }

    pub fn ui_with_gui_mut(&mut self, op: impl FnOnce(&mut Gui)) {
        if let Some(gui) = &mut self.gui {
            op(gui);
        } else {
            debug_assert!(false, "with_gui_mut called, but no GUI exists.");
            eprintln!("with_gui_mut called, but no GUI exists.");
        }
    }

    pub fn ui_create_ui(&mut self) {
        if self.gui.is_some() {
            // This is an indicator of a bug in the frontend, but is not in itself a critical error,
            // so we shouldn't panic in release builds.
            debug_assert!(false, "create_gui called when GUI was already created!");
            eprintln!("WARNING: create_gui called when GUI was already created!");
        } else {
            self.gui = Some(gui::new_gui(
                Rc::clone(&self.registry),
                Rc::clone(&self.ui_engine),
            ));
        }
    }

    pub fn ui_draw_ui(&mut self, data: *mut i8, icon_store: *mut i8) {
        if let Some(gui) = &mut self.gui {
            let mut g = GrahpicsWrapper::new(Rc::clone(&self.graphics_fns), data, icon_store);
            let mut ui_engine = self.ui_engine.borrow_mut();
            ui_engine.display_new_feedback_data();
            drop(ui_engine);
            g.set_color(&gui::constants::COLOR_BG0);
            g.clear();
            gui.draw(&mut g);
        } else {
            debug_assert!(false, "ui_draw_ui called without creating a GUI!");
            eprintln!("ui_draw_ui called without creating a GUI!");
        }
        // g.set_color(&(0, 0, 0));
        // g.draw_rect((0, 0), (640, 480));
        // g.set_color(&(255, 255, 255));
        // g.draw_console_text((640, 480), &err);
        // drop(uengine);
    }

    pub fn ui_destroy_ui(&mut self) {
        if self.gui.is_none() {
            // This is an indicator of a bug in the frontend, but is not in itself a critical error,
            // so we shouldn't panic in release builds.
            debug_assert!(false, "destroy_gui called when GUI was already destroyed!");
            eprintln!("WARNING: destroy_gui called when GUI was already destroyed!");
        } else {
            self.gui = None;
        }
    }
}
