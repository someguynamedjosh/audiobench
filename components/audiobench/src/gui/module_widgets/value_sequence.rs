use super::ModuleWidget;
use super::{IntBoxBase, IntBoxImpl};
use crate::engine::parts as ep;
use crate::engine::static_controls as staticons;
use crate::gui::action::MouseAction;
use crate::gui::constants::*;
use crate::gui::graphics::GrahpicsWrapper;
use crate::gui::{InteractionHint, MouseMods, Tooltip};
use crate::registry::Registry;
use crate::util::*;

yaml_widget_boilerplate::make_widget_outline! {
    widget_struct: ValueSequence,
    constructor: create(
        pos: GridPos,
        size: GridSize,
        sequence_control: ControlledValueSequenceRef,
        ramping_control: AutoconRef,
        tooltip: String,
    ),
    // Feedback for playhead and ramping amount.
    feedback: custom(2),
}

#[derive(Clone)]
pub struct ValueSequence {
    tooltip: String,
    sequence_control: Rcrc<staticons::ControlledValueSequence>,
    ramping_control: Rcrc<ep::Autocon>,
    pos: (f32, f32),
    width: f32,
}

impl ValueSequence {
    const HEIGHT: f32 = grid(2);
    const HEADER_SPACE: f32 = CORNER_SIZE * 2.0;

    pub fn create(
        pos: (f32, f32),
        size: (f32, f32),
        sequence_control: Rcrc<staticons::ControlledValueSequence>,
        ramping_control: Rcrc<ep::Autocon>,
        tooltip: String,
    ) -> ValueSequence {
        ValueSequence {
            tooltip,
            sequence_control,
            ramping_control,
            pos,
            width: size.0,
        }
    }
}

impl ModuleWidget for ValueSequence {
    fn get_position(&self) -> (f32, f32) {
        self.pos
    }

    fn get_bounds(&self) -> (f32, f32) {
        (self.width, ValueSequence::HEIGHT)
    }

    fn respond_to_mouse_press(
        &self,
        local_pos: (f32, f32),
        _mods: &MouseMods,
        _parent_pos: (f32, f32),
    ) -> MouseAction {
        let borrowed = self.sequence_control.borrow();
        let num_steps = borrowed.get_len();
        let step_width = self.width / num_steps as f32;
        let clicked_step = (local_pos.0 / step_width) as usize;
        let mut float_value = borrowed.get_value(clicked_step);
        let cref = Rc::clone(&self.sequence_control);
        let mutator = Box::new(move |delta, steps| {
            float_value += delta / 100.0;
            float_value = float_value.clam(-1.0, 1.0);
            let final_value = if let Some(steps) = steps {
                float_value.snap(-1.0, 1.0, steps)
            } else {
                float_value
            };
            let update = cref.borrow_mut().set_value(clicked_step, final_value);
            let tooltip = Tooltip {
                interaction: InteractionHint::Shift | InteractionHint::Alt,
                text: format!("{:.3}", final_value),
            };
            (update, Some(tooltip))
        });
        MouseAction::ContinuouslyMutateStaticon {
            mutator,
            code_reload_requested: false,
        }
    }

    fn get_tooltip_at(&self, _local_pos: (f32, f32)) -> Option<Tooltip> {
        Some(Tooltip {
            text: self.tooltip.clone(),
            interaction: InteractionHint::LeftClickAndDrag.into(),
        })
    }

    fn draw(
        &self,
        g: &mut GrahpicsWrapper,
        _highlight: bool,
        _parent_pos: (f32, f32),
        feedback_data: &[f32],
    ) {
        assert_eq!(feedback_data.len(), 2);
        g.push_state();
        g.apply_offset(self.pos.0, self.pos.1);

        const H: f32 = ValueSequence::HEIGHT;
        const CS: f32 = CORNER_SIZE;
        const HEAD: f32 = ValueSequence::HEADER_SPACE;
        g.set_color(&COLOR_BG);
        g.fill_rounded_rect(0.0, HEAD, self.width, H - HEAD, CS);

        let borrowed = self.sequence_control.borrow();
        let num_steps = borrowed.get_len();
        let step_width = self.width / num_steps as f32;
        let ramping = feedback_data[1];
        const MIDPOINT: f32 = HEAD + (H - HEAD) * 0.5;
        let first_value = borrowed.get_value(0);
        let mut value = first_value;
        for step_index in 0..num_steps {
            let x = step_index as f32 * step_width;
            if step_index != num_steps - 1 {
                g.set_color(&COLOR_TEXT);
                // g.set_alpha(0.5);
                g.stroke_line(x + step_width, HEAD, x + step_width, H, 1.0);
            }
            g.set_color(&COLOR_KNOB);
            let y = (0.5 - value * 0.5) * (H - HEAD) + HEAD;
            g.set_alpha(0.3);
            g.fill_rect(x, MIDPOINT.min(y), step_width, (MIDPOINT - y).abs());
            g.set_alpha(1.0);
            g.stroke_line(x, y, x + step_width * (1.0 - ramping), y, 2.0);
            let next_value = if step_index < num_steps - 1 {
                borrowed.get_value(step_index + 1)
            } else {
                first_value
            };
            value = next_value;
            let next_y = (0.5 - next_value * 0.5) * (H - HEAD) + HEAD;
            g.stroke_line(
                x + step_width * (1.0 - ramping),
                y,
                x + step_width,
                next_y,
                2.0,
            );
        }

        g.set_color(&COLOR_TEXT);
        g.fill_pie(
            feedback_data[0] * step_width - HEAD,
            0.0,
            HEAD * 2.0,
            0.0,
            std::f32::consts::PI * 0.75,
            std::f32::consts::PI * 0.25,
        );

        g.pop_state();
    }
}

yaml_widget_boilerplate::make_widget_outline! {
    widget_struct: ValueSequenceLength,
    constructor: create(
        registry: RegistryRef,
        pos: GridPos,
        sequence_control: ControlledValueSequenceRef,
        label: String,
        tooltip: String,
    ),
}

pub struct ValueSequenceLength {
    base: IntBoxBase,
    sequence_control: Rcrc<staticons::ControlledValueSequence>,
}

impl ValueSequenceLength {
    pub fn create(
        registry: &Registry,
        pos: (f32, f32),
        sequence_control: Rcrc<staticons::ControlledValueSequence>,
        label: String,
        tooltip: String,
    ) -> Self {
        Self {
            base: IntBoxBase::create(tooltip, registry, pos, (1, 99), label),
            sequence_control,
        }
    }
}

impl IntBoxImpl for ValueSequenceLength {
    fn get_base(&self) -> &IntBoxBase {
        &self.base
    }

    fn get_current_value(&self) -> i32 {
        self.sequence_control.borrow().get_len() as _
    }

    fn make_callback(&self) -> Box<dyn FnMut(i32) -> staticons::StaticonUpdateRequest> {
        let sequence_control = Rc::clone(&self.sequence_control);
        Box::new(move |new_length| {
            assert!(new_length >= 1);
            sequence_control.borrow_mut().set_len(new_length as usize)
        })
    }
}
