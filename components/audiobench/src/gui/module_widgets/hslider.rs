use crate::{
    engine::controls::{Control, FloatInRangeControl},
    gui::{
        constants::*,
        module_widgets::ModuleWidgetImpl,
        mouse_behaviors::{ManipulateControl, ManipulateLane},
        top_level::graph::{Module, ModuleGraph},
        InteractionHint, Tooltip,
    },
    scui_config::{DropTarget, MaybeMouseBehavior, Renderer},
};
use scui::{MouseMods, OnClickBehavior, Vec2D, Widget, WidgetImpl};
use shared_util::prelude::*;

yaml_widget_boilerplate::make_widget_outline! {
    widget_struct: HSlider,
    constructor: new(
        parent: ParentRef,
        pos: GridPos,
        size: GridSize,
        control: FloatInRangeControlRef,
        label: String,
        tooltip: String,
    ),
    feedback: ControlSignal,
}

scui::widget! {
    pub HSlider
    State {
        pos: Vec2D,
        size: Vec2D,
        control: Rcrc<FloatInRangeControl>,
        // This allows the slider to share feedback data with the right-click menu when it it open.
        value: Rcrc<f32>,
        label: String,
        tooltip: String,
    }
    Parents {
        graph: Rc<ModuleGraph>,
        module: Rc<Module>,
    }
}

const HEIGHT: f32 = grid(1);

impl HSlider {
    fn new(
        parent: &impl HSliderParent,
        pos: Vec2D,
        size: Vec2D,
        control: Rcrc<FloatInRangeControl>,
        label: String,
        tooltip: String,
    ) -> Rc<Self> {
        let state = HSliderState {
            pos,
            size: size * (1, 0) + (0.0, HEIGHT),
            control,
            value: rcrc(0.0),
            label,
            tooltip,
        };
        Rc::new(Self::create(parent, state))
    }
}

impl WidgetImpl<Renderer, DropTarget> for HSlider {
    fn get_pos_impl(self: &Rc<Self>) -> Vec2D {
        self.state.borrow().pos
    }

    fn get_size_impl(self: &Rc<Self>) -> Vec2D {
        self.state.borrow().size
    }

