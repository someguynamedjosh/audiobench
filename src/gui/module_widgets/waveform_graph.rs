use super::ModuleWidget;
use crate::gui::constants::*;
use crate::gui::graphics::GrahpicsWrapper;
use crate::util::*;

#[derive(Clone)]
pub struct WaveformGraph {
    pos: (f32, f32),
    size: (f32, f32),
}

impl WaveformGraph {
    pub fn create(pos: (f32, f32), size: (f32, f32)) -> Self {
        Self { pos, size }
    }
}

impl ModuleWidget for WaveformGraph {
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

        g.set_color(&COLOR_TEXT);
        let space_per_segment = self.size.0 as f32 / (feedback_data.len() - 1) as f32;
        let mut old_x = 0.0;
        let mut old_y =
            feedback_data[0].from_range_to_range(-1.0, 1.0, self.size.1 as f32, 0.0) as f32;
        for index in 1..feedback_data.len() {
            let x = (index as f32 * space_per_segment) as f32;
            let y =
                feedback_data[index].from_range_to_range(-1.0, 1.0, self.size.1 as f32, 0.0) as f32;
            g.stroke_line(old_x, old_y, x, y, 1.0);
            old_x = x;
            old_y = y;
        }

        g.pop_state();
    }
}
