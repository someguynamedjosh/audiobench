use scui::{Gui, MaybeMouseBehavior, MouseMods, Vec2D, WidgetImpl};
use std::rc::Rc;

pub use crate::gui::graphics::GrahpicsWrapper as Renderer;
pub use crate::gui::GuiState;

scui::widget! {
    pub Test
}

impl Test {
    fn new(parent: &impl TestParent, pos: Vec2D) -> Rc<Self> {
        Rc::new(Self::create(parent, TestState { pos }))
    }
}

impl<'r> WidgetImpl<'r, Renderer<'r>> for Test {
    fn get_mouse_behavior(self: &Rc<Self>, pos: Vec2D, mods: &MouseMods) -> MaybeMouseBehavior {
        // let child = Self::new(self, Vec2D::new(0.0, 0.0));
        None
    }

    fn draw(self: &Rc<Self>, renderer: &mut Renderer<'r>) {
        self.draw_children(renderer);
    }
}
