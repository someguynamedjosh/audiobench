mod graphics;

use graphics::constants::*;
use graphics::{GrahpicsWrapper, GraphicsFunctions};
use std::f32::consts::PI;

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

        g.set_color(&COLOR_BG);
        g.clear();
        g.set_color(&COLOR_SURFACE);
        g.fill_rect(fatcoord(0), fatcoord(0), FATGRID_2, FATGRID_2);
        g.set_color(&COLOR_BG);
        g.fill_pie(coord(0), coord(0), GRID_2, 0, 0.0, PI);

        g.set_color(&COLOR_KNOB);
        let primary_angle = PI * (1.0 - 0.3);
        g.fill_pie(coord(0), coord(0), GRID_2, 0, primary_angle, PI);

        let lane_size = (GRID_2 / 2 - KNOB_INSIDE_SPACE) / 3;
        for (index, value) in &[(0, 0.2), (1, 0.6), (2, 0.1)] {
            let outer_diameter = GRID_2 - lane_size * index * 2;
            let inner_diameter = outer_diameter - lane_size * 2;
            let inset = (GRID_2 - outer_diameter) / 2;
            g.set_color(&COLOR_AUTOMATION[*index as usize]);
            g.fill_pie(
                coord(0) + inset,
                coord(0) + inset,
                outer_diameter,
                inner_diameter,
                PI * (1.0f32 - value),
                PI,
            );
        }

        g.set_color(&COLOR_TEXT);
        g.write_label(coord(0), coord(1), GRID_2, "AMPLITUDE LINE2");
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
