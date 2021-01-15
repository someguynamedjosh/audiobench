use crate::gui::top_level::graph::WireTracker;
use crate::scui_config::{DropTarget, Renderer};
use crate::{engine::controls::Control, gui::graphics::GrahpicsWrapper};
use scui::{Vec2D, Widget, WidgetImpl};
use shared_util::prelude::*;

pub trait ModuleWidgetImpl: WidgetImpl<Renderer, DropTarget> {
    fn represented_control(self: &Rc<Self>) -> Option<Rcrc<dyn Control>> {
        None
    }

    fn use_input_style_wires(self: &Rc<Self>) -> bool {
        false
    }
}

pub trait ModuleWidget: Widget<Renderer, DropTarget> {
    fn represented_control(self: &Self) -> Option<Rcrc<dyn Control>>;
    fn use_input_style_wires(self: &Self) -> bool;
}

impl<T> ModuleWidget for Rc<T>
where
    T: ModuleWidgetImpl,
    Rc<T>: Widget<Renderer, DropTarget>,
{
    fn represented_control(self: &Self) -> Option<Rcrc<dyn Control>> {
        ModuleWidgetImpl::represented_control(self)
    }

    fn use_input_style_wires(self: &Self) -> bool {
        ModuleWidgetImpl::use_input_style_wires(self)
    }
}
