use super::ModuleWidgetImpl;
use crate::engine::controls::{OptionChoiceControl, UpdateRequest};
use crate::gui::constants::*;
use crate::gui::mouse_behaviors::MutateStaticon;
use crate::gui::{InteractionHint, Tooltip};
use crate::scui_config::{DropTarget, MaybeMouseBehavior, Renderer};
use scui::{MouseMods, Vec2D, WidgetImpl};
use shared_util::prelude::*;

yaml_widget_boilerplate::make_widget_outline! {
    widget_struct: OptionBox,
    constructor: new(
        parent: ParentRef,
        pos: GridPos,
        size: GridSize,
        control: OptionChoiceControlRef,
        label: String,
        tooltip: String,
    ),
}

scui::widget! {
    pub OptionBox
    State {
        pos: Vec2D,
        size: Vec2D,
        control: Rcrc<OptionChoiceControl>,
        label: String,
        tooltip: String,
    }
}

impl OptionBox {
    fn new(
        parent: &impl OptionBoxParent,
        pos: Vec2D,
        size: Vec2D,
        control: Rcrc<OptionChoiceControl>,
        label: String,
        tooltip: String,
    ) -> Rc<Self> {
        let state = OptionBoxState {
            tooltip,
            control,
            pos,
            size,
            label,
        };
        Rc::new(Self::create(parent, state))
    }
}

impl WidgetImpl<Renderer, DropTarget> for OptionBox {
    fn get_pos_impl(self: &Rc<Self>) -> Vec2D {
        self.state.borrow().pos
    }

    fn get_size_impl(self: &Rc<Self>) -> Vec2D {
        self.state.borrow().size
    }

    fn get_mouse_behavior_impl(
        self: &Rc<Self>,
        pos: Vec2D,
        _mods: &MouseMods,
    ) -> MaybeMouseBehavior {
        let state = self.state.borrow();
        let num_options = state.control.borrow().get_options().len();
        let height_per_option = (state.size.y - FONT_SIZE - GRID_P / 2.0) / num_options as f32;
        let option = (pos.y / height_per_option) as usize;
        if option < num_options {
            let cref = Rc::clone(&state.control);
            MutateStaticon::wrap(self, move || cref.borrow_mut().set_selected_option(option))
        } else {
            None
        }
    }

    fn on_hover_impl(self: &Rc<Self>, _pos: Vec2D) -> Option<()> {
        let tooltip = Tooltip {
            text: self.state.borrow().tooltip.clone(),
            interaction: InteractionHint::LeftClick | InteractionHint::DoubleClick,
        };
        self.with_gui_state_mut(|state| {
            state.set_tooltip(tooltip);
        });
        Some(())
    }

    fn draw_impl(self: &Rc<Self>, g: &mut Renderer) {
        let state = self.state.borrow();
        const CS: f32 = CORNER_SIZE;
        g.set_color(&COLOR_BG0);
        let num_options = state.control.borrow().get_options().len();
        // Don't ask why GP / 2 and not just GP, it just looks better and I don't know why.
        let height_per_option = (state.size.y - FONT_SIZE - GRID_P / 2.0) / num_options as f32;
        let h = height_per_option * num_options as f32;
        g.draw_rounded_rect(0, (state.size.x, h), CS);
        let current_option = state.control.borrow().get_selected_option();
        for (index, option) in state.control.borrow().get_options().iter().enumerate() {
            let y = index as f32 * height_per_option;
            if index == current_option {
                g.set_color(&COLOR_BG1);
                g.draw_rounded_rect((0.0, y), (state.size.x, height_per_option), CORNER_SIZE);
            }
            g.set_color(&COLOR_FG1);
            g.draw_text(
                FONT_SIZE,
                (0.0, y),
                (state.size.x, height_per_option),
                (0, 0),
                1,
                option,
            );
        }
        g.draw_text(FONT_SIZE, 0.0, state.size, (0, 1), 1, &state.label);
    }
}

impl ModuleWidgetImpl for OptionBox {}
