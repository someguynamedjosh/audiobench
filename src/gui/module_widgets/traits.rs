use crate::gui::action::{DropTarget, MouseAction};
use crate::gui::graph::WireTracker;
use crate::gui::graphics::GrahpicsWrapper;
use crate::gui::{MouseMods, Tooltip};

pub(in crate::gui) trait ModuleWidget {
    fn get_position(&self) -> (f32, f32);
    fn get_bounds(&self) -> (f32, f32);
    fn draw(
        &self,
        g: &mut GrahpicsWrapper,
        highlight: bool,
        parent_pos: (f32, f32),
        feedback_data: &[f32],
    );

    fn respond_to_mouse_press(
        &self,
        local_pos: (f32, f32),
        mods: &MouseMods,
        parent_pos: (f32, f32),
    ) -> MouseAction {
        MouseAction::None
    }
    fn get_drop_target_at(&self, local_pos: (f32, f32)) -> DropTarget {
        DropTarget::None
    }
    fn get_tooltip_at(&self, local_pos: (f32, f32)) -> Option<Tooltip> {
        None
    }
    fn add_wires(&self, wire_tracker: &mut WireTracker) {}
}
