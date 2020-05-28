use crate::gui::constants::FONT_SIZE;

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
    fill_rounded_rect: fn(*mut i8, i32, i32, i32, i32, i32),
    fill_pie: fn(*mut i8, i32, i32, i32, i32, f32, f32),
    write_text: fn(*mut i8, i32, i32, i32, i32, i32, u8, u8, i32, *const u8),
    write_console_text: fn(*mut i8, i32, i32, *const u8),
    draw_icon: fn(*mut i8, *mut i8, bool, i32, i32, i32, i32),
    draw_box_shadow: fn(*mut i8, i32, i32, i32, i32, i32),
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
        fn fill_rounded_rect(_data: *mut i8, _x: i32, _y: i32, _w: i32, _h: i32, _cs: i32) {
            panic!("ERROR: Graphics functions not set by frontend!");
        }
        fn fill_pie(_data: *mut i8, _x: i32, _y: i32, _r: i32, _ir: i32, _sr: f32, _er: f32) {
            panic!("ERROR: Graphics functions not set by frontend!");
        }
        fn write_text(
            _data: *mut i8,
            _font_size: i32,
            _x: i32,
            _y: i32,
            _w: i32,
            _h: i32,
            _halign: u8,
            _valign: u8,
            _max_lines: i32,
            _text: *const u8,
        ) {
            panic!("ERROR: Graphics functions not set by frontend!");
        }
        fn write_console_text(_data: *mut i8, _w: i32, _h: i32, _text: *const u8) {
            panic!("ERROR: Graphics functions not set by frontend!");
        }
        fn draw_icon(
            _data: *mut i8,
            _icon_store: *mut i8,
            _white: bool,
            _index: i32,
            _x: i32,
            _y: i32,
            _s: i32,
        ) {
            panic!("ERROR: Graphics functions not set by frontend!");
        }
        fn draw_box_shadow(_data: *mut i8, _x: i32, _y: i32, _w: i32, _h: i32, _r: i32) {
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
            fill_rounded_rect,
            fill_pie,
            write_text,
            write_console_text,
            draw_icon,
            draw_box_shadow,
        }
    }
}

#[repr(i8)]
pub enum HAlign {
    Left = 0,
    Center = 1,
    Right = 2,
}

#[repr(i8)]
pub enum VAlign {
    Top = 0,
    Center = 1,
    Bottom = 2,
}

pub struct GrahpicsWrapper<'a> {
    graphics_fns: &'a GraphicsFunctions,
    aux_data: *mut i8,
    icon_store: *mut i8,
}

impl<'a> GrahpicsWrapper<'a> {
    pub fn new(
        graphics_fns: &GraphicsFunctions,
        aux_data: *mut i8,
        icon_store: *mut i8,
    ) -> GrahpicsWrapper {
        GrahpicsWrapper {
            graphics_fns,
            aux_data,
            icon_store,
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

    pub fn fill_rounded_rect(&mut self, x: i32, y: i32, w: i32, h: i32, corner_size: i32) {
        (self.graphics_fns.fill_rounded_rect)(self.aux_data, x, y, w, h, corner_size);
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
        self.write_text(FONT_SIZE, x, y, w, 30, HAlign::Center, VAlign::Top, 2, text)
    }

    pub fn write_text(
        &mut self,
        font_size: i32,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        halign: HAlign,
        valign: VAlign,
        max_lines: i32,
        text: &str,
    ) {
        // TODO: Assert that text is ASCII.
        let raw_text = text.as_bytes();
        let mut raw_text = Vec::from(raw_text);
        raw_text.push(0);
        (self.graphics_fns.write_text)(
            self.aux_data,
            font_size,
            x,
            y,
            w,
            h,
            halign as u8,
            valign as u8,
            max_lines,
            raw_text.as_ptr(),
        );
    }

    pub fn write_console_text(&mut self, w: i32, h: i32, text: &str) {
        // TODO: Assert that text is ASCII.
        let raw_text = text.as_bytes();
        let mut raw_text = Vec::from(raw_text);
        raw_text.push(0);
        (self.graphics_fns.write_console_text)(self.aux_data, w, h, raw_text.as_ptr());
    }

    pub fn draw_white_icon(&mut self, index: usize, x: i32, y: i32, size: i32) {
        (self.graphics_fns.draw_icon)(
            self.aux_data,
            self.icon_store,
            true,
            index as i32,
            x,
            y,
            size,
        );
    }

    pub fn draw_icon(&mut self, index: usize, x: i32, y: i32, size: i32) {
        (self.graphics_fns.draw_icon)(
            self.aux_data,
            self.icon_store,
            false,
            index as i32,
            x,
            y,
            size,
        );
    }

    pub fn draw_box_shadow(&mut self, x: i32, y: i32, w: i32, h: i32, radius: i32) {
        (self.graphics_fns.draw_box_shadow)(self.aux_data, x, y, w, h, radius);
    }

    pub fn draw_inset_box_shadow(
        &mut self,
        mut x: i32,
        mut y: i32,
        mut w: i32,
        mut h: i32,
        radius: i32,
        inset: i32,
    ) {
        x += inset;
        y += inset;
        w -= inset * 2;
        h -= inset * 2;
        self.draw_box_shadow(x, y, w, h, radius);
    }
}
