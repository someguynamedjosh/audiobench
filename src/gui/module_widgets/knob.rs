use super::ModuleWidget;
use crate::engine::parts as ep;
use crate::gui::action::{DropTarget, MouseAction};
use crate::gui::constants::*;
use crate::gui::graph::{Module, WireTracker};
use crate::gui::graphics::{GrahpicsWrapper, HAlign, VAlign};
use crate::gui::{InteractionHint, MouseMods, Tooltip};
use crate::util::*;
use std::f32::consts::PI;

#[derive(Clone)]
pub struct Knob {
    tooltip: String,
    control: Rcrc<ep::Control>,
    // This allows the knob to share feedback data with the right-click menu when it it open.
    value: Rcrc<f32>,
    pos: (f32, f32),
    label: String,
}

impl Knob {
    pub fn create(
        tooltip: String,
        control: Rcrc<ep::Control>,
        pos: (f32, f32),
        label: String,
    ) -> Knob {
        Knob {
            tooltip,
            control,
            value: rcrc(0.0),
            pos,
            label,
        }
    }
}

impl ModuleWidget for Knob {
    fn get_position(&self) -> (f32, f32) {
        self.pos
    }

    fn get_bounds(&self) -> (f32, f32) {
        (grid(2), grid(2))
    }

    fn respond_to_mouse_press(
        &self,
        local_pos: (f32, f32),
        mods: &MouseMods,
        parent_pos: (f32, f32),
    ) -> MouseAction {
        if mods.right_click {
            let pos = (
                self.pos.0 + parent_pos.0 + grid(2) / 2.0,
                self.pos.1 + parent_pos.1 + grid(2) / 2.0,
            );
            MouseAction::OpenMenu(Box::new(KnobEditor::create(
                Rc::clone(&self.control),
                Rc::clone(&self.value),
                pos,
                self.label.clone(),
                self.tooltip.clone(),
            )))
        } else {
            MouseAction::ManipulateControl(Rc::clone(&self.control), self.control.borrow().value)
        }
    }

    fn get_drop_target_at(&self, local_pos: (f32, f32)) -> DropTarget {
        DropTarget::Control(Rc::clone(&self.control))
    }

