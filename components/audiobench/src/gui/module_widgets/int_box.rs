use super::ModuleWidgetImpl;
use crate::engine::static_controls as staticons;
use crate::gui::constants::*;
use crate::gui::graphics::{HAlign, VAlign};
use crate::gui::mouse_behaviors::ManipulateIntBox;
use crate::gui::{InteractionHint, Tooltip};
use crate::scui_config::{DropTarget, Renderer, MaybeMouseBehavior};
use scui::{MouseMods, Vec2D, WidgetImpl};
use shared_util::prelude::*;

pub const WIDTH: f32 = grid(2);
pub const HEIGHT: f32 = grid(2) - FONT_SIZE - GRID_P / 2.0;

/// Use this to create a widget which displays an integer and can be clicked / dragged to modify
/// that integer. You must implement get_current_value(self: &Rc<Self>) -> i32 and
/// make_callback(self: &Rc<Self>) -> Box<dyn FnMut(i32) -> staticons::StaticonUpdateRequest>
/// for the generated widget.
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
                range: IntRange,
                label: String,
                tooltip: String,
                $($field_names: $field_yaml_tys),*
            )
        }

        scui::widget! {
            $v $widget_name
            State {
                pos: Vec2D,
                range: (i32, i32),
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
                range: (i32, i32),
                label: String,
                tooltip: String,
                $($field_names: $field_tys),*
            ) -> Rc<Self> {
                let int = parent.provide_gui_interface();
                let gui_state = int.state.borrow();
                let registry = gui_state.registry.borrow();
                let state = [<$widget_name State>] {
                    pos,
                    range,
                    // Factory library is guaranteed to have these icons.
                    icons: (
                        registry.lookup_icon("factory:increase").unwrap(),
                        registry.lookup_icon("factory:decrease").unwrap(),
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
                let state = self.state.borrow();
                let click_delta = if pos.y > HEIGHT / 2.0 {
                    -1
                } else {
                    1
                };
                Some(Box::new(crate::gui::mouse_behaviors::ManipulateIntBox::new(
                    self,
                    self.make_callback(),
                    state.range.0,
                    state.range.1,
                    click_delta,
                    self.get_current_value(),
                )))
            }

            fn on_hover_impl(self: &Rc<Self>, _pos: Vec2D) -> Option<()> {
                let tooltip = Tooltip {
                    text: self.state.borrow().tooltip.clone(),
                    interaction: InteractionHint::LeftClick
                        | InteractionHint::LeftClickAndDrag
                        | InteractionHint::DoubleClick,
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
        control: ControlledIntRef as Rcrc<staticons::ControlledInt>
    }
}

impl IntBox {
    fn get_current_value(self: &Rc<Self>) -> i32 {
        self.state.borrow().control.borrow().get_value() as _
    }

    fn make_callback(self: &Rc<Self>) -> Box<dyn FnMut(i32) -> staticons::StaticonUpdateRequest> {
        let control = Rc::clone(&self.state.borrow().control);
        Box::new(move |new_value| control.borrow_mut().set_value(new_value as i16))
    }
}

impl ModuleWidgetImpl for IntBox {}
