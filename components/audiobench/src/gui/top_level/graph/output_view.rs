use crate::{engine::parts::JackType, gui::constants::*, scui_config::Renderer};
use scui::Vec2D;

pub trait OutputViewRenderer {
    fn draw_output_view(
        &mut self,
        data: &[f32],
        view_type: JackType,
        wire_start: Vec2D,
        wire_end: Vec2D,
    );
}

const SIZE: Vec2D = Vec2D::new(grid(5), grid(4));
const PADDING: f32 = GRID_P;

impl Renderer {
    fn draw_audio_view(&mut self, data: &[f32]) {
        self.set_color(&COLOR_FG1);
        self.translate((0.0, SIZE.y / 2.0));
        if data.len() == 0 {
            self.draw_line(0, (SIZE.x, 0.0), 1.0);
            return;
        } else if data.len() == 1 {
            let y = -data[0];
            self.draw_line((0.0, y), (SIZE.x, y), 1.0);
            return;
        }
        let step_size = SIZE.x / (data.len() as f32 - 1.0);
        for step in 0..data.len() - 1 {
            let x1 = step as f32 * step_size;
            let x2 = x1 + step_size;
            const HEIGHT: f32 = SIZE.y / 2.0;
            let y1 = -data[step] * HEIGHT;
            let y2 = -data[step + 1] * HEIGHT;
            self.draw_line((x1, y1), (x2, y2), 1.0);
        }
    }

    fn draw_pitch_view(&mut self, data: &[f32]) {
        let pitch = data.get(0).cloned().unwrap_or(0.0);
        let text = format!("{:0.2}Hz", pitch);
        self.set_color(&COLOR_FG1);
        self.draw_text(FONT_SIZE, 0, SIZE, (0, 0), 1, &text);
    }

    fn draw_trigger_view(&mut self, data: &[f32]) {
        let triggered = data.get(0).cloned().unwrap_or(0.0) > 0.5;
        if triggered {
            self.set_color(&COLOR_EDITABLE);
            self.draw_rounded_rect(0, SIZE, CORNER_SIZE);
        }
    }
}

impl OutputViewRenderer for Renderer {
    fn draw_output_view(
        &mut self,
        data: &[f32],
        view_type: JackType,
        wire_start: Vec2D,
        wire_end: Vec2D,
    ) {
        self.push_state();
        self.translate(wire_end - (0.0, SIZE.y / 2.0));

        self.draw_box_shadow(0, SIZE, MODULE_SHADOW_RADIUS);
        self.set_color(&COLOR_BG1);
        self.draw_rounded_rect(-PADDING, SIZE + PADDING * 2.0, CORNER_SIZE);
        self.set_color(&COLOR_BG0);
        self.draw_rounded_rect(0, SIZE, CORNER_SIZE);

        self.set_color(&COLOR_FG1);
        match view_type {
            JackType::Audio | JackType::Waveform => self.draw_audio_view(data),
            JackType::Pitch => self.draw_pitch_view(data),
            JackType::Trigger => self.draw_trigger_view(data),
            // _ => self.draw_text(FONT_SIZE, 0, SIZE, (0, 0), 1, &format!("{:?}", view_type)),
        }

        self.pop_state();

        self.set_color(&COLOR_FG1);
        self.draw_line(wire_start, wire_end, 1.0);
    }
}
