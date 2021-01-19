pub use crate::gui::{
    graphics::GrahpicsWrapper as Renderer, mouse_behaviors::DropTarget, GuiState,
};
pub type MaybeMouseBehavior = scui::MaybeMouseBehavior<DropTarget>;

// Waiting on https://github.com/rust-lang/rust/issues/41517
// pub trait WidgetImpl = scui::WidgetImpl<Renderer, DropTarget>;
