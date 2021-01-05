use super::ModuleWidgetImpl;
use crate::engine::controls::{FrequencyControl, UpdateRequest};
use crate::gui::mouse_behaviors::{ContinuouslyMutateStaticon};
use crate::gui::constants::*;
use crate::gui::graphics::{GrahpicsWrapper, HAlign, VAlign};
use crate::gui::{InteractionHint, Tooltip};
use crate::scui_config::{DropTarget, Renderer, MaybeMouseBehavior};
use scui::{MouseMods, Vec2D, WidgetImpl};
use shared_util::prelude::*;

yaml_widget_boilerplate::make_widget_outline! {
    widget_struct: FrequencyBox,
    constructor: new(
        parent: ParentRef,
        pos: GridPos,
        control: FrequencyControlRef,
        label: String,
        tooltip: String,
    ),
}

scui::widget! {
    pub FrequencyBox
    State {
        pos: Vec2D,
        control: Rcrc<FrequencyControl>,
        label: String,
        tooltip: String,
    }
}

impl FrequencyBox {
    const WIDTH: f32 = grid(2);
    const HEIGHT: f32 = grid(2) - FONT_SIZE - GRID_P / 2.0;
    fn new(
        parent: &impl FrequencyBoxParent,
        pos: Vec2D,
        control: Rcrc<FrequencyControl>,
        label: String,
        tooltip: String,
    ) -> Rc<Self> {
        let state = FrequencyBoxState {
            pos,
            control,
            label,
            tooltip,
        };
        Rc::new(Self::create(parent, state))
    }
}

impl WidgetImpl<Renderer, DropTarget> for FrequencyBox {
    fn get_pos_impl(self: &Rc<Self>) -> Vec2D {
        self.state.borrow().pos
    }

    fn get_size_impl(self: &Rc<Self>) -> Vec2D {
        grid(2).into()
    }

    fn get_mouse_behavior_impl(self: &Rc<Self>, _pos: Vec2D, _mods: &MouseMods) -> MaybeMouseBehavior {
        let state = self.state.borrow();
        let frequency = state.control.borrow();
        let cref = Rc::clone(&state.control);
        let mut float_value = frequency.get_value();
        ContinuouslyMutateStaticon::wrap(self, move |delta, _steps| {
            float_value *= (2.0f32).powf(delta / LOG_OCTAVE_PIXELS);
            float_value = float_value.clam(FrequencyControl::MIN_FREQUENCY, 99_000.0);
            let update = cref.borrow_mut().set_value(float_value);
            (update, None)
        })
    }

    fn on_hover_impl(self: &Rc<Self>, _pos: Vec2D) -> Option<()> {
        let this_state = self.state.borrow();
        self.with_gui_state_mut(|state| {
            state.set_tooltip(Tooltip {
                text: this_state.tooltip.clone(),
                interaction: InteractionHint::LeftClickAndDrag | InteractionHint::RightClick,
            });
        });
        Some(())
    }

    fn draw_impl(self: &Rc<Self>, g: &mut Renderer) {
        let state = self.state.borrow();
        const W: f32 = FrequencyBox::WIDTH;
        const H: f32 = FrequencyBox::HEIGHT;
        const CS: f32 = CORNER_SIZE;
        g.set_color(&COLOR_BG0);
        g.draw_rounded_rect(0, (W, H), CS);
        {
            let val = state.control.borrow().get_formatted_value();
            g.set_color(&COLOR_FG1);
            g.draw_text(
                BIG_FONT_SIZE,
                (GRID_P, 0.0),
                (W - GRID_P * 2.0, H),
                (-1, 0),
                1,
                &val,
            );
        }
        {
            let val = &state.label;
            g.set_color(&COLOR_FG1);
            g.draw_text(FONT_SIZE, 0, (W, grid(2)), (0, 1), 1, val);
        }
    }
}

impl ModuleWidgetImpl for FrequencyBox {}
