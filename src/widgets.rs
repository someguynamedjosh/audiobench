use crate::graphics::{constants::*, GrahpicsWrapper};
use crate::util::RangeMap;
use std::f32::consts::PI;

// This trait is convenient to implement for widgets, but inconvenient to call.
pub trait WidgetImpl {
    fn draw(&self, g: &mut GrahpicsWrapper);
    fn get_pos(&self) -> (i32, i32);
}

// This trait is convenient to call, but inconvenient for widgets to implement.
pub trait Widget: WidgetImpl {
    fn draw(&self, g: &mut GrahpicsWrapper);
}

// All widgets with the easy-to-implement trait will also implement the easy-to-call trait.
impl<T: WidgetImpl> Widget for T {
    fn draw(&self, g: &mut GrahpicsWrapper) {
        g.push_state();
        let pos = WidgetImpl::get_pos(self);
        g.apply_offset(pos.0, pos.1);
        WidgetImpl::draw(self, g);
        g.pop_state();
    }
}

pub struct Knob {
    pub x: i32,
    pub y: i32,
    pub min: f32,
    pub max: f32,
    pub value: f32,
    pub label: String,
    pub automation: Vec<(f32, f32)>,
}

impl Default for Knob {
    fn default() -> Knob {
        Knob {
            x: 0,
            y: 0,
            min: -1.0,
            max: 1.0,
            value: 0.0,
            label: "UNLABELED".to_owned(),
            automation: Vec::new(),
        }
    }
}

impl WidgetImpl for Knob {
    fn get_pos(&self) -> (i32, i32) {
        (self.x, self.y)
    }

    fn draw(&self, g: &mut GrahpicsWrapper) {
        fn value_to_angle(slf: &Knob, value: f32) -> f32 {
            value.from_range_to_range(slf.min, slf.max, PI, 0.0)
        }

        g.set_color(&COLOR_BG);
        g.fill_pie(0, 0, GRID_2, 0, 0.0, PI);
        g.set_color(&COLOR_KNOB);
        let zero_angle = value_to_angle(self, 0.0);
        let value_angle = value_to_angle(self, self.value);
        g.fill_pie(0, 0, GRID_2, 0, zero_angle, value_angle);
        g.set_color(&COLOR_TEXT);
        g.write_label(0, GRID_1 + GRID_P, GRID_2, &self.label);

        if self.automation.len() == 0 {
            return;
        }

        let num_lanes = self.automation.len() as i32;
        let lane_size = KNOB_AUTOMATION_SPACE / num_lanes;
        let lane_size = lane_size.min(KNOB_MAX_LANE_SIZE);
        for (index, (min, max)) in self.automation.iter().enumerate() {
            if index == 1 {
                g.set_color(&COLOR_AUTOMATION_FOCUSED);
            } else {
                g.set_color(&COLOR_AUTOMATION);
            }
            let index = index as i32;
            let outer_diameter = GRID_2 - (KNOB_OUTSIDE_SPACE * 2) - lane_size * index * 2;
            let inner_diameter = outer_diameter - (lane_size - KNOB_LANE_GAP) * 2;
            let inset = (GRID_2 - outer_diameter) / 2;
            let min_angle = value_to_angle(self, *min);
            let max_angle = value_to_angle(self, *max);
            g.fill_pie(
                inset,
                inset,
                outer_diameter,
                inner_diameter,
                min_angle,
                max_angle,
            );
        }
    }
}
