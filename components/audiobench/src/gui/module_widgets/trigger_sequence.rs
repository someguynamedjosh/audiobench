use super::ModuleWidget;
use super::{IntBoxBase, IntBoxImpl};
use crate::engine::static_controls as staticons;
use crate::gui::action::MouseAction;
use crate::gui::constants::*;
use crate::gui::graphics::GrahpicsWrapper;
use crate::gui::{InteractionHint, MouseMods, Tooltip};
use crate::registry::Registry;
use shared_util::prelude::*;

yaml_widget_boilerplate::make_widget_outline! {
    widget_struct: TriggerSequence,
    constructor: create(
        pos: GridPos,
        size: GridSize,
        sequence_control: ControlledTriggerSequenceRef,
        tooltip: String,
    ),
    // Feedback for playhead
    feedback: custom(1),
}

#[derive(Clone)]
pub struct TriggerSequence {
    tooltip: String,
    sequence_control: Rcrc<staticons::ControlledTriggerSequence>,
    pos: (f32, f32),
    width: f32,
}

impl TriggerSequence {
    const HEIGHT: f32 = grid(1);
    const HEADER_SPACE: f32 = CORNER_SIZE * 2.0;
    const STEP_GAP: f32 = CORNER_SIZE / 2.0;

    pub fn create(
        pos: (f32, f32),
        size: (f32, f32),
        sequence_control: Rcrc<staticons::ControlledTriggerSequence>,
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
        _mods: &MouseMods,
        _parent_pos: (f32, f32),
    ) -> MouseAction {
        let num_steps = self.sequence_control.borrow().get_len();
        let step_width = (self.width + TriggerSequence::STEP_GAP) / num_steps as f32;
        let clicked_step = (local_pos.0 / step_width) as usize;
        let cref = Rc::clone(&self.sequence_control);
        MouseAction::MutateStaticon(Box::new(move || {
            cref.borrow_mut().toggle_trigger(clicked_step)
        }))
    }

    fn get_tooltip_at(&self, _local_pos: (f32, f32)) -> Option<Tooltip> {
        Some(Tooltip {
            text: self.tooltip.clone(),
            interaction: InteractionHint::LeftClick.into(),
        })
    }

    fn draw(
        &self,
        g: &mut GrahpicsWrapper,
        _highlight: bool,
        _parent_pos: (f32, f32),
        feedback_data: &[f32],
    ) {
        g.push_state();
        g.apply_offset(self.pos.0, self.pos.1);

        const H: f32 = TriggerSequence::HEIGHT;
        const CS: f32 = CORNER_SIZE;
        const HEAD: f32 = TriggerSequence::HEADER_SPACE;
        const SG: f32 = TriggerSequence::STEP_GAP;
        g.set_color(&COLOR_BG0);

        let borrowed = self.sequence_control.borrow();
        let num_steps = borrowed.get_len();
        let step_width = (self.width + SG) / num_steps as f32;
        for step_index in 0..num_steps {
            let x = step_index as f32 * step_width;
            if borrowed.get_trigger(step_index) {
                g.set_color(&COLOR_EDITABLE);
            } else {
                g.set_color(&COLOR_BG0);
            }
            g.fill_rounded_rect(x, HEAD, step_width - SG, H - HEAD, CS);
        }

        g.set_color(&COLOR_FG1);
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
    widget_struct: TriggerSequenceLength,
    constructor: create(
        registry: RegistryRef,
        pos: GridPos,
        sequence_control: ControlledTriggerSequenceRef,
        label: String,
        tooltip: String,
    ),
}

pub struct TriggerSequenceLength {
    base: IntBoxBase,
    sequence_control: Rcrc<staticons::ControlledTriggerSequence>,
}

impl TriggerSequenceLength {
    pub fn create(
        registry: &Registry,
        pos: (f32, f32),
        sequence_control: Rcrc<staticons::ControlledTriggerSequence>,
        label: String,
        tooltip: String,
    ) -> Self {
        Self {
            base: IntBoxBase::create(tooltip, registry, pos, (1, 99), label),
            sequence_control,
        }
    }
}

impl IntBoxImpl for TriggerSequenceLength {
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
