mod engine;
mod gui;
mod util;

use gui::graphics::{GrahpicsWrapper, GraphicsFunctions};
use gui::{Gui, MouseMods};

pub struct Instance {
    engine: Option<engine::Engine>,
    registry: engine::registry::Registry,
    graphics_fns: GraphicsFunctions,
    gui: Option<Gui>,
    critical_error: Option<String>,
    structure_error: Option<String>,
    silence: Vec<f32>,
}

impl Instance {
    fn new() -> Self {
        let mut critical_error = None;

        let (mut registry, registry_load_result) = engine::registry::Registry::new();
        if let Err(err) = registry_load_result {
            critical_error = Some(format!(
                "Encountered a critical error while loading your libraries:\n{}",
                err
            ));
        }
        let engine = if critical_error.is_none() {
            match engine::Engine::new(&mut registry) {
                Ok(engine) => Some(engine),
                Err(problem) => {
                    critical_error = Some(problem);
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
    fn perform_action(&mut self, action: gui::action::InstanceAction) {
        match action {
            gui::action::InstanceAction::ReloadAuxData => {
                if self.structure_error.is_none() {
                    self.engine.as_mut().unwrap().reload_values();
                }
            }
            gui::action::InstanceAction::ReloadStructure => {
                let res = self.engine.as_mut().unwrap().reload_structure();
                if let Err(err) = res {
                    if let Some(gui) = &mut self.gui {
                        gui.display_error(err.clone());
                    }
                    self.structure_error = Some(err);
                } else {
                    if let Some(gui) = &mut self.gui {
                        gui.clear_status();
                    }
                    self.structure_error = None;
                }
            }
            gui::action::InstanceAction::RenamePatch(name) => {
                self.engine.as_mut().unwrap().rename_current_patch(name);
            }
            gui::action::InstanceAction::SavePatch => {
                self.engine.as_mut().unwrap().save_current_patch();
                if let Some(gui) = &mut self.gui {
                    gui.display_success("Saved successfully!".to_owned());
                }
            }
            gui::action::InstanceAction::NewPatch(callback) => {
                callback(self.engine.as_mut().unwrap().new_patch(&mut self.registry));
            }
            gui::action::InstanceAction::LoadPatch(patch) => {
                let res = self
                    .engine
                    .as_mut()
                    .unwrap()
                    .load_patch(&self.registry, patch);
                if let Some(gui) = &mut self.gui {
                    if let Err(err) = res {
                        gui.display_error(err);
                    }
                    gui.on_patch_change(&self.registry);
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

    pub fn set_buffer_length_and_sample_rate(&mut self, buffer_length: i32, sample_rate: i32) {
        if let Some(engine) = self.engine.as_mut() {
            engine.set_buffer_length_and_sample_rate(buffer_length, sample_rate)
        } else {
            self.silence.resize(buffer_length as usize * 2, 0.0);
        }
    }

    pub fn note_on(&mut self, index: i32, velocity: f32) {
        self.engine.as_mut().map(|e| e.note_on(index, velocity));
    }

    pub fn note_off(&mut self, index: i32) {
        self.engine.as_mut().map(|e| e.note_off(index));
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
            // This way we don't have to copy it in the future.
            self.critical_error = Some(err.clone());
            g.set_color(&(0, 0, 0));
            g.fill_rect(0.0, 0.0, 640.0, 480.0);
            g.set_color(&(255, 255, 255));
            g.write_console_text(640.0, 480.0, &err);
        } else {
            // If the engine has new feedback data (from audio being played) then copy it over before
            // we render the UI so it will show up in the UI.
            self.engine.as_mut().unwrap().display_new_feedback_data();
            g.set_color(&gui::constants::COLOR_BG);
            g.clear();
            if let Some(gui) = &self.gui {
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
pub unsafe extern "C" fn ABSetBufferLengthAndSampleRate(
    instance: *mut Instance,
    buffer_length: i32,
    sample_rate: i32,
) {
    (*instance).set_buffer_length_and_sample_rate(buffer_length, sample_rate)
}

#[no_mangle]
pub unsafe extern "C" fn ABNoteOn(instance: *mut Instance, index: i32, velocity: f32) {
    (*instance).note_on(index, velocity)
}

#[no_mangle]
pub unsafe extern "C" fn ABNoteOff(instance: *mut Instance, index: i32) {
    (*instance).note_off(index)
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
    x: i32,
    y: i32,
    right_click: bool,
    shift: bool,
    precise: bool,
) {
    (*instance).mouse_down(x as f32, y as f32, right_click, shift, precise);
}

#[no_mangle]
pub unsafe extern "C" fn ABUIMouseMove(
    instance: *mut Instance,
    x: i32,
    y: i32,
    right_click: bool,
    shift: bool,
    precise: bool,
) {
    // TOTO: I don't think we're in canvas anymore
    // TODO: Make ABI functions accept floats
    (*instance).mouse_move(x as f32, y as f32, right_click, shift, precise);
}

#[no_mangle]
pub unsafe extern "C" fn ABUIMouseUp(instance: *mut Instance) {
    (*instance).mouse_up();
}

#[no_mangle]
pub unsafe extern "C" fn ABUIKeyPress(instance: *mut Instance, key: u8) {
    (*instance).key_press(key);
}
