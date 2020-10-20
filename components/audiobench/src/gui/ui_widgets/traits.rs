use crate::gui::action::{DropTarget, GuiRequest, MouseAction};
use crate::gui::graphics::GrahpicsWrapper;
use crate::gui::{Gui, MouseMods, Tooltip};
use shared_util::prelude::*;

pub trait UiTab {
    fn respond_to_mouse_press(
        self: &Rc<Self>,
        mouse_pos: (f32, f32),
        mods: &MouseMods,
    ) -> Option<Box<dyn MouseAction>>;
    fn on_scroll(self: &Rc<Self>, delta: f32) -> Vec<GuiRequest>;
    fn get_drop_target_at(self: &Rc<Self>, mouse_pos: (f32, f32)) -> DropTarget;
    fn get_tooltip_at(self: &Rc<Self>, mouse_pos: (f32, f32)) -> Option<Tooltip>;
    fn draw(self: &Rc<Self>, g: &mut GrahpicsWrapper, gui_state: &Gui);
}
