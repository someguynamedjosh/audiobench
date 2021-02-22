use crate::{
    gui::{constants::*, InteractionHint, TabArchetype, Tooltip},
    scui_config::{DropTarget, MaybeMouseBehavior, Renderer},
};
use scui::{MouseMods, OnClickBehavior, Vec2D, WidgetImpl};
use shared_util::prelude::*;

scui::widget! {
    pub TabButton
    State {
        pos: Vec2D,
        icon: usize,
        archetype: TabArchetype,
        title: String,
        tooltip: String,
    }
}

const SIZE: f32 = grid(3);
const ICON_SIZE: f32 = SIZE - GRID_P * 1.5 - FONT_SIZE;

impl TabButton {
    pub const SIZE: f32 = SIZE;

    pub fn new(
        parent: &impl TabButtonParent,
        pos: impl Into<Vec2D>,
        icon: usize,
        archetype: TabArchetype,
        title: String,
        tooltip: String,
    ) -> Rc<Self> {
        let state = TabButtonState {
            pos: pos.into(),
            icon,
            archetype,
            title,
            tooltip: tooltip.to_string(),
        };
        let this = Rc::new(Self::create(parent, state));
        this
    }
}

impl WidgetImpl<Renderer, DropTarget> for TabButton {
    fn get_pos_impl(self: &Rc<Self>) -> Vec2D {
        self.state.borrow().pos
    }

    fn get_size_impl(self: &Rc<Self>) -> Vec2D {
        SIZE.into()
    }

    fn get_mouse_behavior_impl(
        self: &Rc<Self>,
        _pos: Vec2D,
        mods: &MouseMods,
    ) -> MaybeMouseBehavior {
        let state = self.state.borrow();
        let tab = state.archetype.clone().instantiate(self);
        drop(state);
        let this = Rc::clone(&self);
        OnClickBehavior::wrap(move || this.with_gui_state_mut(|state| state.switch_to_or_open(tab)))
    }

    fn on_hover_impl(self: &Rc<Self>, pos: Vec2D) -> Option<()> {
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
