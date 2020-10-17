use super::ModuleWidget;
use crate::engine::static_controls as staticons;
use crate::gui::action::{ContinuouslyMutateStaticon, MouseAction, MutateStaticon};
use crate::gui::constants::*;
use crate::gui::graphics::{GrahpicsWrapper, HAlign, VAlign};
use crate::gui::{InteractionHint, MouseMods, Tooltip};
use shared_util::prelude::*;

yaml_widget_boilerplate::make_widget_outline! {
    widget_struct: DurationBox,
    constructor: create(
        pos: GridPos,
        duration_control: ControlledDurationRef,
        mode_control: ControlledTimingModeRef,
        label: String,
        tooltip: String,
    ),
}

#[derive(Clone)]
pub struct DurationBox {
    tooltip: String,
    duration_control: Rcrc<staticons::ControlledDuration>,
    mode_control: Rcrc<staticons::ControlledTimingMode>,
    pos: (f32, f32),
    label: String,
}

impl DurationBox {
    const WIDTH: f32 = grid(2);
    const HEIGHT: f32 = grid(2) - FONT_SIZE - GRID_P / 2.0;
    pub fn create(
        pos: (f32, f32),
        duration_control: Rcrc<staticons::ControlledDuration>,
        mode_control: Rcrc<staticons::ControlledTimingMode>,
        label: String,
        tooltip: String,
    ) -> DurationBox {
        DurationBox {
            tooltip,
            duration_control,
            mode_control,
            pos,
            label,
        }
    }
}

impl ModuleWidget for DurationBox {
    fn get_position(&self) -> (f32, f32) {
        self.pos
    }

    fn get_bounds(&self) -> (f32, f32) {
        (grid(2), grid(2))
    }

    fn respond_to_mouse_press(
        &self,
        local_pos: (f32, f32),
        mods: &MouseMods,
        _parent_pos: (f32, f32),
    ) -> Option<Box<dyn MouseAction>> {
        let duration = self.duration_control.borrow();
        let cref = Rc::clone(&self.duration_control);
        if mods.right_click {
            MutateStaticon::wrap(move || cref.borrow_mut().toggle_mode())
        } else if duration.is_using_fractional_mode() {
            let (num, den) = duration.get_fractional_value();
            let use_denominator = local_pos.0 >= Self::WIDTH / 2.0;
            if use_denominator {
                let mut float_value = den as f32;
                ContinuouslyMutateStaticon::wrap(move |delta, _steps| {
                    float_value += delta / 12.0;
                    float_value = float_value.clam(1.0, 99.0);
                    let update = cref
                        .borrow_mut()
                        .set_fractional_value((num, float_value as u8));
                    (update, None)
                })
            } else {
                let mut float_value = num as f32;
                ContinuouslyMutateStaticon::wrap(move |delta, _steps| {
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
            ContinuouslyMutateStaticon::wrap(move |delta, _steps| {
                float_value *= (2.0f32).powf(delta / LOG_OCTAVE_PIXELS);
                float_value = float_value.clam(0.0003, 99.8);
                let update = cref.borrow_mut().set_decimal_value(float_value);
                (update, None)
            })
        }
    }

    fn get_tooltip_at(&self, _local_pos: (f32, f32)) -> Option<Tooltip> {
        Some(Tooltip {
            text: self.tooltip.clone(),
            interaction: InteractionHint::LeftClickAndDrag | InteractionHint::RightClick,
        })
    }

    fn draw(
        &self,
        g: &mut GrahpicsWrapper,
        _highlight: bool,
        _parent_pos: (f32, f32),
        _feedback_data: &[f32],
    ) {
        g.push_state();
        g.apply_offset(self.pos.0, self.pos.1);

        const W: f32 = DurationBox::WIDTH;
        const H: f32 = DurationBox::HEIGHT;
        const CS: f32 = CORNER_SIZE;
        g.set_color(&COLOR_BG0);
        g.fill_rounded_rect(0.0, 0.0, W, H, CS);
        {
            let is_beats = self.mode_control.borrow().is_beat_synchronized();
            let val = self.duration_control.borrow().get_formatted_value();
            let val = format!("{}{}", val, if is_beats { "b" } else { "s" });
            const HA: HAlign = HAlign::Right;
            const VA: VAlign = VAlign::Center;
            g.set_color(&COLOR_FG1);
            g.write_text(
                BIG_FONT_SIZE,
                GRID_P,
                0.0,
                W - GRID_P * 2.0,
                H,
                HA,
                VA,
                1,
                &val,
            );
        }
        {
            let val = &self.label;
            const HA: HAlign = HAlign::Center;
            const VA: VAlign = VAlign::Bottom;
            g.set_color(&COLOR_FG1);
            g.write_text(FONT_SIZE, 0.0, 0.0, W, grid(2), HA, VA, 1, val);
        }

        g.pop_state();
    }
}
