use crate::{
    gui::{constants::*, InteractionHint, Tooltip},
    scui_config::{DropTarget, MaybeMouseBehavior, Renderer},
};
use scui::{MouseMods, OnClickBehavior, Vec2D, WidgetImpl};
use shared_util::prelude::*;

scui::widget! {
    pub LinkButton
    State {
        pos: Vec2D,
        icon: usize,
        url: String,
        title: String,
        tooltip: String,
    }
}

const SIZE: f32 = grid(3);
const ICON_SIZE: f32 = SIZE - GRID_P * 1.5 - FONT_SIZE;

impl LinkButton {
    pub const SIZE: f32 = SIZE;

    pub fn new(
        parent: &impl LinkButtonParent,
        pos: impl Into<Vec2D>,
        icon: usize,
        url: String,
        title: String,
        tooltip: String,
    ) -> Rc<Self> {
        let state = LinkButtonState {
            pos: pos.into(),
            icon,
            url,
            title,
            tooltip: tooltip.to_string(),
        };
        let this = Rc::new(Self::create(parent, state));
        this
    }
}

impl WidgetImpl<Renderer, DropTarget> for LinkButton {
    fn get_pos_impl(self: &Rc<Self>) -> Vec2D {
        self.state.borrow().pos
    }

    fn get_size_impl(self: &Rc<Self>) -> Vec2D {
        SIZE.into()
    }

    fn get_mouse_behavior_impl(
        self: &Rc<Self>,
        _pos: Vec2D,
        _mods: &MouseMods,
    ) -> MaybeMouseBehavior {
        let this = Rc::clone(self);
        let state = self.state.borrow();
        let url = state.url.clone();
        OnClickBehavior::wrap(move || {
            if let Err(err) = webbrowser::open(&url) {
                this.with_gui_state_mut(|state| {
                    state.add_error_message(format!(
                        "Failed to open web browser, see console for details."
                    ));
                    eprintln!("WARNING: Failed to open web browser, caused by:\n{}", err);
                })
            }
        })
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
        g.draw_rounded_rect(0, SIZE, CORNER_SIZE);
        const IP: f32 = GRID_P / 2.0;
        const IX: f32 = SIZE / 2.0 - ICON_SIZE / 2.0;
        g.draw_white_icon(state.icon, (IX, IP), ICON_SIZE);
        g.set_color(&COLOR_FG1);
        g.draw_text(
            FONT_SIZE,
            // -1.0 is arbitrary magic number.
            (IP, SIZE - IP - FONT_SIZE - 2.0),
            (SIZE - IP * 2.0, FONT_SIZE),
            (0, -1),
            1,
            &state.title,
        );
    }
}
