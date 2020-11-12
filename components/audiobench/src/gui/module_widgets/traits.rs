use crate::gui::graphics::GrahpicsWrapper;
use crate::gui::top_level::graph::WireTracker;
use crate::scui_config::{DropTarget, Renderer};
use scui::{Vec2D, Widget};

pub trait ModuleWidget: Widget<Renderer, DropTarget> {
    fn draw_in_module(&self, g: &mut GrahpicsWrapper, feedback_data: &[f32]);
    fn add_wires(&self, _wire_tracker: &mut WireTracker) {}
}
