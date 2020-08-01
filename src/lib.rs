mod engine;
mod gui;
mod registry;
mod util;

use gui::graphics::{GrahpicsWrapper, GraphicsFunctions};
use gui::{Gui, MouseMods};

pub struct Instance {
    engine: Option<engine::Engine>,
    registry: registry::Registry,
    graphics_fns: GraphicsFunctions,
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
    fn new() -> Self {
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
            gui::action::InstanceAction::ReloadAuxData => {
                self.engine.as_mut().map(|e| e.reload_aux_data());
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
            self.engine.as_mut().unwrap().display_new_feedback_data();
            g.set_color(&gui::constants::COLOR_BG);
            g.clear();
            if let Some(gui) = &mut self.gui {
                gui.draw(&mut g, &mut self.registry);
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

#[no_mangle]
pub unsafe extern "C" fn ABCreateInstance() -> *mut Instance {
    Box::into_raw(Box::new(Instance::new()))
}

#[no_mangle]
pub unsafe extern "C" fn ABDestroyInstance(instance: *mut Instance) {
    let data = Box::from_raw(instance);
    drop(data);
}

#[no_mangle]
pub unsafe extern "C" fn ABGetNumIcons(instance: *mut Instance) -> i32 {
    (*instance).get_num_icons() as i32
}

#[no_mangle]
pub unsafe extern "C" fn ABGetIconData(
    instance: *mut Instance,
    icon_index: i32,
    data_buffer: *mut *const u8,
    data_length: *mut i32,
) {
    let svg_data = (*instance).borrow_icon_data(icon_index as usize);
    (*data_buffer) = svg_data.as_ptr();
    (*data_length) = svg_data.len() as i32;
}

#[no_mangle]
pub unsafe extern "C" fn ABSetHostFormat(
    instance: *mut Instance,
    buffer_length: i32,
    sample_rate: i32,
) {
    (*instance).set_host_format(buffer_length as usize, sample_rate as usize)
}

#[no_mangle]
pub unsafe extern "C" fn ABSerializePatch(
    instance: *mut Instance,
    data_out: *mut *mut u8,
    size_out: *mut u32,
) {
    let data = (*instance)
        .serialize_patch()
        .into_bytes()
        .into_boxed_slice();
    *size_out = data.len() as u32;
    *data_out = Box::leak(data).as_mut_ptr();
}

#[no_mangle]
pub unsafe extern "C" fn ABCleanupSerializedData(data: *mut u8, size: u32) {
    let slice = std::slice::from_raw_parts_mut(data, size as usize);
    let boxed = Box::from_raw(slice);
    drop(boxed);
}

#[no_mangle]
pub unsafe extern "C" fn ABDeserializePatch(
    instance: *mut Instance,
    data_in: *mut u8,
    size_in: u32,
) {
    let data = std::slice::from_raw_parts(data_in, size_in as usize);
    let data = Vec::from(data);
    (*instance).deserialize_patch(&data[..]);
}

#[no_mangle]
pub unsafe extern "C" fn ABStartNote(instance: *mut Instance, index: i32, velocity: f32) {
    (*instance).start_note(index as usize, velocity)
}

#[no_mangle]
pub unsafe extern "C" fn ABReleaseNote(instance: *mut Instance, index: i32) {
    (*instance).release_note(index as usize)
}

#[no_mangle]
pub unsafe extern "C" fn ABPitchWheel(instance: *mut Instance, value: f32) {
    (*instance).set_pitch_wheel(value)
}

#[no_mangle]
pub unsafe extern "C" fn ABControl(instance: *mut Instance, index: i32, value: f32) {
    (*instance).set_control(index as usize, value)
}

#[no_mangle]
pub unsafe extern "C" fn ABBpm(instance: *mut Instance, bpm: f32) {
    (*instance).set_bpm(bpm)
}

#[no_mangle]
pub unsafe extern "C" fn ABSongTime(instance: *mut Instance, time: f32) {
    (*instance).set_song_time(time)
}

#[no_mangle]
pub unsafe extern "C" fn ABSongBeats(instance: *mut Instance, beats: f32) {
    (*instance).set_song_beats(beats)
}

#[no_mangle]
pub unsafe extern "C" fn ABRenderAudio(instance: *mut Instance) -> *const f32 {
    (*instance).render_audio().as_ptr()
}

#[no_mangle]
pub unsafe extern "C" fn ABSetGraphicsFunctions(
    instance: *mut Instance,
    graphics_fns: GraphicsFunctions,
) {
    (*instance).graphics_fns = graphics_fns;
}

#[no_mangle]
pub unsafe extern "C" fn ABCreateUI(instance: *mut Instance) {
    (*instance).create_ui();
}

#[no_mangle]
pub unsafe extern "C" fn ABDrawUI(
    instance: *mut Instance,
    graphics_data: *mut i8,
    icon_store: *mut i8,
) {
    (*instance).draw_ui(graphics_data, icon_store);
}

#[no_mangle]
pub unsafe extern "C" fn ABDestroyUI(instance: *mut Instance) {
    (*instance).destroy_ui();
}

#[no_mangle]
pub unsafe extern "C" fn ABUIMouseDown(
    instance: *mut Instance,
    x: f32,
    y: f32,
    right_click: bool,
    shift: bool,
    precise: bool,
) {
    (*instance).mouse_down(x, y, right_click, shift, precise);
}

#[no_mangle]
pub unsafe extern "C" fn ABUIMouseMove(
    instance: *mut Instance,
    x: f32,
    y: f32,
    right_click: bool,
    shift: bool,
    precise: bool,
) {
    // TOTO: I don't think we're in canvas anymore
    // TODO: Make ABI functions accept floats
    (*instance).mouse_move(x, y, right_click, shift, precise);
}

#[no_mangle]
pub unsafe extern "C" fn ABUIMouseUp(instance: *mut Instance) {
    (*instance).mouse_up();
}

#[no_mangle]
pub unsafe extern "C" fn ABUIScroll(instance: *mut Instance, delta: f32) {
    (*instance).scroll(delta);
}

#[no_mangle]
pub unsafe extern "C" fn ABUIKeyPress(instance: *mut Instance, key: u8) {
    (*instance).key_press(key);
}
