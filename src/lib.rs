#[repr(C)]
pub struct GraphicsFunctions {
    set_color: fn(*mut i8, u8, u8, u8),
    clear: fn(*mut i8),
    fill_rect: fn(*mut i8, i32, i32, i32, i32),
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
        Self {
            set_color,
            clear,
            fill_rect,
        }
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

    fn draw_interface(&self, data: *mut i8) {
        (self.graphics_fns.set_color)(data, 255, 0, 255);
        (self.graphics_fns.clear)(data);
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
