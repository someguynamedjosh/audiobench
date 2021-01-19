use super::ModuleWidgetImpl;
use crate::engine::controls::{DurationControl, TimingModeControl, };
use crate::gui::constants::*;
use crate::gui::mouse_behaviors::{ContinuouslyMutateControl, MutateControl};
use crate::gui::{InteractionHint, Tooltip};
use crate::scui_config::{DropTarget, MaybeMouseBehavior, Renderer};
use scui::{MouseMods, Vec2D, WidgetImpl};
use shared_util::prelude::*;

yaml_widget_boilerplate::make_widget_outline! {
    widget_struct: DurationBox,
    constructor: new(
        parent: ParentRef,
        pos: GridPos,
        duration_control: DurationControlRef,
        mode_control: TimingModeControlRef,
        label: String,
        tooltip: String,
    ),
}

scui::widget! {
    pub DurationBox
    State {
        pos: Vec2D,
        tooltip: String,
        duration_control: Rcrc<DurationControl>,
        mode_control: Rcrc<TimingModeControl>,
        label: String,
    }
}

const WIDTH: f32 = grid(2);
const HEIGHT: f32 = grid(2) - FONT_SIZE - GRID_P / 2.0;

impl DurationBox {
    pub fn new(
        parent: &impl DurationBoxParent,
        pos: Vec2D,
        duration_control: Rcrc<DurationControl>,
        mode_control: Rcrc<TimingModeControl>,
        label: String,
        tooltip: String,
    ) -> Rc<Self> {
        let state = DurationBoxState {
            pos,
            tooltip,
            duration_control,
            mode_control,
            label,
        };
        Rc::new(Self::create(parent, state))
    }
}

impl WidgetImpl<Renderer, DropTarget> for DurationBox {
    fn get_pos_impl(self: &Rc<Self>) -> Vec2D {
        self.state.borrow().pos
    }

    fn get_size_impl(self: &Rc<Self>) -> Vec2D {
        (grid(2), grid(2)).into()
    }

    fn get_mouse_behavior_impl(
        self: &Rc<Self>,
        pos: Vec2D,
        mods: &MouseMods,
    ) -> MaybeMouseBehavior {
        let state = self.state.borrow();

        let duration = state.duration_control.borrow();
        let cref = Rc::clone(&state.duration_control);
        if mods.right_click {
            MutateControl::wrap(self, move || cref.borrow_mut().toggle_mode())
        } else if duration.is_using_fractional_mode() {
            let (num, den) = duration.get_fractional_value();
            let use_denominator = pos.x >= WIDTH / 2.0;
            if use_denominator {
                let mut float_value = den as f32;
                ContinuouslyMutateControl::wrap(self, move |delta, _steps| {
                    float_value += delta / 12.0;
                    float_value = float_value.clam(1.0, 99.0);
                    let update = cref
                        .borrow_mut()
                        .set_fractional_value((num, float_value as u8));
                    (update, None)
                })
            } else {
                let mut float_value = num as f32;
                ContinuouslyMutateControl::wrap(self, move |delta, _steps| {
                    float_value += delta / 12.0;
                    float_value = float_value.clam(1.0, 99.0);
                    let update = cref
                        .borrow_mut()
                        .set_fractional_value((float_value as u8, den));
                    (update, None)
                })
            }
        } else {
            let mut float_value = duration.get_decimal_value();
            ContinuouslyMutateControl::wrap(self, move |delta, _steps| {
                float_value *= (2.0f32).powf(delta / LOG_OCTAVE_PIXELS);
                float_value = float_value.clam(0.0003, 99.8);
                let update = cref.borrow_mut().set_decimal_value(float_value);
                (update, None)
            })
        }
    }

    fn on_hover_impl(self: &Rc<Self>, _pos: Vec2D) -> Option<()> {
        let this_state = self.state.borrow();
        self.with_gui_state_mut(|state| {
            state.set_tooltip(Tooltip {
                text: this_state.tooltip.clone(),
                interaction: InteractionHint::LeftClickAndDrag | InteractionHint::RightClick,
            });
        });
        Some(())
    }

    fn draw_impl(self: &Rc<Self>, g: &mut Renderer) {
        let state = self.state.borrow();
        const W: f32 = WIDTH;
        const H: f32 = HEIGHT;
        const CS: f32 = CORNER_SIZE;
        g.set_color(&COLOR_BG0);
        g.draw_rounded_rect(0, (W, H), CS);
        {
            let is_beats = state.mode_control.borrow().is_beat_synchronized();
            let val = state.duration_control.borrow().get_formatted_value();
            let val = format!("{}{}", val, if is_beats { "b" } else { "s" });
            g.set_color(&COLOR_FG1);
            g.draw_text(
                BIG_FONT_SIZE,
                (GRID_P, 0.0),
                (W - GRID_P * 2.0, H),
                (-1, 0),
                1,
                &val,
            );
        }
        {
            let val = &state.label;
            g.set_color(&COLOR_FG1);
            g.draw_text(FONT_SIZE, 00, (W, grid(2)), (0, 1), 1, val);
        }
    }
}

impl ModuleWidgetImpl for DurationBox {}
