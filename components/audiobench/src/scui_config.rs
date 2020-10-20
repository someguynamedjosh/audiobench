use scui::{Gui, Pos2D, MaybeMouseBehavior};
use std::rc::Rc;

pub use scui::PlaceholderGuiState as GuiState;
pub use crate::gui::graphics::GrahpicsWrapper as Renderer;

scui::widget! {
    pub Test
}

impl Test {
    fn new(parent: &impl TestParent, pos: Pos2D) -> Rc<Self> {
        Rc::new(Self::create(parent, TestState { pos }))
    }

    fn get_mouse_behavior(self: &Rc<Self>, pos: Pos2D) -> MaybeMouseBehavior {
        // let child = Self::new(self, Pos2D::new(0.0, 0.0));
        None
    }

    fn draw<'r>(self: &Rc<Self>, renderer: &'r mut Renderer<'r>) {
        self.draw_children(renderer);
    }
}

fn test() {
    let gui = Gui::new(
        GuiState,
        |gui| Test::new(gui, Pos2D::new(0.0, 0.0))
    );
}
