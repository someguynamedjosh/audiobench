use crate::gui::graphics::GrahpicsWrapper;
use crate::gui::top_level::graph::WireTracker;
use crate::scui_config::Renderer;
use scui::Widget;

pub trait ModuleWidget: Widget<Renderer> {
    fn draw_in_module(&self, g: &mut GrahpicsWrapper, highlight: bool, feedback_data: &[f32]);
    fn add_wires(&self, _wire_tracker: &mut WireTracker) {}
}
