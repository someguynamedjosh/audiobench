use crate::{
    engine::controls::{Control, TriggerSequenceControl, UpdateRequest},
    gui::{
        constants::*, module_widgets::ModuleWidgetImpl, mouse_behaviors::MutateControl,
        InteractionHint, Tooltip,
    },
    scui_config::{DropTarget, MaybeMouseBehavior, Renderer},
};
use scui::{MouseMods, Vec2D, WidgetImpl};
use shared_util::prelude::*;

yaml_widget_boilerplate::make_widget_outline! {
    widget_struct: TriggerSequence,
    constructor: new(
        parent: ParentRef,
        pos: GridPos,
        size: GridSize,
        control: TriggerSequenceControlRef,
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
        control: Rcrc<TriggerSequenceControl>,
        tooltip: String,
        cursor_pos: f32,
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
        control: Rcrc<TriggerSequenceControl>,
        tooltip: String,
    ) -> Rc<Self> {
        let state = TriggerSequenceState {
            tooltip,
            control,
            pos,
            size: size * (1, 0) + (0.0, HEIGHT),
            cursor_pos: 0.0,
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
        let num_steps = state.control.borrow().get_len();
        let step_width = (state.size.x + STEP_GAP) / num_steps as f32;
        let clicked_step = (pos.x / step_width) as usize;
        let cref = Rc::clone(&state.control);
        MutateControl::wrap(self, move || cref.borrow_mut().toggle_trigger(clicked_step))
    }

    fn on_hover_impl(self: &Rc<Self>, _pos: Vec2D) -> Option<()> {
        let tooltip = Tooltip {
            text: self.state.borrow().tooltip.clone(),
            interaction: vec![InteractionHint::LeftClick],
        };
        self.with_gui_state_mut(|state| {
            state.set_tooltip(tooltip);
        });
        Some(())
    }

    fn draw_impl(self: &Rc<Self>, g: &mut Renderer) {
        let state = self.state.borrow();
        const H: f32 = HEIGHT;
        const CS: f32 = CORNER_SIZE;
        const HEAD: f32 = HEADER_SPACE;
        const SG: f32 = STEP_GAP;
        g.set_color(&COLOR_BG0);

        let borrowed = state.control.borrow();
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
            (state.cursor_pos * step_width - HEAD, 0.0),
            HEAD * 2.0,
            0.0,
            std::f32::consts::PI * 0.75,
            std::f32::consts::PI * 0.25,
        );
    }
}

impl ModuleWidgetImpl for TriggerSequence {
    fn represented_control(self: &Rc<Self>) -> Option<Rcrc<dyn Control>> {
        Some(Rc::clone(&self.state.borrow().control) as _)
    }

    fn take_feedback_data(self: &Rc<Self>, data: Vec<f32>) {
        assert_eq! {data.len(), 1};
        self.state.borrow_mut().cursor_pos = data[0];
    }
}

crate::make_int_box_widget! {
    pub TriggerSequenceLength {
        control: TriggerSequenceControlRef
            as Rcrc<TriggerSequenceControl>
    }
}

impl TriggerSequenceLength {
    fn get_range(self: &Rc<Self>) -> (i32, i32) {
        (1, 99)
    }

    fn get_current_value(&self) -> i32 {
        self.state.borrow().control.borrow().get_len() as _
    }

    fn make_callback(&self) -> Box<dyn FnMut(i32) -> UpdateRequest> {
        let control = Rc::clone(&self.state.borrow().control);
        Box::new(move |new_length| {
            assert!(new_length >= 1);
            control.borrow_mut().set_len(new_length as usize)
        })
    }
}

impl ModuleWidgetImpl for TriggerSequenceLength {}
