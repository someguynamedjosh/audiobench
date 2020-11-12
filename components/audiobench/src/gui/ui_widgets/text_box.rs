use crate::gui::constants::*;
use crate::scui_config::Renderer;
use scui::{MaybeMouseBehavior, MouseMods, OnClickBehavior, TextField, Vec2D, WidgetImpl};
use shared_util::prelude::*;
use std::time::Instant;

scui::widget! {
    pub TextBox
    State {
        pos: Vec2D,
        size: Vec2D,
        field: Rcrc<TextField>,
        blink_timer: Instant,
        enabled: bool,
    }
}

impl TextBox {
    pub fn new(
        parent: &impl TextBoxParent,
        pos: impl Into<Vec2D>,
        size: impl Into<Vec2D>,
        start_value: String,
        defocus_action: Box<dyn Fn(&str)>,
    ) -> Rc<Self> {
        let field = rcrc(TextField::new(start_value, defocus_action));
        let state = TextBoxState {
            pos: pos.into(),
            size: size.into(),
            field,
            blink_timer: Instant::now(),
            enabled: true,
        };
        let this = Rc::new(Self::create(parent, state));
        this
    }

    pub fn set_text(&self, text: String) {
        self.state.borrow().field.borrow_mut().text = text;
    }

    pub fn get_text(&self) -> String {
        self.state.borrow().field.borrow().text.clone()
    }

    pub fn set_enabled(&self, enabled: bool) {
        self.state.borrow_mut().enabled = enabled;
    }
}

impl WidgetImpl<Renderer> for TextBox {
    fn get_pos(self: &Rc<Self>) -> Vec2D {
        self.state.borrow().pos
    }

    fn get_size(self: &Rc<Self>) -> Vec2D {
        self.state.borrow().size
    }

    fn get_mouse_behavior(
        self: &Rc<Self>,
        _mouse_pos: Vec2D,
        _mods: &MouseMods,
    ) -> MaybeMouseBehavior {
        let mut state = self.state.borrow_mut();
        if !state.enabled {
            return None;
        }
        state.blink_timer = std::time::Instant::now();
        let field = Rc::clone(&state.field);
        let gui = Rc::clone(&self.parents.gui);
        OnClickBehavior::wrap(move || {
            gui.focus_text_field(&field);
        })
    }

    fn draw(self: &Rc<Self>, g: &mut Renderer) {
        const GP: f32 = GRID_P;
        let state = self.state.borrow();
        let field = state.field.borrow();
        let text = &field.text;
        let focused = field.is_focused();

        g.set_color(if focused { &COLOR_BG1 } else { &COLOR_BG0 });
        g.draw_rounded_rect(0, state.size, CORNER_SIZE);
        g.set_color(&COLOR_FG1);
        let w = state.size.x - GP * 2.0;
        let text = if focused && state.blink_timer.elapsed().as_millis() % 800 < 400 {
            format!("{}|", text)
        } else {
            format!("{}", text)
        };
        g.draw_text(FONT_SIZE, (GP, 0.0), (w, state.size.y), (-1, 0), 1, &text);
    }
}
