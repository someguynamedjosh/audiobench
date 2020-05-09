mod engine;
mod gui;
mod util;

use gui::graphics::{GrahpicsWrapper, GraphicsFunctions};
use gui::widgets::Widget;

pub struct Instance {
    graphics_fns: GraphicsFunctions,
}

impl Instance {
    fn new() -> Self {
        println!("Created!");
        Self {
            graphics_fns: GraphicsFunctions::placeholders(),
        }
    }
}

impl Instance {
    pub fn draw_interface(&self, data: *mut i8) {
        let mut g = GrahpicsWrapper::new(&self.graphics_fns, data);

        let module_prototype = engine::Module::example();
        let mut module_graph = engine::ModuleGraph::new();

        let mut inst = module_prototype.clone();
        inst.pos = (10, 5);
        module_graph.adopt_module(inst);

        let mut inst = module_prototype.clone();
        inst.pos = (20, 100);
        module_graph.adopt_module(inst);

        let module_graph = util::rcrc(module_graph);
        let gui = engine::ModuleGraph::build_gui(module_graph);
        g.set_color(&gui::constants::COLOR_BG);
        g.clear();
        gui.draw(&mut g);
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
pub unsafe extern "C" fn ABSetGraphicsFunctions(
    instance: *mut Instance,
    graphics_fns: GraphicsFunctions,
) {
    (*instance).graphics_fns = graphics_fns;
}

#[no_mangle]
pub unsafe extern "C" fn ABDrawUI(instance: *mut Instance, graphics_data: *mut i8) {
    (*instance).draw_interface(graphics_data);
}
