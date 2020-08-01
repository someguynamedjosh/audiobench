use super::{ModuleWidget, PopupMenu};
use crate::engine::parts as ep;
use crate::gui::action::{DropTarget, MouseAction};
use crate::gui::constants::*;
use crate::gui::graph::{Module, WireTracker};
use crate::gui::graphics::{GrahpicsWrapper, HAlign, VAlign};
use crate::gui::{InteractionHint, MouseMods, Tooltip};
use crate::util::*;

yaml_widget_boilerplate::make_widget_outline! {
    widget_struct: HSlider,
    constructor: create(
        pos: GridPos,
        size: GridSize,
        control: AutoconRef,
        label: String,
        tooltip: String,
    ),
    feedback: control,
}

#[derive(Clone)]
pub struct HSlider {
    pos: (f32, f32),
    width: f32,
    control: Rcrc<ep::Autocon>,
    // This allows the slider to share feedback data with the right-click menu when it it open.
    value: Rcrc<f32>,
    label: String,
    tooltip: String,
}

impl HSlider {
    pub fn create(
        pos: (f32, f32),
        size: (f32, f32),
        control: Rcrc<ep::Autocon>,
        label: String,
        tooltip: String,
    ) -> HSlider {
        HSlider {
            pos,
            width: size.0,
            control,
            value: rcrc(0.0),
            label,
            tooltip,
        }
    }
}

impl ModuleWidget for HSlider {
    fn get_position(&self) -> (f32, f32) {
        self.pos
    }

    fn get_bounds(&self) -> (f32, f32) {
        (self.width, grid(1))
    }

    fn respond_to_mouse_press(
        &self,
        _local_pos: (f32, f32),
        mods: &MouseMods,
        parent_pos: (f32, f32),
    ) -> MouseAction {
        if mods.right_click {
            let pos = (
                self.pos.0 + parent_pos.0,
                self.pos.1 + parent_pos.1 + grid(1),
            );
            MouseAction::OpenMenu(Box::new(HSliderEditor::create(
                Rc::clone(&self.control),
                Rc::clone(&self.value),
                pos,
                self.width,
                self.label.clone(),
                self.tooltip.clone(),
            )))
        } else {
            MouseAction::ManipulateControl(Rc::clone(&self.control), self.control.borrow().value)
        }
    }

    fn get_drop_target_at(&self, _local_pos: (f32, f32)) -> DropTarget {
        DropTarget::Autocon(Rc::clone(&self.control))
    }

    fn get_tooltip_at(&self, _local_pos: (f32, f32)) -> Option<Tooltip> {
        Some(Tooltip {
            text: self.tooltip.clone(),
            interaction: InteractionHint::LeftClickAndDrag
                | InteractionHint::RightClick
                | InteractionHint::DoubleClick,
        })
    }

    fn add_wires(&self, wire_tracker: &mut WireTracker) {
        let (cx, cy) = (self.pos.0 + grid(2) / 2.0, self.pos.1 + grid(2) / 2.0);
        for lane in self.control.borrow().automation.iter() {
            let (module, output_index) = &lane.connection;
            let output_index = *output_index as i32;
            let module_ref = module.borrow();
            let (ox, oy) = Module::output_position(&*module_ref, output_index);
            wire_tracker.add_wire((ox, oy), (cx, cy));
        }
    }

