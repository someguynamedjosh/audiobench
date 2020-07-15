use crate::registry::yaml::YamlNode;
use super::ModuleWidget;
use super::{IntBoxBase, IntBoxImpl};
use crate::engine::parts as ep;
use crate::gui::action::MouseAction;
use crate::gui::constants::*;
use crate::gui::graphics::{GrahpicsWrapper, HAlign, VAlign};
use crate::gui::{InteractionHint, MouseMods, Tooltip};
use crate::registry::Registry;
use crate::util::*;

yaml_widget_boilerplate::make_widget_outline! {
    widget_struct: TriggerSequence,
    constructor: create(
        pos: GridPos,
        size: GridSize,
        sequence_control: ComplexControlRef,
        tooltip: String,
    ),
    // Feedback for playhead
    feedback: custom(1),
}

#[derive(Clone)]
pub struct TriggerSequence {
    tooltip: String,
    sequence_control: Rcrc<ep::ComplexControl>,
    pos: (f32, f32),
    width: f32,
}

impl TriggerSequence {
    const HEIGHT: f32 = grid(1);
    const HEADER_SPACE: f32 = CORNER_SIZE * 2.0;
    // These should both be the same length.
    const TRIGGER_VALUE: &'static str = "TRUE ,";
    const NO_TRIGGER_VALUE: &'static str = "FALSE,";
    const STEP_GAP: f32 = CORNER_SIZE / 2.0;

    pub fn create(
        pos: (f32, f32),
        size: (f32, f32),
        sequence_control: Rcrc<ep::ComplexControl>,
        tooltip: String,
    ) -> TriggerSequence {
        TriggerSequence {
            tooltip,
            sequence_control,
            pos,
            width: size.0,
        }
    }
}

impl ModuleWidget for TriggerSequence {
    fn get_position(&self) -> (f32, f32) {
        self.pos
    }

    fn get_bounds(&self) -> (f32, f32) {
        (self.width, grid(1))
    }

    fn respond_to_mouse_press(
        &self,
        local_pos: (f32, f32),
        mods: &MouseMods,
        parent_pos: (f32, f32),
    ) -> MouseAction {
        let num_steps = parse_sequence_length(&self.sequence_control);
        let step_width = (self.width + TriggerSequence::STEP_GAP) / num_steps as f32;
        let clicked_step = (local_pos.0 / step_width) as usize;
        const VALUE_LEN: usize = TriggerSequence::TRIGGER_VALUE.len();
        let value_start = clicked_step * VALUE_LEN + 1;
        let borrowed = self.sequence_control.borrow();
        let new_value = if &borrowed.value[value_start..value_start + VALUE_LEN]
            == TriggerSequence::TRIGGER_VALUE
        {
            TriggerSequence::NO_TRIGGER_VALUE
        } else {
            TriggerSequence::TRIGGER_VALUE
        };
        let full_value = format!(
            "{}{}{}",
            &borrowed.value[..value_start],
            new_value,
            &borrowed.value[value_start + VALUE_LEN..]
        );
        MouseAction::SetComplexControl(Rc::clone(&self.sequence_control), full_value)
    }

    fn get_tooltip_at(&self, local_pos: (f32, f32)) -> Option<Tooltip> {
        Some(Tooltip {
            text: self.tooltip.clone(),
            interaction: InteractionHint::LeftClick.into(),
        })
    }

    fn draw(
        &self,
        g: &mut GrahpicsWrapper,
        highlight: bool,
        parent_pos: (f32, f32),
        feedback_data: &[f32],
    ) {
        g.push_state();
        g.apply_offset(self.pos.0, self.pos.1);

        const H: f32 = TriggerSequence::HEIGHT;
        const CS: f32 = CORNER_SIZE;
        const HEAD: f32 = TriggerSequence::HEADER_SPACE;
        const SG: f32 = TriggerSequence::STEP_GAP;
        g.set_color(&COLOR_BG);
        g.fill_rounded_rect(0.0, 0.0, self.width, HEAD + CS, CS);

        let num_steps = parse_sequence_length(&self.sequence_control);
        let step_width = (self.width + SG) / num_steps as f32;
        let borrowed = self.sequence_control.borrow();
        let mut current_value = &borrowed.value[1..];
        debug_assert!(
            TriggerSequence::TRIGGER_VALUE.len() == TriggerSequence::NO_TRIGGER_VALUE.len()
        );
        const VALUE_LEN: usize = TriggerSequence::TRIGGER_VALUE.len();
        for step_index in 0..num_steps {
            let x = step_index as f32 * step_width;
            if &current_value[..VALUE_LEN] == TriggerSequence::TRIGGER_VALUE {
                g.set_color(&COLOR_KNOB);
            } else {
                g.set_color(&COLOR_BG);
            }
            g.fill_rounded_rect(x, HEAD, step_width - SG, H - HEAD, CS);
            current_value = &current_value[VALUE_LEN..];
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

fn parse_sequence_length(control: &Rcrc<ep::ComplexControl>) -> usize {
    debug_assert!(TriggerSequence::TRIGGER_VALUE.len() == TriggerSequence::NO_TRIGGER_VALUE.len());
    (control.borrow().value.len() - 2) / TriggerSequence::TRIGGER_VALUE.len()
}

yaml_widget_boilerplate::make_widget_outline! {
    widget_struct: TriggerSequenceLength,
    constructor: create(
        registry: RegistryRef,
        pos: GridPos,
        sequence_control: ComplexControlRef,
        label: String,
        tooltip: String,
    ),
    complex_control_default_provider: get_defaults,
}

pub struct TriggerSequenceLength {
    base: IntBoxBase,
    sequence_control: Rcrc<ep::ComplexControl>,
}

impl TriggerSequenceLength {
    pub fn create(
        registry: &Registry,
        pos: (f32, f32),
        sequence_control: Rcrc<ep::ComplexControl>,
        label: String,
        tooltip: String,
    ) -> Self {
        Self {
            base: IntBoxBase::create(tooltip, registry, pos, (1, 99), label),
            sequence_control,
        }
    }

    fn get_defaults(
        outline: &GeneratedTriggerSequenceLengthOutline,
        yaml: &YamlNode,
    ) -> Result<Vec<(usize, String)>, String> {
        Ok(vec![(
            outline.sequence_control_index,
            "[TRUE ,FALSE,FALSE,FALSE,]".to_owned(),
        )])
    }
}

impl IntBoxImpl for TriggerSequenceLength {
    fn get_base(&self) -> &IntBoxBase {
        &self.base
    }

    fn get_current_value(&self) -> i32 {
        parse_sequence_length(&self.sequence_control) as i32
    }

    fn make_callback(&self) -> Box<dyn Fn(i32)> {
        let sequence_control = Rc::clone(&self.sequence_control);
        Box::new(move |new_length| {
            assert!(new_length >= 1);
            let new_length = new_length as usize;
            let current_length = parse_sequence_length(&sequence_control);
            let mut borrowed = sequence_control.borrow_mut();
            let current_value = &borrowed.value;
            const VALUE_LEN: usize = TriggerSequence::TRIGGER_VALUE.len();
            if new_length < current_length {
                let new_value = format!("{}]", &current_value[..1 + VALUE_LEN * new_length]);
                borrowed.value = new_value;
            } else if new_length > current_length {
                let mut new_value = String::from(&current_value[..current_value.len() - 1]);
                for _ in current_length..new_length {
                    new_value.push_str(TriggerSequence::NO_TRIGGER_VALUE);
                }
                new_value.push_str("]");
                borrowed.value = new_value;
            }
        })
    }
}
