use super::{ModuleWidget, KnobEditor};
use crate::engine::parts as ep;
use crate::gui::action::{DropTarget, MouseAction};
use crate::gui::constants::*;
use crate::gui::graph::{Module, WireTracker};
use crate::gui::graphics::{GrahpicsWrapper, HAlign, VAlign};
use crate::gui::{InteractionHint, MouseMods, Tooltip};
use shared_util::prelude::*;
use std::f32::consts::PI;

yaml_widget_boilerplate::make_widget_outline! {
    widget_struct: MiniKnob,
    constructor: create(
        pos: GridPos,
        control: AutoconRef,
        label: String,
        tooltip: String,
    ),
    feedback: control,
}

#[derive(Clone)]
pub struct MiniKnob {
    control: Rcrc<ep::Autocon>,
    // This allows the knob to share feedback data with the right-click menu when it it open.
    value: Rcrc<f32>,
    pos: (f32, f32),
    label: String,
    tooltip: String,
}

impl MiniKnob {
    pub fn create(
        pos: (f32, f32),
        control: Rcrc<ep::Autocon>,
        label: String,
        tooltip: String,
    ) -> MiniKnob {
        MiniKnob {
            tooltip,
            control,
            value: rcrc(0.0),
            pos,
            label,
        }
    }
}

impl ModuleWidget for MiniKnob {
    fn get_position(&self) -> (f32, f32) {
        self.pos
    }

    fn get_bounds(&self) -> (f32, f32) {
        (grid(1), grid(1))
    }

    fn respond_to_mouse_press(
        &self,
        _local_pos: (f32, f32),
        mods: &MouseMods,
        parent_pos: (f32, f32),
    ) -> MouseAction {
        if mods.right_click {
            let pos = (
                self.pos.0 + parent_pos.0 + grid(1) / 2.0,
                self.pos.1 + parent_pos.1 + grid(1) / 2.0,
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
        let (cx, cy) = (self.pos.0 + grid(1) / 2.0, self.pos.1 + grid(1) / 2.0);
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
        const MIN_ANGLE: f32 = PI * 1.50;
        const MAX_ANGLE: f32 = -PI * 0.50;

        let control = &*self.control.borrow();
        fn value_to_angle(range: (f32, f32), value: f32) -> f32 {
            value.from_range_to_range(range.0, range.1, MIN_ANGLE, MAX_ANGLE)
        }

        g.set_color(&COLOR_FG1);
        g.apply_offset(self.pos.0, self.pos.1);

        if highlight {
            g.set_color(&COLOR_FG1);
        } else {
            g.set_color(&COLOR_BG0);
        }
        g.fill_pie(
            0.0,
            0.0,
            grid(1),
            KNOB_INSIDE_SPACE,
            MIN_ANGLE,
            MAX_ANGLE,
        );
        g.set_color(&COLOR_EDITABLE);
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
            grid(1),
            KNOB_INSIDE_SPACE,
            zero_angle.clam(MAX_ANGLE, MIN_ANGLE),
            value_angle,
        );
        g.set_alpha(1.0);

        if control.automation.len() > 0 {
            let num_lanes = control.automation.len() as f32;
            let lane_size = KNOB_AUTOMATION_SPACE / num_lanes;
            let lane_size = lane_size.min(KNOB_MAX_LANE_SIZE).max(2.0);
            for (index, lane) in control.automation.iter().enumerate() {
                g.set_color(&COLOR_AUTOMATION);
                let index = index as f32;
                let outer_diameter = grid(1) - (KNOB_OUTSIDE_SPACE * 2.0) - lane_size * index * 2.0;
                let inner_diameter = outer_diameter - (lane_size - KNOB_LANE_GAP) * 2.0;
                let inset = (grid(1) - outer_diameter) / 2.0;
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