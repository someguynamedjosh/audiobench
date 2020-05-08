#[repr(C)]
pub struct GraphicsFunctions {
    set_color: fn(*mut i8, u8, u8, u8),
    clear: fn(*mut i8),
    fill_rect: fn(*mut i8, i32, i32, i32, i32),
    fill_pie: fn(*mut i8, i32, i32, i32, i32, f32, f32),
}

impl GraphicsFunctions {
    fn placeholders() -> Self {
        fn set_color(_data: *mut i8, _r: u8, _g: u8, _b: u8) {
            panic!("ERROR: Graphics functions not set by frontend!");
        }
        fn clear(_data: *mut i8) {
            panic!("ERROR: Graphics functions not set by frontend!");
        }
        fn fill_rect(_data: *mut i8, _x: i32, _y: i32, _w: i32, _h: i32) {
            panic!("ERROR: Graphics functions not set by frontend!");
        }
        fn fill_pie(_data: *mut i8, _x: i32, _y: i32, _r: i32, _ir: i32, _sr: f32, _er: f32) {
            panic!("ERROR: Graphics functions not set by frontend!");
        }
        Self {
            set_color,
            clear,
            fill_rect,
            fill_pie,
        }
    }
}

struct GrahpicsWrapper<'a> {
    graphics_fns: &'a GraphicsFunctions,
    aux_data: *mut i8,
}

impl<'a> GrahpicsWrapper<'a> {
    pub fn new(graphics_fns: &GraphicsFunctions, aux_data: *mut i8) -> GrahpicsWrapper {
        GrahpicsWrapper {
            graphics_fns,
            aux_data,
        }
    }

    pub fn set_color(&mut self, color: &(u8, u8, u8)) {
        (self.graphics_fns.set_color)(self.aux_data, color.0, color.1, color.2);
    }

    pub fn clear(&mut self) {
        (self.graphics_fns.clear)(self.aux_data);
    }

    pub fn fill_rect(&mut self, x: i32, y: i32, w: i32, h: i32) {
        (self.graphics_fns.fill_rect)(self.aux_data, x, y, w, h);
    }

    pub fn fill_pie(
        &mut self,
        x: i32,
        y: i32,
        radius: i32,
        inner_radius: i32,
        start_rad: f32,
        end_rad: f32,
    ) {
        (self.graphics_fns.fill_pie)(
            self.aux_data,
            x,
            y,
            radius,
            inner_radius,
            start_rad,
            end_rad,
        );
    }
}

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
        g.set_color(&(255, 0, 255));
        g.clear();
        g.set_color(&(255, 255, 255));
        g.fill_rect(0, 0, 10, 10);
        g.fill_pie(100, 100, 50, 30, 0.0, std::f32::consts::PI);
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
