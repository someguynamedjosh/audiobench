#[repr(C)]
pub struct GraphicsFunctions {
    push_state: fn(*mut i8),
    pop_state: fn(*mut i8),
    apply_offset: fn(*mut i8, i32, i32),

    set_color: fn(*mut i8, u8, u8, u8),
    set_alpha: fn(*mut i8, f32),
    clear: fn(*mut i8),
    stroke_line: fn(*mut i8, i32, i32, i32, i32, f32),
    fill_rect: fn(*mut i8, i32, i32, i32, i32),
    fill_pie: fn(*mut i8, i32, i32, i32, i32, f32, f32),
    write_label: fn(*mut i8, i32, i32, i32, *const u8),
}

impl GraphicsFunctions {
    pub fn placeholders() -> Self {
        fn push_state(_data: *mut i8) {
            panic!("ERROR: Graphics functions not set by frontend!");
        }
        fn pop_state(_data: *mut i8) {
            panic!("ERROR: Graphics functions not set by frontend!");
        }
        fn apply_offset(_data: *mut i8, _x: i32, _y: i32) {
            panic!("ERROR: Graphics functions not set by frontend!");
        }
        fn set_color(_data: *mut i8, _r: u8, _g: u8, _b: u8) {
            panic!("ERROR: Graphics functions not set by frontend!");
        }
        fn set_alpha(_data: *mut i8, _alpha: f32) {
            panic!("ERROR: Graphics functions not set by frontend!");
        }
        fn clear(_data: *mut i8) {
            panic!("ERROR: Graphics functions not set by frontend!");
        }
        fn stroke_line(_data: *mut i8, _x1: i32, _y1: i32, _x2: i32, _y2: i32, _weight: f32) {
            panic!("ERROR: Graphics functions not set by frontend!");
        }
        fn fill_rect(_data: *mut i8, _x: i32, _y: i32, _w: i32, _h: i32) {
            panic!("ERROR: Graphics functions not set by frontend!");
        }
        fn fill_pie(_data: *mut i8, _x: i32, _y: i32, _r: i32, _ir: i32, _sr: f32, _er: f32) {
            panic!("ERROR: Graphics functions not set by frontend!");
        }
        fn write_label(_data: *mut i8, _x: i32, _y: i32, _w: i32, _text: *const u8) {
            panic!("ERROR: Graphics functions not set by frontend!");
        }
        Self {
            push_state,
            pop_state,
            apply_offset,
            set_color,
            set_alpha,
            clear,
            stroke_line,
            fill_rect,
            fill_pie,
            write_label,
        }
    }
}

pub struct GrahpicsWrapper<'a> {
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

    pub fn push_state(&mut self) {
        (self.graphics_fns.push_state)(self.aux_data);
    }

    pub fn pop_state(&mut self) {
        (self.graphics_fns.pop_state)(self.aux_data);
    }

    pub fn apply_offset(&mut self, x: i32, y: i32) {
        (self.graphics_fns.apply_offset)(self.aux_data, x, y);
    }

    pub fn set_color(&mut self, color: &(u8, u8, u8)) {
        (self.graphics_fns.set_color)(self.aux_data, color.0, color.1, color.2);
    }

    pub fn set_alpha(&mut self, alpha: f32) {
        (self.graphics_fns.set_alpha)(self.aux_data, alpha);
    }

    pub fn clear(&mut self) {
        (self.graphics_fns.clear)(self.aux_data);
    }

    pub fn stroke_line(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, weight: f32) {
        (self.graphics_fns.stroke_line)(self.aux_data, x1, y1, x2, y2, weight);
    }

    pub fn fill_rect(&mut self, x: i32, y: i32, w: i32, h: i32) {
        (self.graphics_fns.fill_rect)(self.aux_data, x, y, w, h);
    }

    pub fn fill_pie(
        &mut self,
        x: i32,
        y: i32,
        diameter: i32,
        inner_diameter: i32,
        start_rad: f32,
        end_rad: f32,
    ) {
        (self.graphics_fns.fill_pie)(
            self.aux_data,
            x,
            y,
            diameter,
            inner_diameter,
            start_rad,
            end_rad,
        );
    }

    pub fn write_label(&mut self, x: i32, y: i32, w: i32, text: &str) {
        // TODO: Assert that text is ASCII.
        let raw_text = text.as_bytes();
        let mut raw_text = Vec::from(raw_text);
        raw_text.push(0);
        (self.graphics_fns.write_label)(self.aux_data, x, y, w, raw_text.as_ptr());
    }
}

pub mod constants {
    /// Pixels of padding between grid squares.
    pub const GRID_P: i32 = 8;
    /// Number of pixels a single grid square takes.
    pub const GRID_1: i32 = 20;

    /// Computes the amount of space (in pixels) taken by the given number of grid tiles, with
    /// padding between each tile.
    pub const fn grid(num_spaces: i32) -> i32 {
        GRID_1 * num_spaces + GRID_P * (num_spaces - 1)
    }
    /// Space used by 2 grid squares.
    pub const GRID_2: i32 = grid(2);

    /// Like grid, but returns the amount of space used including extra outside padding. Use  
    /// alongside the fatcoord function.
    pub const fn fatgrid(num_spaces: i32) -> i32 {
        GRID_1 * num_spaces + GRID_P * (num_spaces + 1)
    }
    /// Space used by 1 fat grid square.
    pub const FATGRID_1: i32 = fatgrid(1);
    /// Space used by 2 fat grid squares.
    pub const FATGRID_2: i32 = fatgrid(2);

    /// Computes the coordinate where the provided grid cell begins. For example, 0 would return
    /// GRID_P and 1 would return GRID_1 + GRID_P * 2.
    pub const fn coord(index: i32) -> i32 {
        GRID_1 * index + GRID_P * (index + 1)
    }
    /// Like coord, but allows space for extra padding. Use alongside the fatgrid function.
    pub const fn fatcoord(index: i32) -> i32 {
        GRID_1 * index + GRID_P * index
    }

    pub const KNOB_OUTSIDE_SPACE: i32 = 3;
    pub const KNOB_INSIDE_SPACE: i32 = 3;
    pub const KNOB_AUTOMATION_SPACE: i32 = GRID_2 / 2 - KNOB_OUTSIDE_SPACE - KNOB_INSIDE_SPACE;
    pub const KNOB_LANE_GAP: i32 = 1;
    pub const KNOB_MAX_LANE_SIZE: i32 = 4;

    const fn hex_color(hex: u32) -> (u8, u8, u8) {
        (
            ((hex >> 16) & 0xFF) as u8,
            ((hex >> 8) & 0xFF) as u8,
            ((hex >> 0) & 0xFF) as u8,
        )
    }

    pub const COLOR_BG: (u8, u8, u8) = hex_color(0x121520);
    pub const COLOR_SURFACE: (u8, u8, u8) = hex_color(0x48525F);
    pub const COLOR_KNOB: (u8, u8, u8) = hex_color(0xFF0022);
    pub const COLOR_AUTOMATION: (u8, u8, u8) = hex_color(0xC7D5E8);
    pub const COLOR_AUTOMATION_FOCUSED: (u8, u8, u8) = hex_color(0x54bdff);
    pub const COLOR_TEXT: (u8, u8, u8) = (0xFF, 0xFF, 0xFF);
}
