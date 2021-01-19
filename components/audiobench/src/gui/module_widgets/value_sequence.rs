use super::ModuleWidgetImpl;
use crate::engine::controls::{FloatInRangeControl, UpdateRequest, ValueSequenceControl};
use crate::gui::constants::*;
use crate::gui::mouse_behaviors::{ContinuouslyMutateControl};
use crate::gui::{InteractionHint, Tooltip};
use crate::scui_config::{DropTarget, MaybeMouseBehavior, Renderer};
use scui::{MouseMods, Vec2D, WidgetImpl};
use shared_util::prelude::*;

yaml_widget_boilerplate::make_widget_outline! {
    widget_struct: ValueSequence,
    constructor: new(
        parent: ParentRef,
        pos: GridPos,
        size: GridSize,
        sequence_control: ValueSequenceControlRef,
        ramping_control: FloatInRangeControlRef,
        tooltip: String,
    ),
    // Feedback for playhead and ramping amount.
    feedback: ManualValue,
}

scui::widget! {
    pub ValueSequence
    State {
        tooltip: String,
        sequence_control: Rcrc<ValueSequenceControl>,
        ramping_control: Rcrc<FloatInRangeControl>,
        pos: Vec2D,
        size: Vec2D,
    }
}

const HEIGHT: f32 = grid(2);
const HEADER_SPACE: f32 = CORNER_SIZE * 2.0;

impl ValueSequence {
    fn new(
        parent: &impl ValueSequenceParent,
        pos: Vec2D,
        size: Vec2D,
        sequence_control: Rcrc<ValueSequenceControl>,
        ramping_control: Rcrc<FloatInRangeControl>,
        tooltip: String,
    ) -> Rc<Self> {
        let state = ValueSequenceState {
            tooltip,
            sequence_control,
            ramping_control,
            pos,
            size: size * (1, 0) + (0.0, HEIGHT),
        };
        Rc::new(Self::create(parent, state))
    }
}

impl WidgetImpl<Renderer, DropTarget> for ValueSequence {
    fn get_pos_impl(self: &Rc<Self>) -> Vec2D {
        self.state.borrow().pos
    }

    fn get_size_impl(self: &Rc<Self>) -> Vec2D {
        self.state.borrow().size
    }

    fn get_mouse_behavior_impl(
        self: &Rc<Self>,
        pos: Vec2D,
        _mods: &MouseMods,
    ) -> MaybeMouseBehavior {
        let state = self.state.borrow();
        let borrowed = state.sequence_control.borrow();
        let num_steps = borrowed.get_len();
        let step_width = state.size.x / num_steps as f32;
        let clicked_step = (pos.x / step_width) as usize;
        let mut float_value = borrowed.get_value(clicked_step);
        let cref = Rc::clone(&state.sequence_control);
        ContinuouslyMutateControl::wrap(self, move |delta, steps| {
            float_value += delta / 100.0;
            float_value = float_value.clam(-1.0, 1.0);
            let final_value = if let Some(steps) = steps {
                float_value.snap(-1.0, 1.0, steps)
            } else {
                float_value
            };
            let update = cref.borrow_mut().set_value(clicked_step, final_value);
            let tooltip = Tooltip {
                interaction: InteractionHint::SnappingModifier | InteractionHint::PrecisionModifier,
                text: format!("{:.3}", final_value),
            };
            (update, Some(tooltip))
        })
    }

    fn on_hover_impl(self: &Rc<Self>, _pos: Vec2D) -> Option<()> {
        let tooltip = Tooltip {
            text: self.state.borrow().tooltip.clone(),
            interaction: InteractionHint::LeftClickAndDrag.into(),
        };
        self.with_gui_state_mut(|state| {
            state.set_tooltip(tooltip);
        });
        Some(())
    }

    fn draw_impl(self: &Rc<Self>, g: &mut Renderer) {
        let state = self.state.borrow();
        let feedback_data: &[f32] = unimplemented!();
        assert_eq!(feedback_data.len(), 2);
        const H: f32 = HEIGHT;
        const CS: f32 = CORNER_SIZE;
        const HEAD: f32 = HEADER_SPACE;
        g.set_color(&COLOR_BG0);
        g.draw_rounded_rect((0.0, HEAD), (state.size.x, H - HEAD), CS);

        let borrowed = state.sequence_control.borrow();
        let num_steps = borrowed.get_len();
        let step_width = state.size.x / num_steps as f32;
        let ramping = feedback_data[1];
        const MIDPOINT: f32 = HEAD + (H - HEAD) * 0.5;
        let first_value = borrowed.get_value(0);
        let mut value = first_value;
        for step_index in 0..num_steps {
            let x = step_index as f32 * step_width;
            if step_index != num_steps - 1 {
                g.set_color(&COLOR_FG1);
                // g.set_alpha(0.5);
                g.draw_line((x + step_width, HEAD), (x + step_width, H), 1.0);
            }
            g.set_color(&COLOR_EDITABLE);
            let y = (0.5 - value * 0.5) * (H - HEAD) + HEAD;
            g.set_alpha(0.3);
            g.draw_rect((x, MIDPOINT.min(y)), (step_width, (MIDPOINT - y).abs()));
            g.set_alpha(1.0);
            g.draw_line((x, y), (x + step_width * (1.0 - ramping), y), 2.0);
            let next_value = if step_index < num_steps - 1 {
                borrowed.get_value(step_index + 1)
            } else {
                first_value
            };
            value = next_value;
            let next_y = (0.5 - next_value * 0.5) * (H - HEAD) + HEAD;
            g.draw_line(
                (x + step_width * (1.0 - ramping), y),
                (x + step_width, next_y),
                2.0,
            );
        }

        g.set_color(&COLOR_FG1);
        g.draw_pie(
            (feedback_data[0] * step_width - HEAD, 0.0),
            HEAD * 2.0,
            0.0,
            std::f32::consts::PI * 0.75,
            std::f32::consts::PI * 0.25,
        );

        g.pop_state();
    }
}

crate::make_int_box_widget! {
    pub ValueSequenceLength {
        sequence_control: ValueSequenceControlRef
            as Rcrc<ValueSequenceControl>
    }
}

impl ValueSequenceLength {
    fn get_range(self: &Rc<Self>) -> (i32, i32) {
        (1, 99)
    }

    fn get_current_value(&self) -> i32 {
        self.state.borrow().sequence_control.borrow().get_len() as _
    }

    fn make_callback(&self) -> Box<dyn FnMut(i32) -> UpdateRequest> {
        let sequence_control = Rc::clone(&self.state.borrow().sequence_control);
        Box::new(move |new_length| {
            assert!(new_length >= 1);
            sequence_control.borrow_mut().set_len(new_length as usize)
        })
    }
}

impl ModuleWidgetImpl for ValueSequence {}
impl ModuleWidgetImpl for ValueSequenceLength {}
