use crate::{
    gui::{constants::*, InteractionHint, Tooltip},
    scui_config::{DropTarget, MaybeMouseBehavior, Renderer},
};
use scui::{MouseMods, Vec2D, WidgetImpl};
use shared_util::prelude::*;

scui::widget! {
    pub IconButton
    State {
        pos: Vec2D,
        size: f32,
        icon: usize,
        enabled: bool,
        mouse_behavior: Box<dyn FnMut(&MouseMods) -> MaybeMouseBehavior>,
        tooltip: String,
    }
}

impl IconButton {
    pub fn new<F>(
        parent: &impl IconButtonParent,
        pos: impl Into<Vec2D>,
        size: f32,
        icon: usize,
        mouse_behavior: F,
        tooltip: impl ToString,
    ) -> Rc<Self>
    where
        F: 'static + FnMut(&MouseMods) -> MaybeMouseBehavior,
    {
        let state = IconButtonState {
            pos: pos.into(),
            size,
            icon,
            enabled: true,
            mouse_behavior: Box::new(mouse_behavior),
            tooltip: tooltip.to_string(),
        };
        let this = Rc::new(Self::create(parent, state));
        this
    }

    pub fn set_enabled(self: &Rc<Self>, enabled: bool) {
        self.state.borrow_mut().enabled = enabled;
    }
}

impl WidgetImpl<Renderer, DropTarget> for IconButton {
    fn get_pos_impl(self: &Rc<Self>) -> Vec2D {
        self.state.borrow().pos
    }

    fn get_size_impl(self: &Rc<Self>) -> Vec2D {
        self.state.borrow().size.into()
    }

    fn get_mouse_behavior_impl(
        self: &Rc<Self>,
        _pos: Vec2D,
        mods: &MouseMods,
    ) -> MaybeMouseBehavior {
        let mut state = self.state.borrow_mut();
        if !state.enabled {
            return None;
        }
        // https://github.com/rust-lang/rust/issues/51886
        (&mut *state.mouse_behavior)(mods)
    }

    fn on_hover_impl(self: &Rc<Self>, _pos: Vec2D) -> Option<()> {
        let tooltip = Tooltip {
            text: self.state.borrow().tooltip.clone(),
            interaction: vec![InteractionHint::LeftClick],
        };
        self.with_gui_state_mut(|state| state.set_tooltip(tooltip));
        Some(())
    }

    fn draw_impl(self: &Rc<Self>, g: &mut Renderer) {
        let state = self.state.borrow();
        g.set_color(&COLOR_BG0);
        g.draw_rounded_rect(0, state.size, CORNER_SIZE);
        const IP: f32 = GRID_P / 2.0;
        g.draw_white_icon(state.icon, IP, state.size - IP * 2.0);
        if !state.enabled {
            g.set_color(&COLOR_BG0);
            g.set_alpha(0.5);
            g.draw_rounded_rect(0, state.size, CORNER_SIZE);
        }
    }
}