    fn draw(
        &self,
        g: &mut GrahpicsWrapper,
        highlight: bool,
        _parent_pos: (f32, f32),
        feedback_data: &[f32],
    ) {
        g.push_state();
        const CS: f32 = CORNER_SIZE;

        let control = &*self.control.borrow();
        fn value_to_point(range: (f32, f32), width: f32, value: f32) -> f32 {
            value.from_range_to_range(range.0, range.1, 0.0, width)
        }

        g.set_color(&COLOR_TEXT);
        g.apply_offset(self.pos.0, self.pos.1);

        if highlight {
            g.set_color(&COLOR_TEXT);
        } else {
            g.set_color(&COLOR_BG);
        }
        g.fill_rounded_rect(0.0, 0.0, self.width, grid(1), CS);
        g.set_color(&COLOR_KNOB);
        if highlight {
            g.set_alpha(0.5);
        }
        let zero_point = value_to_point(control.range, self.width, 0.0);
        // If manual, show the manual value. If automated, show the most recent value recorded
        // from when a note was actually playing.
        let value = if control.automation.len() > 0 {
            feedback_data[0]
        } else {
            control.value
        };
        *self.value.borrow_mut() = value;
        let value_point = value_to_point(control.range, self.width, value);
        g.fill_rounded_rect(
            zero_point.min(value_point),
            0.0,
            (zero_point - value_point).abs(),
            grid(1),
            CS,
        );
        g.set_alpha(1.0);
        g.set_color(&COLOR_TEXT);

        if control.automation.len() > 0 {
            let num_lanes = control.automation.len() as f32;
            let lane_size = (grid(1) - CS * 2.0) / num_lanes;
            let lane_size = lane_size.min(KNOB_MAX_LANE_SIZE).max(2.0);
            for (index, lane) in control.automation.iter().enumerate() {
                g.set_color(&COLOR_AUTOMATION);
                let index = index as f32;
                let start_point = value_to_point(control.range, self.width, lane.range.0);
                let end_point = value_to_point(control.range, self.width, lane.range.1);
                g.fill_rect(
                    start_point.min(end_point),
                    // -0.5 makes it crisper at default zoom.
                    CS + (lane_size) * index - 0.5,
                    (start_point - end_point).abs(),
                    lane_size - KNOB_LANE_GAP,
                );
            }
        }

        g.pop_state();
    }
}

#[derive(Clone)]
pub struct HSliderEditor {
    control: Rcrc<ep::Autocon>,
    value: Rcrc<f32>,
    pos: (f32, f32),
    size: (f32, f32),
    label: String,
    tooltip: String,
}

impl HSliderEditor {
    fn create(
        control: Rcrc<ep::Autocon>,
        value: Rcrc<f32>,
        // Bottom left position.
        center_pos: (f32, f32),
        width: f32,
        label: String,
        tooltip: String,
    ) -> Self {
        let num_channels = control.borrow().automation.len().max(0) as f32;
        let required_height = grid(1)
            + KNOB_MENU_LANE_GAP
            + (KNOB_MENU_LANE_SIZE + KNOB_MENU_LANE_GAP) * num_channels;
        let size = (
            width + grid(4) + GRID_P * 2.0,
            required_height + GRID_P * 2.0,
        );
        Self {
            control,
            value,
            pos: (
                center_pos.0 - grid(2) - GRID_P,
                center_pos.1 - size.1 + GRID_P,
            ),
            size,
            label,
            tooltip,
        }
    }

    fn value_to_point(&self, value: f32) -> f32 {
        let range = self.control.borrow().range;
        const GP: f32 = GRID_P;
        value.from_range_to_range(range.0, range.1, 0.0, self.size.0 - (GP + grid(1)) * 2.0)
            + GP
            + grid(1)
    }
}

impl PopupMenu for HSliderEditor {
    fn get_pos(&self) -> (f32, f32) {
        self.pos
    }

    fn get_bounds(&self) -> (f32, f32) {
        self.size
    }

    fn respond_to_mouse_press(&self, local_pos: (f32, f32), mods: &MouseMods) -> MouseAction {
        const GP: f32 = GRID_P;
        const GAP: f32 = KNOB_MENU_LANE_GAP;
        const LS: f32 = KNOB_MENU_LANE_SIZE;
        let row = ((local_pos.1 - GP) / (GAP + LS)).max(0.0) as usize;
        let control_ref = self.control.borrow();
        if row >= control_ref.automation.len() {
            // Clicked the actual control...
            return MouseAction::ManipulateControl(Rc::clone(&self.control), control_ref.value);
        }
        // Lanes are rendered backwards, flip it back around.
        let lane = control_ref.automation.len() - row - 1;
        let point = local_pos.0;
        let lane_range = control_ref.automation[lane].range;
        let mut min_point = self.value_to_point(lane_range.0);
        let mut max_point = self.value_to_point(lane_range.1);
        let ends_flipped = lane_range.0 > lane_range.1;
        if ends_flipped {
            let tmp = min_point;
            min_point = max_point;
            max_point = tmp;
        }
        if point > min_point && point < max_point {
            return if mods.right_click {
                MouseAction::RemoveLane(Rc::clone(&self.control), lane)
            } else {
                MouseAction::ManipulateLane(Rc::clone(&self.control), lane)
            };
        }
        // xor
        return if (point < min_point) != ends_flipped {
            MouseAction::ManipulateLaneStart(Rc::clone(&self.control), lane, lane_range.0)
        } else {
            MouseAction::ManipulateLaneEnd(Rc::clone(&self.control), lane, lane_range.1)
        };
    }

