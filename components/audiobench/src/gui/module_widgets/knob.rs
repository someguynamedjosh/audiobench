use crate::{
    engine::controls::{Control, FloatInRangeControl},
    gui::{
        constants::*,
        module_widgets::ModuleWidgetImpl,
        mouse_behaviors::{ManipulateFIRControl, ManipulateLane},
        top_level::graph::{Module, ModuleGraph},
        InteractionHint, Tooltip,
    },
    scui_config::{DropTarget, MaybeMouseBehavior, Renderer},
};
use scui::{MouseMods, OnClickBehavior, Vec2D, Widget, WidgetImpl};
use shared_util::prelude::*;
use std::f32::consts::PI;

yaml_widget_boilerplate::make_widget_outline! {
    widget_struct: Knob,
    constructor: new(
        parent: ParentRef,
        pos: GridPos,
        control: FloatInRangeControlRef,
        label: String,
        tooltip: String,
    ),
    feedback: ControlSignal,
}

scui::widget! {
    pub Knob
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

impl Knob {
    fn new(
        parent: &impl KnobParent,
        pos: Vec2D,
        control: Rcrc<FloatInRangeControl>,
        label: String,
        tooltip: String,
    ) -> Rc<Self> {
        let state = KnobState {
            tooltip,
            control,
            value: rcrc(0.0),
            pos,
            label,
        };
        Rc::new(Self::create(parent, state))
    }
}

impl WidgetImpl<Renderer, DropTarget> for Knob {
    fn get_pos_impl(self: &Rc<Self>) -> Vec2D {
        self.state.borrow().pos
    }

    fn get_size_impl(self: &Rc<Self>) -> Vec2D {
        grid(2).into()
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
            Some(Box::new(ManipulateFIRControl::new(
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
        let hmode = self.parents.graph.get_highlight_mode();
        let highlight = hmode.should_highlight(&state.control);
        let dim = hmode.should_dim(&state.control);
        let control = state.control.borrow();
        const MIN_ANGLE: f32 = PI * 1.10;
        const MAX_ANGLE: f32 = -PI * 0.10;

        fn value_to_angle(range: (f32, f32), value: f32) -> f32 {
            value.from_range_to_range(range.0, range.1, MIN_ANGLE, MAX_ANGLE)
        }

        g.set_color(&COLOR_FG1);

        if highlight {
            g.set_color(&COLOR_FG1);
        } else {
            g.set_color(&COLOR_BG0);
        }
        g.draw_pie(0, grid(2), KNOB_INSIDE_SPACE * 2.0, MIN_ANGLE, MAX_ANGLE);

        let zero_angle = value_to_angle(control.range, 0.0);
        // If manual, show the manual value. If automated, show the most recent value recorded
        // from when a note was actually playing.
        let value = if control.automation.len() > 0 {
            *state.value.borrow()
        } else {
            control.value
        };
        *state.value.borrow_mut() = value;
        let value_angle = value_to_angle(control.range, value);

        g.set_color(&COLOR_EDITABLE);
        if highlight || dim {
            g.set_alpha(0.5);
        }
        g.draw_pie(
            0,
            grid(2),
            KNOB_INSIDE_SPACE * 2.0,
            zero_angle.clam(MAX_ANGLE, MIN_ANGLE),
            value_angle,
        );
        g.set_alpha(1.0);
        g.set_color(&COLOR_FG1);
        g.draw_text(FONT_SIZE, 0, grid(2), (0, 1), 1, &state.label);

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
                g.draw_pie(inset, outer_diameter, inner_diameter, min_angle, max_angle);
            }
        }
    }
}

impl ModuleWidgetImpl for Knob {
    fn represented_control(self: &Rc<Self>) -> Option<Rcrc<dyn Control>> {
        Some(Rc::clone(&self.state.borrow().control) as _)
    }

    fn take_feedback_data(self: &Rc<Self>, data: Vec<f32>) {
        assert!(data.len() == 1);
        *self.state.borrow_mut().value.borrow_mut() = data[0];
    }
}

scui::widget! {
    pub(super) KnobEditor
    State {
        control: Rcrc<FloatInRangeControl>,
        value: Rcrc<f32>,
        pos: Vec2D,
        size: Vec2D,
        label: String,
        tooltip: String,
    }
}

impl KnobEditor {
    pub(super) fn new(
        parent: &impl KnobEditorParent,
        control: Rcrc<FloatInRangeControl>,
        value: Rcrc<f32>,
        center_pos: Vec2D,
        label: String,
        tooltip: String,
    ) -> Rc<Self> {
        let num_channels = control.borrow().automation.len().max(2) as f32;
        let required_radius =
            (KNOB_MENU_LANE_SIZE + KNOB_MENU_LANE_GAP) * num_channels + KNOB_MENU_KNOB_OR + GRID_P;
        let size = (required_radius * 2.0, required_radius + fatgrid(1)).into();
        let state = KnobEditorState {
            control,
            value,
            pos: (center_pos - size / 2.0),
            size,
            label,
            tooltip,
        };
        Rc::new(Self::create(parent, state))
    }
}

impl WidgetImpl<Renderer, DropTarget> for KnobEditor {
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
        // Yes, the x is intentional. The center of the knob is not vertically centered.
        // y coordinate is inverted from how it appears on screen.
        let centered = (pos - state.size.x / 2.0) * (1, -1);
        let (angle, radius) = (centered.angle(), centered.length());
        let control = &*state.control.borrow();
        let auto_lanes = control.automation.len();
        // Clicked somewhere in the top "half" where the main knob and automation lanes are.
        if angle >= 0.0 && angle <= PI {
            let radius = radius as f32;
            if radius < KNOB_MENU_KNOB_IR {
                // Nothing interactable inside the knob.
            } else if radius < KNOB_MENU_KNOB_OR {
                return Some(Box::new(ManipulateFIRControl::new(
                    self,
                    Rc::clone(&state.control),
                )));
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
                    return Some(Box::new(if (angle > min_angle) != ends_flipped {
                        ManipulateLane::start_only(self, Rc::clone(&state.control), lane)
                    } else {
                        ManipulateLane::end_only(self, Rc::clone(&state.control), lane)
                    }));
                }
            }
        } else {
            // If we clicked under one of the automation lanes...
            if -centered.y > 0.0 && centered.x.abs() > KNOB_MENU_KNOB_OR {
                let lane = ((centered.x.abs() - KNOB_MENU_KNOB_OR)
                    / (KNOB_MENU_LANE_SIZE + KNOB_MENU_LANE_GAP))
                    as usize;
                if lane >= auto_lanes {
                    return None;
                }
                // Lanes are shown in reverse order.
                let lane = auto_lanes - lane - 1;
                let lane_range = control.automation[lane].range;
                let ends_flipped = lane_range.0 > lane_range.1;
                // xor
                return Some(Box::new(if (centered.x > 0.0) != ends_flipped {
                    ManipulateLane::end_only(self, Rc::clone(&state.control), lane)
                } else {
                    ManipulateLane::start_only(self, Rc::clone(&state.control), lane)
                }));
            }
        }
        None
    }

    fn on_hover_impl(self: &Rc<Self>, pos: Vec2D) -> Option<()> {
        let state = self.state.borrow();
        // Yes, the x is intentional. The center of the knob is not vertically centered.
        // y coordinate is inverted from how it appears on screen.
        let centered = (pos - state.size.x / 2.0) * (1, -1);
        let (angle, radius) = (centered.angle(), centered.length());
        let control = &*state.control.borrow();
        let auto_lanes = control.automation.len();
        // Clicked somewhere in the top "half" where the main knob and automation lanes are.
        if !(angle >= 0.0 && angle <= PI) {
            // If we clicked under one of the automation lanes...
            if -centered.y > 0.0 && centered.x.abs() > KNOB_MENU_KNOB_OR {
                let lane = ((centered.x.abs() - KNOB_MENU_KNOB_OR)
                    / (KNOB_MENU_LANE_SIZE + KNOB_MENU_LANE_GAP))
                    as usize;
                if lane < auto_lanes {
                    let tooltip = Tooltip {
                        text: format!(
                            "Automation lane #{}, click + drag to move one of the ends.",
                            lane + 1,
                        ),
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
            }
            return None;
        }
        let radius = radius as f32;
        if radius < KNOB_MENU_KNOB_IR {
            return None;
            // Nothing interactable inside the knob.
        }
        let tooltip = if radius < KNOB_MENU_KNOB_OR {
            Tooltip {
                text: state.tooltip.clone(),
                interaction: vec![
                    InteractionHint::LeftClickAndDrag,
                    InteractionHint::DoubleClick,
                ],
            }
        } else {
            let radius = radius - KNOB_MENU_KNOB_OR;
            let lane = (radius / (KNOB_MENU_LANE_SIZE + KNOB_MENU_LANE_GAP)) as usize;
            if lane < auto_lanes {
                Tooltip {
                    text: format!(
                    "Automation lane #{}, click + drag on empty space to move one end at a time.",
                    lane + 1,
                ),
                    interaction: vec![
                        InteractionHint::LeftClickAndDrag,
                        InteractionHint::DoubleClick,
                        InteractionHint::RightClick,
                    ],
                }
            } else {
                return None;
            }
        };
        self.with_gui_state_mut(|state| {
            state.set_tooltip(tooltip);
        });
        None
    }

    fn draw_impl(self: &Rc<Self>, g: &mut Renderer) {
        let state = self.state.borrow();
        let control = state.control.borrow();
        const BSR: f32 = POPUP_SHADOW_RADIUS;
        const CS: f32 = CORNER_SIZE;

        g.draw_inset_box_shadow(0, state.size, BSR, CS);
        g.set_color(&COLOR_BG2);
        g.draw_rounded_rect(0, state.size, CS);

        fn value_to_angle(range: (f32, f32), value: f32) -> f32 {
            value.from_range_to_range(range.0, range.1, PI, 0.0)
        }
        g.translate((state.size.x / 2.0, state.size.y - fatgrid(1)));

        const KOR: f32 = KNOB_MENU_KNOB_OR;
        const KIR: f32 = KNOB_MENU_KNOB_IR;
        g.set_color(&COLOR_BG0);
        g.draw_pie(-KOR, KOR * 2.0, KIR * 2.0, PI, 0.0);
        g.set_color(&COLOR_EDITABLE);
        let zero_angle = value_to_angle(control.range, 0.0);
        let value = *state.value.borrow();
        let value_angle = value_to_angle(control.range, value);
        g.draw_pie(
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
            g.set_color(&COLOR_BG0);
            g.draw_pie(-or, or * 2.0, ir * 2.0, PI, 0.0);
            g.set_color(&COLOR_AUTOMATION);
            let min_angle = value_to_angle(control.range, lane.range.0);
            let max_angle = value_to_angle(control.range, lane.range.1);
            let ir = if lane.range.0 > lane.range.1 {
                ir + LS / 2.0
            } else {
                ir
            };
            g.draw_pie(-or, or * 2.0, ir * 2.0, min_angle, max_angle);
        }

        g.set_color(&COLOR_FG1);
        let value_text = format!("{}{}", format_decimal(value, 3), control.suffix);
        g.draw_label((-KIR, -12.0), KIR * 2.0, &value_text);
        g.draw_label((-KOR, GRID_P), KOR * 2.0, &state.label);
    }
}
