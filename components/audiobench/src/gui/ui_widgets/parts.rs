use crate::gui::action::{GuiRequest, MouseAction};
use crate::gui::constants::*;
use crate::gui::graphics::{GrahpicsWrapper, HAlign, VAlign};
use crate::gui::MouseMods;
use shared_util::prelude::*;

pub struct TextField {
    pub text: String,
    focused: bool,
    defocus_action: Box<dyn Fn(&str) -> Vec<GuiRequest>>,
}

impl TextField {
    fn new(
        start_value: String,
        defocus_action: Box<dyn Fn(&str) -> Vec<GuiRequest>>,
    ) -> Self {
        Self {
            text: start_value,
            focused: false,
            defocus_action,
        }
    }

    pub fn focus(&mut self) {
        debug_assert!(!self.focused);
        self.focused = true;
    }

    pub fn defocus(&mut self) -> Vec<GuiRequest> {
        debug_assert!(self.focused);
        self.focused = false;
        (self.defocus_action)(&self.text)
    }
}

pub struct TextBox {
    pub pos: (f32, f32),
    pub size: (f32, f32),
    pub field: Rcrc<TextField>,
    blink_timer: std::time::Instant,
}

impl TextBox {
    pub fn create(
        pos: (f32, f32),
        size: (f32, f32),
        start_value: String,
        defocus_action: Box<dyn Fn(&str) -> Vec<GuiRequest>>,
    ) -> Self {
        Self {
            pos,
            size,
            field: rcrc(TextField::new(start_value, defocus_action)),
            blink_timer: std::time::Instant::now(),
        }
    }

    pub fn respond_to_mouse_press(
        &mut self,
        _mouse_pos: (f32, f32),
        _mods: &MouseMods,
    ) -> Option<Box<dyn MouseAction>> {
        self.blink_timer = std::time::Instant::now();
        GuiRequest::FocusTextField(Rc::clone(&self.field)).into()
    }

    pub fn draw(&self, g: &mut GrahpicsWrapper) {
        const GP: f32 = GRID_P;
        let field = self.field.borrow();
        let text = &field.text;
        let focused = field.focused;
        g.push_state();
        g.apply_offset(self.pos.0, self.pos.1);

        g.set_color(if focused { &COLOR_BG1 } else { &COLOR_BG0 });
        g.fill_rounded_rect(0.0, 0.0, self.size.0, self.size.1, CORNER_SIZE);
        g.set_color(&COLOR_FG1);
        const H: HAlign = HAlign::Left;
        const V: VAlign = VAlign::Center;
        let w = self.size.0 - GP * 2.0;
        let text = if focused && self.blink_timer.elapsed().as_millis() % 800 < 400 {
            format!("{}|", text)
        } else {
            format!("{}", text)
        };
        g.write_text(FONT_SIZE, GP, 0.0, w, self.size.1, H, V, 1, &text);

        g.pop_state();
    }
}

pub struct IconButton {
    pos: (f32, f32),
    size: f32,
    icon: usize,
    pub enabled: bool,
}

impl IconButton {
    pub fn create(pos: (f32, f32), size: f32, icon: usize) -> Self {
        Self {
            pos,
            size,
            icon,
            enabled: true,
        }
    }

    pub fn mouse_in_bounds(&self, mouse_pos: (f32, f32)) -> bool {
        self.enabled && mouse_pos.sub(self.pos).inside((self.size, self.size))
    }

    pub fn draw(&self, g: &mut GrahpicsWrapper) {
        g.push_state();
        g.apply_offset(self.pos.0, self.pos.1);

        g.set_color(&COLOR_BG0);
        g.fill_rounded_rect(0.0, 0.0, self.size, self.size, CORNER_SIZE);
        const IP: f32 = GRID_P / 2.0;
        g.draw_white_icon(self.icon, IP, IP, self.size - IP * 2.0);
        if !self.enabled {
            g.set_color(&COLOR_BG0);
            g.set_alpha(0.5);
            g.fill_rounded_rect(0.0, 0.0, self.size, self.size, CORNER_SIZE);
        }

        g.pop_state();
    }
}
