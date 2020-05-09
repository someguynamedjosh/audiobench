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

        m.num_inputs = 2;
        m.num_outputs = 1;
        m.w = fatgrid(4);

        k.value = 0.5;
        k.automation.push((-1.0, 0.2));
        k.automation.push((0.0, 0.8));
        k.label = "volume".to_owned();
        k.x = coord(0);
        k.y = coord(0);

        let mut k2 = k.clone();
        k2.x = coord(2);

        g.push_state();
        g.apply_offset(30, 30);

        m.draw(&mut g);
        m.apply_transform(&mut g);
        k.draw(&mut g);
        k2.draw(&mut g);

        g.pop_state();
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
