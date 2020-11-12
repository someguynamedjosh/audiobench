pub use crate::gui::graphics::GrahpicsWrapper as Renderer;
pub use crate::gui::mouse_behaviors::DropTarget;
pub use crate::gui::GuiState;

// Waiting on https://github.com/rust-lang/rust/issues/41517
// pub trait WidgetImpl = scui::WidgetImpl<Renderer, DropTarget>;
