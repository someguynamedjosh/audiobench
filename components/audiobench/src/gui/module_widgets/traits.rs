use crate::engine::controls::Control;
use crate::scui_config::{DropTarget, Renderer};
use scui::{Widget, WidgetImpl};
use shared_util::prelude::*;

pub trait ModuleWidgetImpl: WidgetImpl<Renderer, DropTarget> {
    fn represented_control(self: &Rc<Self>) -> Option<Rcrc<dyn Control>> {
        None
    }

    fn use_input_style_wires(self: &Rc<Self>) -> bool {
        false
    }

    fn take_feedback_data(self: &Rc<Self>, data: Vec<f32>) {}
}

pub trait ModuleWidget: Widget<Renderer, DropTarget> {
    fn represented_control(self: &Self) -> Option<Rcrc<dyn Control>>;
    fn use_input_style_wires(self: &Self) -> bool;
    fn take_feedback_data(self: &Self, data: Vec<f32>);
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

    fn take_feedback_data(self: &Self, data: Vec<f32>) {
        ModuleWidgetImpl::take_feedback_data(self, data)
    }
}
