use super::ModuleWidgetImpl;
use crate::engine::static_controls as staticons;
use crate::gui::action::{MouseAction, MutateStaticon};
use crate::gui::constants::*;
use crate::gui::graphics::GrahpicsWrapper;
use crate::gui::{InteractionHint, Tooltip};
use crate::registry::Registry;
use crate::scui_config::{DropTarget, Renderer};
use scui::{MouseMods, OnClickBehavior, Vec2D, WidgetImpl};
use shared_util::prelude::*;

yaml_widget_boilerplate::make_widget_outline! {
    widget_struct: WaveformGraph,
    constructor: new(
        parent: ParentRef,
        pos: GridPos,
        size: GridSize,
    ),
    // 42 pixels wide on normal zoom, +2 for cursor.
    feedback: custom(44),
}

scui::widget! {
    pub WaveformGraph
    State {
        pos: Vec2D,
        size: Vec2D,
    }
}

impl WaveformGraph {
    fn new(parent: &impl WaveformGraphParent, pos: Vec2D, size: Vec2D) -> Rc<Self> {
        let state = WaveformGraphState { pos, size };
        Rc::new(Self::create(parent, state))
    }
}

impl WidgetImpl<Renderer, DropTarget> for WaveformGraph {
    fn get_pos_impl(self: &Rc<Self>) -> Vec2D {
        self.state.borrow().pos
    }
    fn get_size_impl(self: &Rc<Self>) -> Vec2D {
        self.state.borrow().size
    }

    fn draw_impl(self: &Rc<Self>, g: &mut Renderer) {
        let feedback_data: &[f32] = unimplemented!();
        let state = self.state.borrow();

        const CS: f32 = CORNER_SIZE;
        g.set_color(&COLOR_BG0);
        g.draw_rounded_rect(0, state.size, CS);

        g.set_color(&COLOR_FG1);
        let space_per_segment = state.size.x as f32 / (feedback_data.len() - 3) as f32;
        let mut old_point = Vec2D::new(
            0.0,
            feedback_data[0].from_range_to_range(-1.0, 1.0, state.size.y, 0.0),
        );
        for index in 1..feedback_data.len() - 2 {
            let new_point = Vec2D::new(
                index as f32 * space_per_segment,
                feedback_data[index].from_range_to_range(-1.0, 1.0, state.size.y, 0.0),
            );
            g.draw_line(old_point, new_point, 1.0);
            old_point = new_point;
        }
        let cursor_phase = feedback_data[feedback_data.len() - 2];
        let cursor_value = feedback_data[feedback_data.len() - 1];
        if cursor_phase >= 0.0 {
            let x = state.size.x * cursor_phase;
            g.draw_line((x, 0.0), (x, state.size.y), 1.0);
            let y = state.size.y * (1.0 - cursor_value) / 2.0;
            g.draw_line((0.0, y), (state.size.x, y), 1.0);
        }
    }
}

impl ModuleWidgetImpl for WaveformGraph {}
