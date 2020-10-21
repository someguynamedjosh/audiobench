use scui::{Gui, MaybeMouseBehavior, MouseMods, Pos2D, WidgetImpl};
use std::rc::Rc;

pub use crate::gui::graphics::GrahpicsWrapper as Renderer;
pub use scui::PlaceholderGuiState as GuiState;

scui::widget! {
    pub Test
}

impl Test {
    fn new(parent: &impl TestParent, pos: Pos2D) -> Rc<Self> {
        Rc::new(Self::create(parent, TestState { pos }))
    }
}

impl<'r> WidgetImpl<'r, Renderer<'r>> for Test {
    fn get_mouse_behavior(self: &Rc<Self>, pos: Pos2D, mods: &MouseMods) -> MaybeMouseBehavior {
        // let child = Self::new(self, Pos2D::new(0.0, 0.0));
        None
    }

    fn draw(self: &Rc<Self>, renderer: &mut Renderer<'r>) {
        self.draw_children(renderer);
    }
}

fn test() {
    let gui = Gui::new(GuiState, |gui| Test::new(gui, Pos2D::new(0.0, 0.0)));
}