    fn get_tooltip_at(&self, local_pos: (f32, f32)) -> Option<Tooltip> {
        const GP: f32 = GRID_P;
        const GAP: f32 = KNOB_MENU_LANE_GAP;
        const LS: f32 = KNOB_MENU_LANE_SIZE;
        let row = ((local_pos.1 - GP) / (GAP + LS)).max(0.0) as usize;
        let control_ref = self.control.borrow();
        if row >= control_ref.automation.len() {
            // Inside the actual control
            return Some(Tooltip {
                text: self.tooltip.clone(),
                interaction: InteractionHint::LeftClickAndDrag | InteractionHint::DoubleClick,
            });
        }
        // Lanes are rendered backwards, flip it back around.
        let lane = control_ref.automation.len() - row - 1;
        let point = local_pos.0;
        if point < GP + grid(1) || point > self.size.0 - GP - grid(1) {
            // Clicked outside the lane.
            return Some(Tooltip {
                text: format!(
                    "Automation lane #{}, click + drag to move one of the ends.",
                    lane + 1,
                ),
                interaction: InteractionHint::LeftClickAndDrag | InteractionHint::DoubleClick,
            });
        } else {
            return Some(Tooltip {
                text: format!(
                    "Automation lane #{}, click + drag on empty space to move one end at a time.",
                    lane + 1,
                ),
                interaction: InteractionHint::LeftClickAndDrag | InteractionHint::DoubleClick,
            });
        }
    }

    fn draw(&self, g: &mut GrahpicsWrapper) {
        g.push_state();

        g.apply_offset(self.pos.0, self.pos.1);
        const BSR: f32 = POPUP_SHADOW_RADIUS;
        const CS: f32 = CORNER_SIZE;
        const GP: f32 = GRID_P;
        g.draw_inset_box_shadow(0.0, 0.0, self.size.0, self.size.1, BSR, CS);
        g.set_color(&COLOR_SURFACE);
        g.fill_rounded_rect(0.0, 0.0, self.size.0, self.size.1, CS);

        let control = &*self.control.borrow();

        g.set_color(&COLOR_BG);
        let x = GP + grid(1);
        let width = self.size.0 - (GP + grid(1)) * 2.0;
        let boty = self.size.1 - GP - grid(1);
        g.fill_rounded_rect(x, boty, width, grid(1), CS);
        g.set_color(&COLOR_KNOB);
        let value = *self.value.borrow();
        let zero_point = self.value_to_point(0.0);
        let value_point = self.value_to_point(value);
        g.fill_rounded_rect(
            zero_point.min(value_point),
            boty,
            (zero_point - value_point).abs(),
            grid(1),
            CS,
        );

        const GAP: f32 = KNOB_MENU_LANE_GAP;
        const LS: f32 = KNOB_MENU_LANE_SIZE;
        for (index, lane) in control.automation.iter().rev().enumerate() {
            g.set_color(&COLOR_BG);
            let y = GP + (LS + GAP) * index as f32;
            g.fill_rounded_rect(x, y, width, LS, CS);
            g.set_color(&COLOR_AUTOMATION);
            let min_point = self.value_to_point(lane.range.0);
            let max_point = self.value_to_point(lane.range.1);
            let height = if lane.range.0 > lane.range.1 {
                LS / 2.0
            } else {
                LS
            };
            g.fill_rounded_rect(
                min_point.min(max_point),
                y,
                (min_point - max_point).abs(),
                height,
                CS,
            );
        }

        g.set_color(&COLOR_TEXT);
        g.write_text(
            FONT_SIZE,
            x + GP,
            boty,
            width,
            grid(1),
            HAlign::Left,
            VAlign::Center,
            1,
            &self.label,
        );
        let value_text = format!("{}{}", format_decimal(value, 3), control.suffix);
        g.write_text(
            FONT_SIZE,
            x + GP,
            boty,
            width - GP * 2.0,
            grid(1),
            HAlign::Right,
            VAlign::Center,
            1,
            &value_text,
        );

        g.pop_state();
    }
}
