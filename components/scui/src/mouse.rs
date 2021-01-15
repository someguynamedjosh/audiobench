use crate::Vec2D;
use std::time::Duration;

/// How many pixels the mouse must move across while being held down for dragging to start.
pub const DRAG_DEADZONE: f32 = 4.0;
/// The maximum amount of time between two clicks for the event to be considered a double-click.
pub const DOUBLE_CLICK_TIME: Duration = Duration::from_millis(400);
/// If the user moves the mouse at least this much between two clicks, it will not be considered
/// a double-click.
pub const DOUBLE_CLICK_RANGE: f32 = 4.0;

pub trait MouseBehavior<DT> {
    fn on_drag(&mut self, _delta: Vec2D, _mods: &MouseMods) {}
    fn on_drop(self: Box<Self>, _drop_target: Option<DT>) {}
    fn on_click(self: Box<Self>) {}
    fn on_double_click(self: Box<Self>) {}
}

pub struct OnClickBehavior<F: FnOnce()> {
    action: F,
}

impl<F: FnOnce() + 'static> OnClickBehavior<F> {
    pub fn wrap<DT>(action: F) -> MaybeMouseBehavior<DT> {
        Some(Box::new(Self { action }))
    }
}

impl<F: FnOnce() + 'static> From<F> for OnClickBehavior<F> {
    fn from(other: F) -> Self {
        Self { action: other }
    }
}

impl<F: FnOnce(), DT> MouseBehavior<DT> for OnClickBehavior<F> {
    fn on_click(self: Box<Self>) {
        (self.action)();
    }
}

pub type MaybeMouseBehavior<DT> = Option<Box<dyn MouseBehavior<DT>>>;

pub struct MouseMods {
    pub right_click: bool,
    pub snap: bool,
    pub precise: bool,
}
