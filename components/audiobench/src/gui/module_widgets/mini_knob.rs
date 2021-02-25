use crate::{
    engine::controls::{Control, FloatInRangeControl},
    gui::{
        constants::*,
        module_widgets::{KnobEditor, ModuleWidgetImpl},
        mouse_behaviors::ManipulateControl,
        top_level::graph::{Module, ModuleGraph},
        InteractionHint, Tooltip,
    },
    scui_config::{DropTarget, MaybeMouseBehavior, Renderer},
};
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
    feedback: ControlSignal,
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
        const MIN_ANGLE: f32 = PI * 1.40;
        const MAX_ANGLE: f32 = -PI * 0.40;

        fn value_to_angle(range: (f32, f32), value: f32) -> f32 {
            value.from_range_to_range(range.0, range.1, MIN_ANGLE, MAX_ANGLE)
        }

        g.set_color(&COLOR_FG1);

        if highlight {
            g.set_color(&COLOR_FG1);
        } else {
            g.set_color(&COLOR_BG0);
        }
        g.draw_pie(0, grid(1), KNOB_INSIDE_SPACE, MIN_ANGLE, MAX_ANGLE);
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
            grid(1),
            KNOB_INSIDE_SPACE,
            zero_angle.clam(MAX_ANGLE, MIN_ANGLE),
            value_angle,
        );
        g.set_alpha(1.0);
    }
}

impl ModuleWidgetImpl for MiniKnob {
    fn represented_control(self: &Rc<Self>) -> Option<Rcrc<dyn Control>> {
        Some(Rc::clone(&self.state.borrow().control) as _)
    }

    fn take_feedback_data(self: &Rc<Self>, data: Vec<f32>) {
        assert!(data.len() == 1);
        *self.state.borrow_mut().value.borrow_mut() = data[0];
    }
}
