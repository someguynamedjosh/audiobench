use super::ModuleWidget;
use crate::gui::constants::*;
use crate::gui::graphics::{GrahpicsWrapper, HAlign, VAlign};
use std::f32::consts::PI;

#[derive(Clone)]
pub struct EnvelopeGraph {
    pos: (f32, f32),
    size: (f32, f32),
}

impl EnvelopeGraph {
    pub(super) fn create(pos: (f32, f32), size: (f32, f32)) -> Self {
        Self { pos, size }
    }
}

impl ModuleWidget for EnvelopeGraph {
    fn get_position(&self) -> (f32, f32) {
        self.pos
    }

    fn get_bounds(&self) -> (f32, f32) {
        self.size
    }

    fn draw(
        &self,
        g: &mut GrahpicsWrapper,
        highlight: bool,
        parent_pos: (f32, f32),
        feedback_data: &[f32],
    ) {
        g.push_state();

        const CS: f32 = CORNER_SIZE;
        g.apply_offset(self.pos.0, self.pos.1);
        g.set_color(&COLOR_BG);
        g.fill_rounded_rect(0.0, 0.0, self.size.0, self.size.1, CS);
        g.apply_offset(0.0, CS);

        g.set_color(&COLOR_TEXT);
        let (a, d, s, r) = (
            feedback_data[0],
            feedback_data[1],
            feedback_data[2],
            feedback_data[3],
        );
        let total_duration = (a + d + r).max(0.2); // to prevent div0
        let w = self.size.0;
        let h = self.size.1 - CS * 2.0;
        let decay_x = (w as f32 * (a / total_duration)) as f32;
        let sustain_y = ((1.0 - s) * h as f32) as f32;
        let release_x = (w as f32 * ((a + d) / total_duration)) as f32;
        let silence_x = (w as f32 * ((a + d + r) / total_duration)) as f32;
        g.stroke_line(0.0, h, decay_x, 0.0, 2.0);
        g.stroke_line(decay_x, 0.0, release_x, sustain_y, 2.0);
        g.stroke_line(release_x, sustain_y, silence_x, h, 2.0);

        g.set_alpha(0.5);
        g.stroke_line(decay_x, -CS, decay_x, h + CS, 1.0);
        g.stroke_line(release_x, -CS, release_x, h + CS, 1.0);
        let (cx, cy) = (feedback_data[4], feedback_data[5]);
        let cx = (cx / total_duration * w as f32) as f32;
        let cy = ((-cy * 0.5 + 0.5) * h as f32) as f32;
        g.stroke_line(cx, 0.0, cx, h, 1.0);
        g.stroke_line(0.0, cy, w, cy, 1.0);
        g.set_alpha(1.0);
        const DOT_SIZE: f32 = 8.0;
        const DR: f32 = DOT_SIZE / 2.0;
        g.fill_pie(cx - DR, cy - DR, DR * 2.0, 0.0, 0.0, PI * 2.0);

        let ms = (total_duration * 1000.0) as i32;
        let ms_text = if ms > 999 {
            format!("{},{:03}ms", ms / 1000, ms % 1000)
        } else {
            format!("{}ms", ms)
        };
        g.write_text(
            FONT_SIZE,
            0.0,
            0.0,
            w,
            h,
            HAlign::Right,
            VAlign::Top,
            1,
            &ms_text,
        );

        g.pop_state();
    }
}
