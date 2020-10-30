use crate::gui::constants::FONT_SIZE;
use scui::Vec2D;

#[repr(C)]
pub struct GraphicsFunctions {
    push_state: fn(*mut i8),
    pop_state: fn(*mut i8),
    apply_offset: fn(*mut i8, f32, f32),
    apply_scale: fn(*mut i8, f32),

    set_color: fn(*mut i8, u8, u8, u8),
    set_alpha: fn(*mut i8, f32),
    clear: fn(*mut i8),
    stroke_line: fn(*mut i8, f32, f32, f32, f32, f32),
    fill_rect: fn(*mut i8, f32, f32, f32, f32),
    fill_rounded_rect: fn(*mut i8, f32, f32, f32, f32, f32),
    fill_pie: fn(*mut i8, f32, f32, f32, f32, f32, f32),
    write_text: fn(*mut i8, f32, f32, f32, f32, f32, u8, u8, i32, *const u8),
    write_console_text: fn(*mut i8, f32, f32, *const u8),
    draw_icon: fn(*mut i8, *mut i8, bool, i32, f32, f32, f32),
    draw_box_shadow: fn(*mut i8, f32, f32, f32, f32, f32),
}

impl GraphicsFunctions {
    pub fn placeholders() -> Self {
        fn push_state(_data: *mut i8) {
            panic!("ERROR: Graphics functions not set by frontend!");
        }
        fn pop_state(_data: *mut i8) {
            panic!("ERROR: Graphics functions not set by frontend!");
        }
        fn apply_offset(_data: *mut i8, _x: f32, _y: f32) {
            panic!("ERROR: Graphics functions not set by frontend!");
        }
        fn apply_scale(_data: *mut i8, _s: f32) {
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
        fn stroke_line(_data: *mut i8, _x1: f32, _y1: f32, _x2: f32, _y2: f32, _weight: f32) {
            panic!("ERROR: Graphics functions not set by frontend!");
        }
        fn fill_rect(_data: *mut i8, _x: f32, _y: f32, _w: f32, _h: f32) {
            panic!("ERROR: Graphics functions not set by frontend!");
        }
        fn fill_rounded_rect(_data: *mut i8, _x: f32, _y: f32, _w: f32, _h: f32, _cs: f32) {
            panic!("ERROR: Graphics functions not set by frontend!");
        }
        fn fill_pie(_data: *mut i8, _x: f32, _y: f32, _r: f32, _ir: f32, _sr: f32, _er: f32) {
            panic!("ERROR: Graphics functions not set by frontend!");
        }
        fn write_text(
            _data: *mut i8,
            _font_size: f32,
            _x: f32,
            _y: f32,
            _w: f32,
            _h: f32,
            _halign: u8,
            _valign: u8,
            _max_lines: i32,
            _text: *const u8,
        ) {
            panic!("ERROR: Graphics functions not set by frontend!");
        }
        fn write_console_text(_data: *mut i8, _w: f32, _h: f32, _text: *const u8) {
            panic!("ERROR: Graphics functions not set by frontend!");
        }
        fn draw_icon(
            _data: *mut i8,
            _icon_store: *mut i8,
            _white: bool,
            _index: i32,
            _x: f32,
            _y: f32,
            _s: f32,
        ) {
            panic!("ERROR: Graphics functions not set by frontend!");
        }
        fn draw_box_shadow(_data: *mut i8, _x: f32, _y: f32, _w: f32, _h: f32, _r: f32) {
            panic!("ERROR: Graphics functions not set by frontend!");
        }
        Self {
            push_state,
            pop_state,
            apply_offset,
            apply_scale,
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

impl From<i32> for HAlign {
    fn from(other: i32) -> Self {
        match other {
            -1 => Self::Left,
            0 => Self::Center,
            1 => Self::Right,
            _ => panic!(
                "Valid alignment values are -1, 0, and 1, got {} instead.",
                other
            ),
        }
    }
}

#[repr(i8)]
pub enum VAlign {
    Top = 0,
    Center = 1,
    Bottom = 2,
}

impl From<i32> for VAlign {
    fn from(other: i32) -> Self {
        match other {
            -1 => Self::Top,
            0 => Self::Center,
            1 => Self::Bottom,
            _ => panic!(
                "Valid alignment values are -1, 0, and 1, got {} instead.",
                other
            ),
        }
    }
}

pub struct GrahpicsWrapper {
    graphics_fns: std::rc::Rc<GraphicsFunctions>,
    aux_data: *mut i8,
    icon_store: *mut i8,
}

impl<'a> GrahpicsWrapper {
    pub fn new(
        graphics_fns: std::rc::Rc<GraphicsFunctions>,
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

    pub fn translate<T: Into<Vec2D>>(&mut self, offset: T) {
        let offset = offset.into();
        (self.graphics_fns.apply_offset)(self.aux_data, offset.x, offset.y);
    }

    pub fn scale(&mut self, s: f32) {
        (self.graphics_fns.apply_scale)(self.aux_data, s);
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

    pub fn draw_line<T: Into<Vec2D>, U: Into<Vec2D>>(&mut self, p1: T, p2: U, weight: f32) {
        let p1 = p1.into();
        let p2 = p2.into();
        (self.graphics_fns.stroke_line)(self.aux_data, p1.x, p1.y, p2.x, p2.y, weight);
    }

    pub fn draw_rect<T: Into<Vec2D>, U: Into<Vec2D>>(&mut self, top_right: T, size: U) {
        let top_right = top_right.into();
        let size = size.into();
        (self.graphics_fns.fill_rect)(self.aux_data, top_right.x, top_right.y, size.x, size.y);
    }

    pub fn draw_rounded_rect<T: Into<Vec2D>, U: Into<Vec2D>>(
        &mut self,
        top_right: T,
        size: U,
        corner_size: f32,
    ) {
        let top_right = top_right.into();
        let size = size.into();
        (self.graphics_fns.fill_rounded_rect)(
            self.aux_data,
            top_right.x,
            top_right.y,
            size.x,
            size.y,
            corner_size,
        );
    }

    pub fn draw_pie<T: Into<Vec2D>>(
        &mut self,
        center: T,
        diameter: f32,
        inner_diameter: f32,
        start_rad: f32,
        end_rad: f32,
    ) {
        let center = center.into();
        (self.graphics_fns.fill_pie)(
            self.aux_data,
            center.x,
            center.y,
            diameter,
            inner_diameter,
            start_rad,
            end_rad,
        );
    }

    pub fn draw_label<T: Into<Vec2D>>(&mut self, center: T, w: f32, text: &str) {
        self.draw_text(
            FONT_SIZE,
            center,
            (w, 30.0),
            (HAlign::Center, VAlign::Top),
            2,
            text,
        )
    }

    pub fn draw_text<T: Into<Vec2D>, U: Into<Vec2D>, H: Into<HAlign>, V: Into<VAlign>>(
        &mut self,
        font_size: f32,
        top_left: T,
        size: U,
        align: (H, V),
        max_lines: i32,
        text: &str,
    ) {
        let top_left = top_left.into();
        let size = size.into();
        // TODO: Assert that text is ASCII.
        let raw_text = text.as_bytes();
        let mut raw_text = Vec::from(raw_text);
        raw_text.push(0);
        (self.graphics_fns.write_text)(
            self.aux_data,
            font_size,
            top_left.x,
            top_left.y,
            size.x,
            size.y,
            align.0.into() as u8,
            align.1.into() as u8,
            max_lines,
            raw_text.as_ptr(),
        );
    }

    pub fn draw_console_text<T: Into<Vec2D>>(&mut self, size: T, text: &str) {
        let size = size.into();
        // TODO: Assert that text is ASCII.
        let raw_text = text.as_bytes();
        let mut raw_text = Vec::from(raw_text);
        raw_text.push(0);
        (self.graphics_fns.write_console_text)(self.aux_data, size.x, size.y, raw_text.as_ptr());
    }

    pub fn draw_white_icon<T: Into<Vec2D>>(&mut self, index: usize, top_left: T, size: f32) {
        let top_left = top_left.into();
        (self.graphics_fns.draw_icon)(
            self.aux_data,
            self.icon_store,
            true,
            index as i32,
            top_left.x,
            top_left.y,
            size,
        );
    }

    pub fn draw_icon<T: Into<Vec2D>>(&mut self, index: usize, top_left: T, size: f32) {
        let top_left = top_left.into();
        (self.graphics_fns.draw_icon)(
            self.aux_data,
            self.icon_store,
            false,
            index as i32,
            top_left.x,
            top_left.y,
            size,
        );
    }

    pub fn draw_box_shadow<T: Into<Vec2D>, U: Into<Vec2D>>(
        &mut self,
        top_left: T,
        size: U,
        radius: f32,
    ) {
        let top_left = top_left.into();
        let size = size.into();
        (self.graphics_fns.draw_box_shadow)(
            self.aux_data,
            top_left.x,
            top_left.y,
            size.x,
            size.y,
            radius,
        );
    }

    pub fn draw_inset_box_shadow<T: Into<Vec2D>, U: Into<Vec2D>>(
        &mut self,
        top_left: T,
        size: U,
        radius: f32,
        inset: f32,
    ) {
        let top_left = top_left.into() + inset;
        let size = size.into() - inset * 2.0;
        self.draw_box_shadow(top_left, size, radius);
    }
}

impl scui::Renderer for GrahpicsWrapper {
    fn push_state(&mut self) {
        GrahpicsWrapper::push_state(self)
    }

    fn pop_state(&mut self) {
        GrahpicsWrapper::pop_state(self)
    }

    fn translate(&mut self, offset: scui::Vec2D) {
        GrahpicsWrapper::translate(self, offset)
    }
}
