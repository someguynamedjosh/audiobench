mod graphics;
mod util;
mod widgets;

use graphics::constants::*;
use graphics::{GrahpicsWrapper, GraphicsFunctions};
use widgets::Widget;

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
        let mut m = widgets::Module::default();
        let mut k = widgets::Knob::default();

        k.value = 0.5;
        k.automation.push((-1.0, 0.2));
        k.automation.push((0.0, 0.8));
        k.label = "volume".to_owned();
        k.pos = (coord(0), coord(0));

        let mut k2 = k.clone();
        k2.pos.0 = coord(2);

        m.num_inputs = 2;
        m.num_outputs = 1;
        m.size.0 = fatgrid(4);
        m.adopt_child(k);
        m.adopt_child(k2);

        let mut graph = widgets::ModuleGraph::default();
        graph.adopt_child(m);
        graph.offset = (30, 30);

        graph.draw(&mut g);
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
