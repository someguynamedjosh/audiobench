mod engine;
mod gui;
mod util;

use gui::graphics::{GrahpicsWrapper, GraphicsFunctions};
use gui::Gui;

pub struct Instance {
    graphics_fns: GraphicsFunctions,
    gui: Option<Gui>,
    engine: engine::Engine,
}

impl Instance {
    fn new() -> Self {
        let (engine, setup_status) = engine::Engine::new();

        Self {
            graphics_fns: GraphicsFunctions::placeholders(),
            gui: None,
            engine,
        }
    }
}

impl Instance {
    pub fn render_audio(&mut self) -> &[f32] {
        self.engine.render_audio()
    }

    pub fn create_ui(&mut self) {
        if self.gui.is_some() {
            // This is an indicator of a bug in the frontend, but is not in itself a critical error,
            // so we shouldn't panic in release builds.
            debug_assert!(false, "create_gui called when GUI was already created!");
            eprintln!("WARNING: create_gui called when GUI was already created!");
        } else {
            self.gui = Some(Gui::new(engine::parts::ModuleGraph::build_gui(util::Rc::clone(
                self.engine.borrow_module_graph_ref(),
            ))));
        }
    }

    pub fn draw_ui(&self, data: *mut i8) {
        let mut g = GrahpicsWrapper::new(&self.graphics_fns, data);
        g.set_color(&gui::constants::COLOR_BG);
        g.clear();
        if let Some(gui) = &self.gui {
            gui.draw(&mut g);
        } else {
            panic!("draw_ui called before GUI was created!");
        }
    }

    pub fn destroy_ui(&mut self) {
        if self.gui.is_none() {
            // This is an indicator of a bug in the frontend, but is not in itself a critical error,
            // so we shouldn't panic in release builds.
            debug_assert!(false, "destroy_gui called when GUI was already destroyed!");
            eprintln!("WARNING: destroy_gui called when GUI was already destroyed!");
        } else {
            self.gui = None;
        }
    }

    pub fn mouse_down(&mut self, x: i32, y: i32) {
        if let Some(gui) = &mut self.gui {
            gui.on_mouse_down((x, y));
        } else {
            debug_assert!(false, "mouse_down called, but no GUI exists.");
            eprintln!("WARNING: mouse_down called, but no GUI exists.");
        }
    }

    pub fn mouse_move(&mut self, x: i32, y: i32) {
        if let Some(gui) = &mut self.gui {
            gui.on_mouse_move((x, y));
        } else {
            debug_assert!(false, "mouse_move called, but no GUI exists.");
            eprintln!("WARNING: mouse_move called, but no GUI exists.");
        }
    }

    pub fn mouse_up(&mut self) {
        if let Some(gui) = &mut self.gui {
            gui.on_mouse_up();
        } else {
            debug_assert!(false, "mouse_up called, but no GUI exists.");
            eprintln!("WARNING: mouse_up called, but no GUI exists.");
        }
        // TODO: Make a more robust solution for this.
        self.engine.mark_module_graph_dirty();
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
pub unsafe extern "C" fn ABDrawUI(instance: *mut Instance, graphics_data: *mut i8) {
    (*instance).draw_ui(graphics_data);
}

#[no_mangle]
pub unsafe extern "C" fn ABDestroyUI(instance: *mut Instance) {
    (*instance).destroy_ui();
}

#[no_mangle]
pub unsafe extern "C" fn ABUIMouseDown(instance: *mut Instance, x: i32, y: i32) {
    (*instance).mouse_down(x, y);
}

#[no_mangle]
pub unsafe extern "C" fn ABUIMouseMove(instance: *mut Instance, x: i32, y: i32) {
    (*instance).mouse_move(x, y);
}

#[no_mangle]
pub unsafe extern "C" fn ABUIMouseUp(instance: *mut Instance) {
    (*instance).mouse_up();
}
