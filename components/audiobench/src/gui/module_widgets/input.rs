use super::ModuleWidgetImpl;
use crate::engine::controls::{InputControl, UpdateRequest};
use crate::gui::constants::*;
use crate::gui::graphics::{GrahpicsWrapper, HAlign, VAlign};
use crate::gui::mouse_behaviors::{ContinuouslyMutateStaticon, MutateStaticon};
use crate::gui::top_level::graph::Module;
use crate::gui::{InteractionHint, Tooltip};
use crate::scui_config::{DropTarget, MaybeMouseBehavior, Renderer};
use scui::{MouseBehavior, MouseMods, Vec2D, WidgetImpl};
use shared_util::prelude::*;
use std::f32::consts::TAU;

yaml_widget_boilerplate::make_widget_outline! {
    widget_struct: Input,
    constructor: new(
        parent: ParentRef,
        y: i32,
        control: InputControlRef,
        label: String,
        tooltip: String,
        icon: OptionalString,
    ),
}

scui::widget! {
    pub Input
    State {
        pos: Vec2D,
        control: Rcrc<InputControl>,
        tooltip: String,
        label: String,
        icon: usize,
        small_icon: Option<usize>,
    }
    Parents {
        module: Rc<Module>,
    }
}

const WIDTH: f32 = JACK_SIZE + JACK_SIZE;
const HEIGHT: f32 = grid(1);

impl Input {
    pub fn new(
        parent: &impl InputParent,
        y: i32,
        control: Rcrc<InputControl>,
        label: String,
        tooltip: String,
        optional_icon: Option<String>,
    ) -> Rc<Self> {
        let pos = Vec2D::new(0.0, coord(y));
        let int = parent.provide_gui_interface();
        let gui_state = int.state.borrow();
        let registry = gui_state.registry.borrow();
        let mut icon = registry
            .lookup_icon(control.borrow().get_type().icon_name())
            .unwrap();
        let mut small_icon = None;
        if let Some(name) = optional_icon {
            // TODO: Better error unimplemented!()
            let index = registry.lookup_icon(&name).unwrap();
            small_icon = Some(icon);
            icon = index;
        }
        let state = InputState {
            pos,
            control,
            label,
            tooltip,
            icon,
            small_icon,
        };
        Rc::new(Self::create(parent, state))
    }
}

struct InputBehavior(Rcrc<InputControl>);

impl MouseBehavior<DropTarget> for InputBehavior {
    fn on_click(self: Box<Self>) {
        let mut control = self.0.borrow_mut();
        if control.get_used_default().is_some() {
            control.next_default();
        }
    }
}

impl WidgetImpl<Renderer, DropTarget> for Input {
    fn get_pos_impl(self: &Rc<Self>) -> Vec2D {
        self.state.borrow().pos
    }

    fn get_size_impl(self: &Rc<Self>) -> Vec2D {
        (WIDTH, HEIGHT).into()
    }

    fn get_mouse_behavior_impl(
        self: &Rc<Self>,
        pos: Vec2D,
        mods: &MouseMods,
    ) -> MaybeMouseBehavior {
        let control = Rc::clone(&self.state.borrow().control);
        Some(Box::new(InputBehavior(control)))
    }

    fn on_hover_impl(self: &Rc<Self>, _pos: Vec2D) -> Option<()> {
        let this_state = self.state.borrow();
        self.with_gui_state_mut(|state| {
            state.set_tooltip(Tooltip {
                text: this_state.tooltip.clone(),
                interaction: InteractionHint::LeftClick | InteractionHint::LeftClickAndDrag,
            });
        });
        Some(())
    }

    fn draw_impl(self: &Rc<Self>, g: &mut Renderer) {
        let hovered = self.parents.module.is_hovered();
        let state = self.state.borrow();
        let control = state.control.borrow();
        const W: f32 = WIDTH;
        const H: f32 = HEIGHT;
        const CS: f32 = CORNER_SIZE;
        const JS: f32 = JACK_SIZE;
        const JIP: f32 = JACK_ICON_PADDING;

        // TODO: Highlighting. unimplemented!()
        g.set_color(&COLOR_FG1);

        if let Some(default) = control.get_used_default() {
            let icon = self
                .with_gui_state(|state| state.registry.borrow().lookup_icon(default.icon).unwrap());
            g.draw_pie(0, JS, 0.0, 0.0, TAU);
            g.draw_rect((JS / 2.0, 0.0), JS);
            g.draw_icon(icon, JIP, JS - JIP * 2.0);
        }

        g.draw_rounded_rect((JS, 0.0), JS, CS);
        g.draw_rect((JS, 0.0), (CS, JS));

        if let Some(icon) = state.small_icon {
            const JSIS: f32 = JACK_SMALL_ICON_SIZE;
            const X: f32 = JS * 2.0 - JSIS / 2.0;
            const Y: f32 = JS - JSIS - JIP;
            g.draw_rounded_rect((X - JIP, Y - JIP), JSIS + JIP * 2.0, CS);
            g.draw_icon(icon, (X, Y), JSIS);
        }
        g.draw_icon(state.icon, (JS + JIP, JIP), JS - JIP * 2.0);

        if hovered {
            if let Some(default) = control.get_used_default() {
                g.draw_text(
                    FONT_SIZE,
                    (-100.0 - GRID_P, -JS / 2.0),
                    (100.0, JS),
                    (1, 1),
                    1,
                    &state.label,
                );
                g.draw_text(
                    FONT_SIZE,
                    (-100.0 - GRID_P, JS / 2.0),
                    (100.0, JS),
                    (1, -1),
                    1,
                    &format!("({})", &default.name),
                );
            } else {
                g.draw_text(
                    FONT_SIZE,
                    (-100.0 - GRID_P + JS, 0.0),
                    (100.0, JS),
                    (1, 0),
                    1,
                    &state.label,
                );
            }
        }
    }
}

impl ModuleWidgetImpl for Input {}
