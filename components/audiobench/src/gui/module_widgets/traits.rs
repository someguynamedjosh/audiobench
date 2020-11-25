use crate::gui::graphics::GrahpicsWrapper;
use crate::gui::top_level::graph::WireTracker;
use crate::scui_config::{DropTarget, Renderer};
use scui::{Vec2D, Widget, WidgetImpl};
use shared_util::prelude::*;

pub trait ModuleWidgetImpl: WidgetImpl<Renderer, DropTarget> {
    fn add_wires(self: &Rc<Self>, _wire_tracker: &mut WireTracker) {}
}

pub trait ModuleWidget: Widget<Renderer, DropTarget> {
    fn add_wires(self: &Self, wire_tracker: &mut WireTracker);
}

impl<T> ModuleWidget for Rc<T>
where
    T: ModuleWidgetImpl,
    Rc<T>: Widget<Renderer, DropTarget>,
{
    fn add_wires(self: &Self, wire_tracker: &mut WireTracker) {
        ModuleWidgetImpl::add_wires(self, wire_tracker);
    }
}
