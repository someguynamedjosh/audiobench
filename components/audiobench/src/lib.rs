pub(crate) mod config;
mod engine;
mod gui;
mod registry;
mod scui_config;

use crossbeam_channel::{Receiver, Sender, TryRecvError};
use engine::{AudioThreadEngine, UiThreadEngine};
use gui::graphics::GrahpicsWrapper;
pub use gui::graphics::GraphicsFunctions;
use gui::Gui;
use registry::{save_data::Patch, Registry};
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

    ui_request_pipe: Receiver<CrossThreadHelpRequest>,
    ui_response_pipe: Sender<CrossThreadHelpResponse>,
    audio_request_pipe: Sender<CrossThreadHelpRequest>,
    audio_response_pipe: Receiver<CrossThreadHelpResponse>,
    vk_offset: i32,
}

/// This is ugly and disgusting but AFAIK it is sound. Basically we need to be able to save and
/// load patches from the audio thread but that logic is handled by the UI thread but that's not
/// always running so when it's running we need to have it handle that logic through channels and
/// when it's not running the audio thread handles it immediately.

enum CrossThreadHelpRequest {
    SerializePatch,
    DeserializePatch(Vec<u8>),
}

enum CrossThreadHelpResponse {
    SerializePatch(String),
    DeserializePatch(Result<(), ()>),
}

impl Instance {
    pub fn new() -> Result<Self, String> {
        observatory::init();
        let registry = rcrc(Registry::new()?);
        let (ui_engine, audio_engine) = engine::new_engine(Rc::clone(&registry))?;
        let graphics_fns = Rc::new(GraphicsFunctions::placeholders());
        let (audio_request_pipe, ui_request_pipe) = crossbeam_channel::bounded(1);
        let (ui_response_pipe, audio_response_pipe) = crossbeam_channel::bounded(1);
        Ok(Self {
            registry,
            ui_engine,
            audio_engine,
            graphics_fns,
            gui: None,
            audio: Vec::new(),

            ui_request_pipe,
            ui_response_pipe,
            audio_request_pipe,
            audio_response_pipe,
            vk_offset: 0,
        })
    }

    fn serialize_patch(&mut self) -> String {
        self.ui_engine.borrow_mut().serialize_current_patch()
    }

    fn deserialize_patch(&mut self, serialized: &[u8]) -> Result<(), ()> {
        let registry = self.registry.borrow();
        let deserialized = match Patch::load_readable("dummy".into(), serialized) {
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
        let mut patch = None;
        for other_ptr in registry.borrow_patches() {
            let other = other_ptr.borrow();
            if deserialized.borrow_name() == other.borrow_name()
                && deserialized.serialize() == other.serialize()
            {
                patch = Some(Rc::clone(other_ptr));
                break;
            }
        }
        drop(registry);
        if patch.is_none() {
            let new_patch = Rc::clone(self.registry.borrow_mut().create_new_user_patch());
            new_patch.borrow_mut().deserialize(serialized).unwrap();
            patch = Some(new_patch);
        }
        self.ui_engine.borrow_mut().load_patch(patch.unwrap())?;
        Ok(())
    }

    // This should be regularly called as long as the UI is open.
    pub fn ui_handle_cross_thread_help(&mut self) {
        match self.ui_request_pipe.try_recv() {
            Ok(CrossThreadHelpRequest::DeserializePatch(data)) => {
                let res = self.ui_deserialize_patch(&data[..]);
                self.ui_response_pipe
                    .send(CrossThreadHelpResponse::DeserializePatch(res))
                    .unwrap();
            }
            Ok(CrossThreadHelpRequest::SerializePatch) => {
                println!("received from gui");
                let res = self.ui_serialize_patch();
                self.ui_response_pipe
                    .send(CrossThreadHelpResponse::SerializePatch(res))
                    .unwrap();
            }
            Err(TryRecvError::Empty) => (),
            Err(err) => panic!("Unexpected error {}", err),
        }
    }

    pub fn ui_serialize_patch(&mut self) -> String {
        self.serialize_patch()
    }

    pub fn audio_serialize_patch(&mut self) -> String {
        if self.gui.is_some() {
            println!("send to gui");
            self.audio_request_pipe
                .send(CrossThreadHelpRequest::SerializePatch)
                .unwrap();
            if let CrossThreadHelpResponse::SerializePatch(res) =
                self.audio_response_pipe.recv().unwrap()
            {
                res
            } else {
                panic!("Unexpected response");
            }
        } else {
            println!("do instantly");
            self.serialize_patch()
        }
    }

    pub fn ui_deserialize_patch(&mut self, serialized: &[u8]) -> Result<(), ()> {
        self.deserialize_patch(serialized)
    }

    pub fn audio_deserialize_patch(&mut self, serialized: &[u8]) -> Result<(), ()> {
        if self.gui.is_some() {
            self.audio_request_pipe
                .send(CrossThreadHelpRequest::DeserializePatch(Vec::from(
                    serialized,
                )))
                .unwrap();
            if let CrossThreadHelpResponse::DeserializePatch(res) =
                self.audio_response_pipe.recv().unwrap()
            {
                res
            } else {
                panic!("Unexpected response");
            }
        } else {
            self.deserialize_patch(serialized)
        }
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

    fn ui_vk_index(&self, key: char) -> Option<usize> {
        // q == 45 == A3
        "q2we4r5ty7u8i9op"
            .chars()
            .position(|candidate| candidate == key)
            .map(|idx| idx as i32 + self.vk_offset + 45)
            .map(|idx| {
                if idx >= 0 && idx < 128 {
                    Some(idx as usize)
                } else {
                    None
                }
            })
            .flatten()
    }

    pub fn ui_vk_down(&mut self, key: char) {
        if let Some(index) = self.ui_vk_index(key) {
            self.ui_engine
                .borrow_mut()
                .virtual_keyboard_note(index, true);
        } else if key == 'z' {
            self.vk_offset -= 12;
        } else if key == 'x' {
            self.vk_offset += 12;
        }
    }

    pub fn ui_vk_up(&mut self, key: char) {
        if let Some(index) = self.ui_vk_index(key) {
            self.ui_engine
                .borrow_mut()
                .virtual_keyboard_note(index, false);
        }
    }
}
