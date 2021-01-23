use crate::{
    engine::controls::{IntControl, UpdateRequest},
    gui::{constants::*, module_widgets::ModuleWidgetImpl, InteractionHint, Tooltip},
    scui_config::{DropTarget, MaybeMouseBehavior, Renderer},
};
use scui::{MouseMods, Vec2D, WidgetImpl};
use shared_util::prelude::*;

pub const WIDTH: f32 = grid(2);
pub const HEIGHT: f32 = grid(2) - FONT_SIZE - GRID_P / 2.0;

/// Use this to create a widget which displays an integer and can be clicked / dragged to modify
/// that integer. You must implement get_current_value(self: &Rc<Self>) -> i32,
/// make_callback(self: &Rc<Self>) -> Box<dyn FnMut(i32) -> UpdateRequest>,
/// and get_range() -> (i32, i32) for the generated widget.
#[macro_export]
macro_rules! make_int_box_widget {
    (
        $v: vis $widget_name: ident {
            $($field_names: ident : $field_yaml_tys: ident as $field_tys: ty),*
        }
    ) => {
        yaml_widget_boilerplate::make_widget_outline! {
            widget_struct: $widget_name,
            constructor: new(
                parent: ParentRef,
                pos: GridPos,
                label: String,
                tooltip: String,
                $($field_names: $field_yaml_tys),*
            )
        }

        scui::widget! {
            $v $widget_name
            State {
                pos: Vec2D,
                icons: (usize, usize),
                label: String,
                tooltip: String,
                $($field_names: $field_tys),*
            }
        }

        paste::paste!{
        impl $widget_name {
            pub fn new(
                parent: &impl [<$widget_name Parent>],
                pos: Vec2D,
                label: String,
                tooltip: String,
                $($field_names: $field_tys),*
            ) -> Rc<Self> {
                let int = parent.provide_gui_interface();
                let gui_state = int.state.borrow();
                let registry = gui_state.registry.borrow();
                let state = [<$widget_name State>] {
                    pos,
                    // Factory library is guaranteed to have these icons.
                    icons: (
                        registry.lookup_icon("Factory:increase").unwrap(),
                        registry.lookup_icon("Factory:decrease").unwrap(),
                    ),
                    label,
                    tooltip,
                    $($field_names),*
                };
                Rc::new(Self::create(parent, state))
            }
        }
        }

        impl WidgetImpl<Renderer, DropTarget> for $widget_name {
            fn get_pos_impl(self: &Rc<Self>) -> Vec2D {
                self.state.borrow().pos
            }

            fn get_size_impl(self: &Rc<Self>) -> Vec2D {
                grid(2).into()
            }

            fn get_mouse_behavior_impl(
                self: &Rc<Self>,
                pos: Vec2D,
                _mods: &MouseMods,
            ) -> MaybeMouseBehavior {
                let range = self.get_range();
                let click_delta = if pos.y > HEIGHT / 2.0 {
                    -1
                } else {
                    1
                };
                Some(Box::new(crate::gui::mouse_behaviors::ManipulateIntBox::new(
                    self,
                    self.make_callback(),
                    range.0,
                    range.1,
                    click_delta,
                    self.get_current_value(),
                )))
            }

            fn on_hover_impl(self: &Rc<Self>, _pos: Vec2D) -> Option<()> {
                let tooltip = Tooltip {
                    text: self.state.borrow().tooltip.clone(),
                    interaction: vec![
                        InteractionHint::LeftClick, 
                        InteractionHint::LeftClickAndDrag, 
                        InteractionHint::DoubleClick
                    ],
                };
                self.with_gui_state_mut(|state| {
                    state.set_tooltip(tooltip);
                });
                Some(())
            }

            fn draw_impl(
                self: &Rc<Self>,
                g: &mut Renderer,
            ) {
                let state = self.state.borrow();

                const W: f32 = crate::gui::module_widgets::int_box::WIDTH;
                const H: f32 = crate::gui::module_widgets::int_box::HEIGHT;
                const CS: f32 = CORNER_SIZE;
                g.set_color(&COLOR_BG0);
                g.draw_rounded_rect(0, (W, H), CS);
                const IS: f32 = H / 2.0;
                g.draw_white_icon(state.icons.0, (W - IS, 0.0), IS);
                g.draw_white_icon(state.icons.1, (W - IS, IS), IS);
                {
                    let val = format!("{}", self.get_current_value());
                    g.set_color(&COLOR_FG1);
                    g.draw_text(BIG_FONT_SIZE, 0, (W - IS - 4.0, H), (1, 0), 1, &val);
                }
                {
                    let val = &state.label;
                    g.set_color(&COLOR_FG1);
                    g.draw_text(FONT_SIZE, 0, (W, grid(2)), (0, 1), 1, val);
                }
            }
        }
    };
}

make_int_box_widget! {
    pub IntBox {
        control: IntControlRef as Rcrc<IntControl>
    }
}

impl IntBox {
    fn get_range(self: &Rc<Self>) -> (i32, i32) {
        let range16 = self.state.borrow().control.borrow().get_range();
        (range16.0 as _, range16.1 as _)
    }

    fn get_current_value(self: &Rc<Self>) -> i32 {
        self.state.borrow().control.borrow().get_value() as _
    }

    fn make_callback(self: &Rc<Self>) -> Box<dyn FnMut(i32) -> UpdateRequest> {
        let control = Rc::clone(&self.state.borrow().control);
        Box::new(move |new_value| control.borrow_mut().set_value(new_value as i16))
    }
}

impl ModuleWidgetImpl for IntBox {}