    fn get_mouse_behavior_impl(
        self: &Rc<Self>,
        _pos: Vec2D,
        mods: &MouseMods,
    ) -> MaybeMouseBehavior {
        let state = self.state.borrow();

        if mods.right_click {
            let parent_pos = self.parents.module.get_pos();
            let pos = state.pos + parent_pos + grid(2) / 2.0;
            let graph = Rc::clone(&self.parents.graph);
            let menu = HSliderEditor::new(
                self,
                Rc::clone(&state.control),
                Rc::clone(&state.value),
                pos,
                state.size.x,
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
        Some(DropTarget::Control(
            Rc::clone(&self.state.borrow().control) as _
        ))
    }

    fn on_hover_impl(self: &Rc<Self>, _pos: Vec2D) -> Option<()> {
        let tooltip = Tooltip {
            text: self.state.borrow().tooltip.clone(),
            interaction: vec![
                InteractionHint::LeftClickAndDrag,
                InteractionHint::RightClick,
                InteractionHint::DoubleClick,
            ],
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
        const CS: f32 = CORNER_SIZE;

        let control = &*state.control.borrow();
        fn value_to_point(range: (f32, f32), width: f32, value: f32) -> f32 {
            value.from_range_to_range(range.0, range.1, 0.0, width)
        }

        g.set_color(&COLOR_FG1);

        if highlight {
            g.set_color(&COLOR_FG1);
        } else {
            g.set_color(&COLOR_BG0);
        }
        g.draw_rounded_rect(0, state.size, CS);
        g.set_color(&COLOR_EDITABLE);
        if highlight {
            g.set_alpha(0.5);
        }
        let zero_point = value_to_point(control.range, state.size.x, 0.0);
        // If manual, show the manual value. If automated, show the most recent value recorded
        // from when a note was actually playing.
        let value = if control.automation.len() > 0 {
            feedback_data[0]
        } else {
            control.value
        };
        *state.value.borrow_mut() = value;
        let value_point = value_to_point(control.range, state.size.x, value);
        g.draw_rounded_rect(
            (zero_point.min(value_point), 0.0),
            ((zero_point - value_point).abs(), state.size.y),
            CS,
        );
        g.set_alpha(1.0);
        g.set_color(&COLOR_FG1);

        if control.automation.len() > 0 {
            let num_lanes = control.automation.len() as f32;
            let lane_size = (grid(1) - CS * 2.0) / num_lanes;
            let lane_size = lane_size.min(KNOB_MAX_LANE_SIZE).max(2.0);
            for (index, lane) in control.automation.iter().enumerate() {
                g.set_color(&COLOR_AUTOMATION);
                let index = index as f32;
                let start_point = value_to_point(control.range, state.size.x, lane.range.0);
                let end_point = value_to_point(control.range, state.size.x, lane.range.1);
                g.draw_rect(
                    // -0.5 makes it crisper at default zoom.
                    (start_point.min(end_point), CS + (lane_size) * index - 0.5),
                    ((start_point - end_point).abs(), lane_size - KNOB_LANE_GAP),
                );
            }
        }

        g.pop_state();
    }
}

impl ModuleWidgetImpl for HSlider {
    fn represented_control(self: &Rc<Self>) -> Option<Rcrc<dyn Control>> {
        Some(Rc::clone(&self.state.borrow().control) as _)
    }
}

scui::widget! {
    pub HSliderEditor
    State {
        control: Rcrc<FloatInRangeControl>,
        value: Rcrc<f32>,
        pos: Vec2D,
        size: Vec2D,
        label: String,
        tooltip: String,
    }
}

impl HSliderEditor {
    fn new(
        parent: &impl HSliderEditorParent,
        control: Rcrc<FloatInRangeControl>,
        value: Rcrc<f32>,
        // Bottom left position.
        center_pos: Vec2D,
        width: f32,
        label: String,
        tooltip: String,
    ) -> Rc<Self> {
        let num_channels = control.borrow().automation.len().max(0) as f32;
        let required_height = grid(1)
            + KNOB_MENU_LANE_GAP
            + (KNOB_MENU_LANE_SIZE + KNOB_MENU_LANE_GAP) * num_channels;
        let size = Vec2D::new(
            width + grid(4) + GRID_P * 2.0,
            required_height + GRID_P * 2.0,
        );
        let state = HSliderEditorState {
            control,
            value,
            pos: center_pos - (grid(2) - GRID_P, size.y + GRID_P),
            size,
            label,
            tooltip,
        };
        Rc::new(Self::create(parent, state))
    }

    fn value_to_point(&self, value: f32) -> f32 {
        let state = self.state.borrow();
        let range = state.control.borrow().range;
        const GP: f32 = GRID_P;
        value.from_range_to_range(range.0, range.1, 0.0, state.size.x - (GP + grid(1)) * 2.0)
            + GP
            + grid(1)
    }
}

impl WidgetImpl<Renderer, DropTarget> for HSliderEditor {
    fn get_pos_impl(self: &Rc<Self>) -> Vec2D {
        self.state.borrow().pos
    }

    fn get_size_impl(self: &Rc<Self>) -> Vec2D {
        self.state.borrow().size
    }

    fn get_mouse_behavior_impl(
        self: &Rc<Self>,
        pos: Vec2D,
        mods: &MouseMods,
    ) -> MaybeMouseBehavior {
        let state = self.state.borrow();
        const GP: f32 = GRID_P;
        const GAP: f32 = KNOB_MENU_LANE_GAP;
        const LS: f32 = KNOB_MENU_LANE_SIZE;
        let row = ((pos.y - GP) / (GAP + LS)).max(0.0) as usize;
        let control_ref = state.control.borrow();
        if row >= control_ref.automation.len() {
            // Clicked the actual control...
            return Some(Box::new(ManipulateControl::new(
                self,
                Rc::clone(&state.control),
            )));
        }
        // Lanes are rendered backwards, flip it back around.
        let lane = control_ref.automation.len() - row - 1;
        let point = pos.x;
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
            if mods.right_click {
                let control = Rc::clone(&state.control);
                let engine = self.with_gui_state(|state| Rc::clone(&state.engine));
                return OnClickBehavior::wrap(move || {
                    control.borrow_mut().automation.remove(lane);
                    engine.borrow_mut().regenerate_code()
                });
            } else {
                return Some(Box::new(ManipulateLane::new(
                    self,
                    Rc::clone(&state.control),
                    lane,
                )));
            };
        }
        // xor
        return Some(Box::new(if (point < min_point) != ends_flipped {
            ManipulateLane::start_only(self, Rc::clone(&state.control), lane)
        } else {
            ManipulateLane::end_only(self, Rc::clone(&state.control), lane)
        }));
    }

    fn on_hover_impl(self: &Rc<Self>, pos: Vec2D) -> Option<()> {
        let state = self.state.borrow();
        const GP: f32 = GRID_P;
        const GAP: f32 = KNOB_MENU_LANE_GAP;
        const LS: f32 = KNOB_MENU_LANE_SIZE;
        let row = ((pos.y - GP) / (GAP + LS)).max(0.0) as usize;
        let control_ref = state.control.borrow();
        if row >= control_ref.automation.len() {
            // Inside the actual control
            let tooltip = Tooltip {
                text: state.tooltip.clone(),
                interaction: vec![
                    InteractionHint::LeftClickAndDrag,
                    InteractionHint::DoubleClick,
                ],
            };
            self.with_gui_state_mut(|state| {
                state.set_tooltip(tooltip);
            });
            return Some(());
        }
        // Lanes are rendered backwards, flip it back around.
        let lane = control_ref.automation.len() - row - 1;
        let point = pos.x;
        let tooltip = if point < GP + grid(1) || point > state.size.x - GP - grid(1) {
            // Clicked outside the lane.
            Tooltip {
                text: format!(
                    "Automation lane #{}, click + drag to move one of the ends.",
                    lane + 1,
                ),
                interaction: vec![
                    InteractionHint::LeftClickAndDrag,
                    InteractionHint::DoubleClick,
                ],
            }
        } else {
            Tooltip {
                text: format!(
                    "Automation lane #{}, click + drag on empty space to move one end at a time.",
                    lane + 1,
                ),
                interaction: vec![
                    InteractionHint::LeftClickAndDrag,
                    InteractionHint::DoubleClick,
                ],
            }
        };
        self.with_gui_state_mut(|state| {
            state.set_tooltip(tooltip);
        });
        Some(())
    }

    fn draw_impl(self: &Rc<Self>, g: &mut Renderer) {
        let state = self.state.borrow();
        const BSR: f32 = POPUP_SHADOW_RADIUS;
        const CS: f32 = CORNER_SIZE;
        const GP: f32 = GRID_P;
        g.draw_inset_box_shadow(0, state.size, BSR, CS);
        g.set_color(&COLOR_BG2);
        g.draw_rounded_rect(0, state.size, CS);

        let control = &*state.control.borrow();

        g.set_color(&COLOR_BG0);
        let x = GP + grid(1);
        let width = state.size.x - (GP + grid(1)) * 2.0;
        let boty = state.size.y - GP - grid(1);
        g.draw_rounded_rect((x, boty), (width, grid(1)), CS);
        g.set_color(&COLOR_EDITABLE);
        let value = *state.value.borrow();
        let zero_point = self.value_to_point(0.0);
        let value_point = self.value_to_point(value);
        g.draw_rounded_rect(
            (zero_point.min(value_point), boty),
            ((zero_point - value_point).abs(), grid(1)),
            CS,
        );

        const GAP: f32 = KNOB_MENU_LANE_GAP;
        const LS: f32 = KNOB_MENU_LANE_SIZE;
        for (index, lane) in control.automation.iter().rev().enumerate() {
            g.set_color(&COLOR_BG0);
            let y = GP + (LS + GAP) * index as f32;
            g.draw_rounded_rect((x, y), (width, LS), CS);
            g.set_color(&COLOR_AUTOMATION);
            let min_point = self.value_to_point(lane.range.0);
            let max_point = self.value_to_point(lane.range.1);
            let height = if lane.range.0 > lane.range.1 {
                LS / 2.0
            } else {
                LS
            };
            g.draw_rounded_rect(
                (min_point.min(max_point), y),
                ((min_point - max_point).abs(), height),
                CS,
            );
        }

        g.set_color(&COLOR_FG1);
        g.draw_text(
            FONT_SIZE,
            (x + GP, boty),
            (width, grid(1)),
            (-1, 0),
            1,
            &state.label,
        );
        let value_text = format!("{}{}", format_decimal(value, 3), control.suffix);
        g.draw_text(
            FONT_SIZE,
            (x + GP, boty),
            (width - GP * 2.0, grid(1)),
            (1, 0),
            1,
            &value_text,
        );
    }
}
