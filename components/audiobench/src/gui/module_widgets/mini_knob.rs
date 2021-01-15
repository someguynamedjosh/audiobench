use super::ModuleWidgetImpl;
use crate::engine::controls::FloatInRangeControl;
use crate::engine::parts as ep;
use crate::gui::constants::*;
use crate::gui::module_widgets::KnobEditor;
use crate::gui::mouse_behaviors::ManipulateControl;
use crate::gui::top_level::graph::{Module, ModuleGraph, WireTracker};
use crate::gui::{InteractionHint, Tooltip};
use crate::scui_config::{DropTarget, MaybeMouseBehavior, Renderer};
use scui::{MouseMods, OnClickBehavior, Vec2D, Widget, WidgetImpl};
use shared_util::prelude::*;
use std::f32::consts::PI;

yaml_widget_boilerplate::make_widget_outline! {
    widget_struct: MiniKnob,
    constructor: new(
        parent: ParentRef,
        pos: GridPos,
        control: FloatInRangeControlRef,
        label: String,
        tooltip: String,
    ),
    feedback: control,
}

scui::widget! {
    pub MiniKnob
    State {
        pos: Vec2D,
        control: Rcrc<FloatInRangeControl>,
        // This allows the knob to share feedback data with the right-click menu when it it open.
        value: Rcrc<f32>,
        label: String,
        tooltip: String,
    }
    Parents {
        graph: Rc<ModuleGraph>,
        module: Rc<Module>,
    }
}

impl MiniKnob {
    fn new(
        parent: &impl MiniKnobParent,
        pos: Vec2D,
        control: Rcrc<FloatInRangeControl>,
        label: String,
        tooltip: String,
    ) -> Rc<Self> {
        let state = MiniKnobState {
            tooltip,
            control,
            value: rcrc(0.0),
            pos,
            label,
        };
        Rc::new(Self::create(parent, state))
    }
}

impl WidgetImpl<Renderer, DropTarget> for MiniKnob {
    fn get_pos_impl(self: &Rc<Self>) -> Vec2D {
        self.state.borrow().pos
    }

    fn get_size_impl(self: &Rc<Self>) -> Vec2D {
        grid(1).into()
    }

    fn get_mouse_behavior_impl(
        self: &Rc<Self>,
        _pos: Vec2D,
        mods: &MouseMods,
    ) -> MaybeMouseBehavior {
        let state = self.state.borrow();
        if mods.right_click {
            let parent_pos = self.parents.module.get_pos();
            let pos = state.pos + parent_pos + grid(1) / 2.0;
            let graph = Rc::clone(&self.parents.graph);
            let menu = KnobEditor::new(
                self,
                Rc::clone(&state.control),
                Rc::clone(&state.value),
                pos,
                state.label.clone(),
                state.tooltip.clone(),
            );
            OnClickBehavior::wrap(move || {
                graph.open_menu(Box::new(menu));
            })
        } else {
            Some(Box::new(ManipulateControl::new(
                self,
                Rc::clone(&state.control),
            )))
        }
    }

    fn get_drop_target_impl(self: &Rc<Self>, _pos: Vec2D) -> Option<DropTarget> {
        Some(DropTarget::Autocon(Rc::clone(&self.state.borrow().control)))
    }

    fn on_hover_impl(self: &Rc<Self>, _pos: Vec2D) -> Option<()> {
        let tooltip = Tooltip {
            text: self.state.borrow().tooltip.clone(),
            interaction: InteractionHint::LeftClickAndDrag
                | InteractionHint::RightClick
                | InteractionHint::DoubleClick,
        };
        self.with_gui_state_mut(|state| {
            state.set_tooltip(tooltip);
        });
        Some(())
    }

    fn draw_impl(self: &Rc<Self>, g: &mut Renderer) {
        let state = self.state.borrow();
        let highlight = unimplemented!();
        let feedback_data: &[f32] = unimplemented!();
        const MIN_ANGLE: f32 = PI * 1.10;
        const MAX_ANGLE: f32 = -PI * 0.10;

        let control = &*state.control.borrow();
        fn value_to_angle(range: (f32, f32), value: f32) -> f32 {
            value.from_range_to_range(range.0, range.1, MIN_ANGLE, MAX_ANGLE)
        }

        g.set_color(&COLOR_FG1);

        if highlight {
            g.set_color(&COLOR_FG1);
        } else {
            g.set_color(&COLOR_BG0);
        }
        g.draw_pie(0, grid(1), KNOB_INSIDE_SPACE * 2.0, MIN_ANGLE, MAX_ANGLE);
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
        *state.value.borrow_mut() = value;
        let value_angle = value_to_angle(control.range, value);
        g.draw_pie(
            0,
            grid(1),
            KNOB_INSIDE_SPACE * 2.0,
            zero_angle.clam(MAX_ANGLE, MIN_ANGLE),
            value_angle,
        );
        g.set_alpha(1.0);
        g.set_color(&COLOR_FG1);
        g.draw_text(FONT_SIZE, 0, grid(1), (0, 1), 1, &state.label);

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
                g.draw_pie(inset, outer_diameter, inner_diameter, min_angle, max_angle);
            }
        }
    }
}

impl ModuleWidgetImpl for MiniKnob {
    fn add_wires(self: &Rc<Self>, wire_tracker: &mut WireTracker) {
        let state = self.state.borrow();
        let center = state.pos + grid(1) / 2.0;
        for lane in state.control.borrow().automation.iter() {
            unimplemented!();
        }
    }
}
