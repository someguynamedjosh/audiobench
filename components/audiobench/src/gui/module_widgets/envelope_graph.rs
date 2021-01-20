use crate::{
    gui::{constants::*, module_widgets::ModuleWidgetImpl},
    scui_config::{DropTarget, Renderer},
};
use scui::{Vec2D, WidgetImpl};
use shared_util::prelude::*;
use std::{convert::TryInto, f32::consts::PI};

yaml_widget_boilerplate::make_widget_outline! {
    widget_struct: EnvelopeGraph,
    constructor: new(
        parent: ParentRef,
        pos: GridPos,
        size: GridSize,
    ),
    // 4 for parameters, 2 for cursor.
    feedback: ManualValue,
}

scui::widget! {
    pub EnvelopeGraph
    State {
        pos: Vec2D,
        size: Vec2D,
        feedback: Option<[f32; 6]>,
    }
}

impl EnvelopeGraph {
    fn new(parent: &impl EnvelopeGraphParent, pos: Vec2D, size: Vec2D) -> Rc<Self> {
        let state = EnvelopeGraphState {
            pos,
            size,
            feedback: None,
        };
        Rc::new(Self::create(parent, state))
    }
}

const BLANK_FEEDBACK: [f32; 6] = [0.0, 0.1, 1.0, 0.2, 0.0, -1.0];

impl WidgetImpl<Renderer, DropTarget> for EnvelopeGraph {
    fn get_pos_impl(self: &Rc<Self>) -> Vec2D {
        self.state.borrow().pos
    }

    fn get_size_impl(self: &Rc<Self>) -> Vec2D {
        self.state.borrow().size
    }

    fn draw_impl(self: &Rc<Self>, g: &mut Renderer) {
        let state = self.state.borrow();
        let feedback_data = state.feedback.as_ref().unwrap_or(&BLANK_FEEDBACK);

        const CS: f32 = CORNER_SIZE;
        g.set_color(&COLOR_BG0);
        g.draw_rounded_rect(0, state.size, CS);
        g.translate((0.0, CS));

        g.set_color(&COLOR_FG1);
        let (a, d, s, r) = (
            feedback_data[0],
            feedback_data[1],
            feedback_data[2],
            feedback_data[3],
        );
        let total_duration = (a + d + r).max(0.2); // to prevent div0
        let w = state.size.x;
        let h = state.size.y - CS * 2.0;
        let decay_x = (w as f32 * (a / total_duration)) as f32;
        let sustain_y = ((1.0 - s) * h as f32) as f32;
        let release_x = (w as f32 * ((a + d) / total_duration)) as f32;
        let silence_x = (w as f32 * ((a + d + r) / total_duration)) as f32;
        g.draw_line((0.0, h), (decay_x, 0.0), 2.0);
        g.draw_line((decay_x, 0.0), (release_x, sustain_y), 2.0);
        g.draw_line((release_x, sustain_y), (silence_x, h), 2.0);

        g.set_alpha(0.5);
        g.draw_line((decay_x, -CS), (decay_x, h + CS), 1.0);
        g.draw_line((release_x, -CS), (release_x, h + CS), 1.0);
        let cursor_pos = Vec2D::new(
            feedback_data[4] / total_duration * w,
            (-feedback_data[5] * 0.5 + 0.5) * h,
        );
        g.draw_line((cursor_pos.x, 0.0), (cursor_pos.x, h), 1.0);
        g.draw_line((0.0, cursor_pos.y), (w, cursor_pos.y), 1.0);
        g.set_alpha(1.0);
        const DOT_SIZE: f32 = 8.0;
        const DR: f32 = DOT_SIZE / 2.0;
        g.draw_pie(cursor_pos - DR, DR * 2.0, 0.0, 0.0, PI * 2.0);

        let ms = (total_duration * 1000.0) as i32;
        let ms_text = if ms > 999 {
            format!("{},{:03}ms", ms / 1000, ms % 1000)
        } else {
            format!("{}ms", ms)
        };
        g.draw_text(FONT_SIZE, 0, (w, h), (1, -1), 1, &ms_text);
    }
}

impl ModuleWidgetImpl for EnvelopeGraph {
    fn take_feedback_data(self: &Rc<Self>, data: Vec<f32>) {
        assert_eq!(data.len(), 6);
        self.state.borrow_mut().feedback = Some(data.try_into().unwrap());
    }
}
