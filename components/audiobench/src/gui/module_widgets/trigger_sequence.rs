use super::ModuleWidgetImpl;
use crate::engine::controls::{TriggerSequenceControl, UpdateRequest};
use crate::gui::constants::*;
use crate::gui::mouse_behaviors::MutateControl;
use crate::gui::{InteractionHint, Tooltip};
use crate::scui_config::{DropTarget, MaybeMouseBehavior, Renderer};
use scui::{MouseMods, Vec2D, WidgetImpl};
use shared_util::prelude::*;

yaml_widget_boilerplate::make_widget_outline! {
    widget_struct: TriggerSequence,
    constructor: new(
        parent: ParentRef,
        pos: GridPos,
        size: GridSize,
        sequence_control: TriggerSequenceControlRef,
        tooltip: String,
    ),
    // Feedback for playhead
    feedback: ManualValue,
}

scui::widget! {
    pub TriggerSequence
    State {
        pos: Vec2D,
        size: Vec2D,
        sequence_control: Rcrc<TriggerSequenceControl>,
        tooltip: String,
    }
}

const HEIGHT: f32 = grid(1);
const HEADER_SPACE: f32 = CORNER_SIZE * 2.0;
const STEP_GAP: f32 = CORNER_SIZE / 2.0;

impl TriggerSequence {
    fn new(
        parent: &impl TriggerSequenceParent,
        pos: Vec2D,
        size: Vec2D,
        sequence_control: Rcrc<TriggerSequenceControl>,
        tooltip: String,
    ) -> Rc<Self> {
        let state = TriggerSequenceState {
            tooltip,
            sequence_control,
            pos,
            size: size * (1, 0) + (0.0, HEIGHT),
        };
        Rc::new(Self::create(parent, state))
    }
}

impl WidgetImpl<Renderer, DropTarget> for TriggerSequence {
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
        let num_steps = state.sequence_control.borrow().get_len();
        let step_width = (state.size.x + STEP_GAP) / num_steps as f32;
        let clicked_step = (pos.x / step_width) as usize;
        let cref = Rc::clone(&state.sequence_control);
        MutateControl::wrap(self, move || cref.borrow_mut().toggle_trigger(clicked_step))
    }

    fn on_hover_impl(self: &Rc<Self>, _pos: Vec2D) -> Option<()> {
        let tooltip = Tooltip {
            text: self.state.borrow().tooltip.clone(),
            interaction: InteractionHint::LeftClick.into(),
        };
        self.with_gui_state_mut(|state| {
            state.set_tooltip(tooltip);
        });
        Some(())
    }

    fn draw_impl(self: &Rc<Self>, g: &mut Renderer) {
        let state = self.state.borrow();
        let feedback_data: &[f32] = unimplemented!();
        const H: f32 = HEIGHT;
        const CS: f32 = CORNER_SIZE;
        const HEAD: f32 = HEADER_SPACE;
        const SG: f32 = STEP_GAP;
        g.set_color(&COLOR_BG0);

        let borrowed = state.sequence_control.borrow();
        let num_steps = borrowed.get_len();
        let step_width = (state.size.x + SG) / num_steps as f32;
        for step_index in 0..num_steps {
            let x = step_index as f32 * step_width;
            if borrowed.get_trigger(step_index) {
                g.set_color(&COLOR_EDITABLE);
            } else {
                g.set_color(&COLOR_BG0);
            }
            g.draw_rounded_rect((x, HEAD), (step_width - SG, H - HEAD), CS);
        }

        g.set_color(&COLOR_FG1);
        g.draw_pie(
            (feedback_data[0] * step_width - HEAD, 0.0),
            HEAD * 2.0,
            0.0,
            std::f32::consts::PI * 0.75,
            std::f32::consts::PI * 0.25,
        );
    }
}

crate::make_int_box_widget! {
    pub TriggerSequenceLength {
        sequence_control: TriggerSequenceControlRef
            as Rcrc<TriggerSequenceControl>
    }
}

impl TriggerSequenceLength {
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

impl ModuleWidgetImpl for TriggerSequence {}
impl ModuleWidgetImpl for TriggerSequenceLength {}