    fn get_tooltip_at(&self, local_pos: (f32, f32)) -> Option<Tooltip> {
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
        parent_pos: (f32, f32),
        feedback_data: &[f32],
    ) {
        g.push_state();
        const MIN_ANGLE: f32 = PI * 1.10;
        const MAX_ANGLE: f32 = -PI * 0.10;

        let control = &*self.control.borrow();
        fn value_to_angle(range: (f32, f32), value: f32) -> f32 {
            value.from_range_to_range(range.0, range.1, MIN_ANGLE, MAX_ANGLE)
        }

        g.set_color(&COLOR_TEXT);
        g.apply_offset(self.pos.0, self.pos.1);

        if highlight {
            g.set_color(&COLOR_TEXT);
        } else {
            g.set_color(&COLOR_BG);
        }
        g.fill_pie(
            0.0,
            0.0,
            grid(2),
            KNOB_INSIDE_SPACE * 2.0,
            MIN_ANGLE,
            MAX_ANGLE,
        );
        g.set_color(&COLOR_KNOB);
        if highlight {
            g.set_alpha(0.5);
        }
        let zero_angle = value_to_angle(control.range, 0.0);
        // If manual, show the manual value. If automated, show the most recent value recorded
        // from when a note was actually playing.
        let value = if control.automation.len() > 0 {
            feedback_data[0]
        } else {
            control.value
        };
        *self.value.borrow_mut() = value;
        let value_angle = value_to_angle(control.range, value);
        g.fill_pie(
            0.0,
            0.0,
            grid(2),
            KNOB_INSIDE_SPACE * 2.0,
            zero_angle.clam(MAX_ANGLE, MIN_ANGLE),
            value_angle,
        );
        g.set_alpha(1.0);
        g.set_color(&COLOR_TEXT);
        const H: HAlign = HAlign::Center;
        const V: VAlign = VAlign::Bottom;
        g.write_text(FONT_SIZE, 0.0, 0.0, grid(2), grid(2), H, V, 1, &self.label);

        if control.automation.len() > 0 {
            let num_lanes = control.automation.len() as f32;
            let lane_size = KNOB_AUTOMATION_SPACE / num_lanes;
            let lane_size = lane_size.min(KNOB_MAX_LANE_SIZE).max(2.0);
            for (index, lane) in control.automation.iter().enumerate() {
                g.set_color(&COLOR_AUTOMATION);
                let index = index as f32;
                let outer_diameter = grid(2) - (KNOB_OUTSIDE_SPACE * 2.0) - lane_size * index * 2.0;
                let inner_diameter = outer_diameter - (lane_size - KNOB_LANE_GAP) * 2.0;
                let inset = (grid(2) - outer_diameter) / 2.0;
                let min_angle = value_to_angle(control.range, lane.range.0);
                let max_angle = value_to_angle(control.range, lane.range.1);
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

        g.pop_state();
    }
}

#[derive(Clone)]
pub struct KnobEditor {
    control: Rcrc<ep::Control>,
    value: Rcrc<f32>,
    pos: (f32, f32),
    size: (f32, f32),
    label: String,
    tooltip: String,
}

impl KnobEditor {
    fn create(
        control: Rcrc<ep::Control>,
        value: Rcrc<f32>,
        center_pos: (f32, f32),
        label: String,
        tooltip: String,
    ) -> Self {
        let num_channels = control.borrow().automation.len().max(2) as f32;
        let required_radius =
            (KNOB_MENU_LANE_SIZE + KNOB_MENU_LANE_GAP) * num_channels + KNOB_MENU_KNOB_OR + GRID_P;
        let size = (required_radius * 2.0, required_radius + fatgrid(1));
        Self {
            control,
            value,
            pos: (center_pos.0 - size.0 / 2.0, center_pos.1 - size.1 / 2.0),
            size,
            label,
            tooltip,
        }
    }

    pub(in crate::gui) fn get_pos(&self) -> (f32, f32) {
        self.pos
    }

    pub(in crate::gui) fn get_bounds(&self) -> (f32, f32) {
        self.size
    }

    pub(in crate::gui) fn respond_to_mouse_press(
        &self,
        local_pos: (f32, f32),
        mods: &MouseMods,
    ) -> MouseAction {
        // Yes, the last 0 is intentional. The center of the knob is not vertically centered.
        let (cx, cy) = (
            local_pos.0 - self.size.0 / 2.0,
            local_pos.1 - self.size.0 / 2.0,
        );
        // y coordinate is inverted from how it appears on screen.
        let (fcx, fcy) = (cx as f32, -cy as f32);
        let (angle, radius) = (fcy.atan2(fcx), (fcy * fcy + fcx * fcx).sqrt());
        let control = &*self.control.borrow();
        let auto_lanes = control.automation.len();
        // Clicked somewhere in the top "half" where the main knob and automation lanes are.
        if angle >= 0.0 && angle <= PI {
            let radius = radius as f32;
            if radius < KNOB_MENU_KNOB_IR {
                // Nothing interactable inside the knob.
            } else if radius < KNOB_MENU_KNOB_OR {
                return MouseAction::ManipulateControl(
                    Rc::clone(&self.control),
                    self.control.borrow().value,
                );
            } else {
                let radius = radius - KNOB_MENU_KNOB_OR;
                let lane = (radius / (KNOB_MENU_LANE_SIZE + KNOB_MENU_LANE_GAP)) as usize;
                if lane < auto_lanes {
                    // It's rendered backwards so we need to flip the index to make it visually
                    // match up.
                    let lane = auto_lanes - lane - 1;
                    let range = control.range;
                    let lane_range = control.automation[lane].range;
                    let mut min_angle = lane_range.0.from_range_to_range(range.0, range.1, PI, 0.0);
                    let mut max_angle = lane_range.1.from_range_to_range(range.0, range.1, PI, 0.0);
                    let ends_flipped = lane_range.0 > lane_range.1;
                    if ends_flipped {
                        let tmp = min_angle;
                        min_angle = max_angle;
                        max_angle = tmp;
                    }
                    if angle < min_angle && angle > max_angle {
                        return if mods.right_click {
                            MouseAction::RemoveLane(Rc::clone(&self.control), lane)
                        } else {
                            MouseAction::ManipulateLane(Rc::clone(&self.control), lane)
                        };
                    }
                    // xor
                    return if (angle > min_angle) != ends_flipped {
                        MouseAction::ManipulateLaneStart(
                            Rc::clone(&self.control),
                            lane,
                            lane_range.0,
                        )
                    } else {
                        MouseAction::ManipulateLaneEnd(Rc::clone(&self.control), lane, lane_range.1)
                    };
                }
            }
        } else {
            // If we clicked under one of the automation lanes...
            if cy > 0.0 && fcx.abs() > KNOB_MENU_KNOB_OR {
                let lane = ((fcx.abs() - KNOB_MENU_KNOB_OR)
                    / (KNOB_MENU_LANE_SIZE + KNOB_MENU_LANE_GAP))
                    as usize;
                if lane >= auto_lanes {
                    return MouseAction::None;
                }
                let lane_range = control.automation[lane].range;
                let ends_flipped = lane_range.0 > lane_range.1;
                // xor
                return if (fcx > 0.0) != ends_flipped {
                    MouseAction::ManipulateLaneEnd(Rc::clone(&self.control), lane, lane_range.1)
                } else {
                    MouseAction::ManipulateLaneStart(Rc::clone(&self.control), lane, lane_range.0)
                };
            }
        }
        MouseAction::None
    }

    pub(in crate::gui) fn get_tooltip_at(&self, local_pos: (f32, f32)) -> Option<Tooltip> {
        // Yes, the last 0 is intentional. The center of the knob is not vertically centered.
        let (cx, cy) = (
            local_pos.0 - self.size.0 / 2.0,
            local_pos.1 - self.size.0 / 2.0,
        );
        // y coordinate is inverted from how it appears on screen.
        let (fcx, fcy) = (cx as f32, -cy as f32);
        let (angle, radius) = (fcy.atan2(fcx), (fcy * fcy + fcx * fcx).sqrt());
        let control = &*self.control.borrow();
        let auto_lanes = control.automation.len();
        // Clicked somewhere in the top "half" where the main knob and automation lanes are.
        if !(angle >= 0.0 && angle <= PI) {
            // If we clicked under one of the automation lanes...
            if cy > 0.0 && fcx.abs() > KNOB_MENU_KNOB_OR {
                let lane = ((fcx.abs() - KNOB_MENU_KNOB_OR)
                    / (KNOB_MENU_LANE_SIZE + KNOB_MENU_LANE_GAP))
                    as usize;
                if lane < auto_lanes {
                    return Some(Tooltip {
                        text: format!(
                            "Automation lane #{}, click + drag to move one of the ends.",
                            lane + 1,
                        ),
                        interaction: InteractionHint::LeftClickAndDrag
                            | InteractionHint::DoubleClick,
                    });
                }
            }
            return None;
        }
        let radius = radius as f32;
        if radius < KNOB_MENU_KNOB_IR {
            return None;
            // Nothing interactable inside the knob.
        }
        if radius < KNOB_MENU_KNOB_OR {
            return Some(Tooltip {
                text: self.tooltip.clone(),
                interaction: InteractionHint::LeftClickAndDrag | InteractionHint::DoubleClick,
            });
        }
        let radius = radius - KNOB_MENU_KNOB_OR;
        let lane = (radius / (KNOB_MENU_LANE_SIZE + KNOB_MENU_LANE_GAP)) as usize;
        if lane < auto_lanes {
            return Some(Tooltip {
                text: format!(
                    "Automation lane #{}, click + drag on empty space to move one end at a time.",
                    lane + 1,
                ),
                interaction: InteractionHint::LeftClickAndDrag | InteractionHint::DoubleClick,
            });
        }
        None
    }

    pub(in crate::gui) fn draw(&self, g: &mut GrahpicsWrapper) {
        g.push_state();

        g.apply_offset(self.pos.0, self.pos.1);
        const BSR: f32 = POPUP_SHADOW_RADIUS;
        const CS: f32 = CORNER_SIZE;
        g.draw_inset_box_shadow(0.0, 0.0, self.size.0, self.size.1, BSR, CS);
        g.set_color(&COLOR_SURFACE);
        g.fill_rounded_rect(0.0, 0.0, self.size.0, self.size.1, CS);

        let control = &*self.control.borrow();
        fn value_to_angle(range: (f32, f32), value: f32) -> f32 {
            value.from_range_to_range(range.0, range.1, PI, 0.0)
        }
        g.apply_offset(self.size.0 / 2.0, self.size.1 - fatgrid(1));

        const KOR: f32 = KNOB_MENU_KNOB_OR;
        const KIR: f32 = KNOB_MENU_KNOB_IR;
        g.set_color(&COLOR_BG);
        g.fill_pie(-KOR, -KOR, KOR * 2.0, KIR * 2.0, PI, 0.0);
        g.set_color(&COLOR_KNOB);
        let zero_angle = value_to_angle(control.range, 0.0);
        let value = *self.value.borrow();
        let value_angle = value_to_angle(control.range, value);
        g.fill_pie(
            -KOR,
            -KOR,
            KOR * 2.0,
            KIR * 2.0,
            zero_angle.clam(0.0, PI),
            value_angle,
        );

        const GAP: f32 = KNOB_MENU_LANE_GAP;
        const LS: f32 = KNOB_MENU_LANE_SIZE;
        for (index, lane) in control.automation.iter().rev().enumerate() {
            let ir = KOR + GAP + (GAP + LS) * index as f32;
            let or = ir + LS;
            g.set_color(&COLOR_BG);
            g.fill_pie(-or, -or, or * 2.0, ir * 2.0, PI, 0.0);
            g.set_color(&COLOR_AUTOMATION);
            let min_angle = value_to_angle(control.range, lane.range.0);
            let max_angle = value_to_angle(control.range, lane.range.1);
            let ir = if lane.range.0 > lane.range.1 {
                ir + LS / 2.0
            } else {
                ir
            };
            g.fill_pie(-or, -or, or * 2.0, ir * 2.0, min_angle, max_angle);
        }

        g.set_color(&COLOR_TEXT);
        let value_text = format!("{}{}", format_decimal(value, 3), control.suffix);
        g.write_label(-KIR, -12.0, KIR * 2.0, &value_text);
        g.write_label(-KOR, GRID_P, KOR * 2.0, &self.label);

        g.pop_state();
    }
}
