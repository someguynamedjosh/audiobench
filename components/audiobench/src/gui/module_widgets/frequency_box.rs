use super::ModuleWidget;
use crate::engine::controls as controls;
use crate::gui::action::MouseAction;
use crate::gui::constants::*;
use crate::gui::graphics::{GrahpicsWrapper, HAlign, VAlign};
use crate::gui::{InteractionHint, MouseMods, Tooltip};
use shared_util::prelude::*;

yaml_widget_boilerplate::make_widget_outline! {
    widget_struct: FrequencyBox,
    constructor: create(
        pos: GridPos,
        control: FrequencyControlRef,
        label: String,
        tooltip: String,
    ),
}

#[derive(Clone)]
pub struct FrequencyBox {
    tooltip: String,
    control: Rcrc<controls::FrequencyControl>,
    pos: (f32, f32),
    label: String,
}

impl FrequencyBox {
    const WIDTH: f32 = grid(2);
    const HEIGHT: f32 = grid(2) - FONT_SIZE - GRID_P / 2.0;
    pub fn create(
        pos: (f32, f32),
        control: Rcrc<controls::FrequencyControl>,
        label: String,
        tooltip: String,
    ) -> FrequencyBox {
        FrequencyBox {
            tooltip,
            control,
            pos,
            label,
        }
    }
}

impl ModuleWidget for FrequencyBox {
    fn get_position(&self) -> (f32, f32) {
        self.pos
    }

    fn get_bounds(&self) -> (f32, f32) {
        (grid(2), grid(2))
    }

    fn respond_to_mouse_press(
        &self,
        _local_pos: (f32, f32),
        _mods: &MouseMods,
        _parent_pos: (f32, f32),
    ) -> MouseAction {
        let frequency = self.control.borrow();
        let cref = Rc::clone(&self.control);
        let mut float_value = frequency.get_value();
        let mutator = Box::new(move |delta, _steps| {
            float_value *= (2.0f32).powf(delta / LOG_OCTAVE_PIXELS);
            float_value = float_value.clam(controls::FrequencyControl::MIN_FREQUENCY, 99_000.0);
            let update = cref.borrow_mut().set_value(float_value);
            (update, None)
        });
        MouseAction::ContinuouslyMutateControl {
            mutator,
            code_reload_requested: false,
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

        const W: f32 = FrequencyBox::WIDTH;
        const H: f32 = FrequencyBox::HEIGHT;
        const CS: f32 = CORNER_SIZE;
        g.set_color(&COLOR_BG0);
        g.fill_rounded_rect(0.0, 0.0, W, H, CS);
        {
            let val = self.control.borrow().get_formatted_value();
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
