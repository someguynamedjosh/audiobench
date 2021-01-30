use super::ModuleWidgetImpl;
use crate::gui::constants::*;
use crate::gui::top_level::graph::{ConnectToControl, Module, ModuleGraph};
use crate::gui::{InteractionHint, Tooltip};
use crate::scui_config::{DropTarget, MaybeMouseBehavior, Renderer};
use crate::{
    engine::{
        controls::{Control, InputControl},
        UiThreadEngine,
    },
    registry::{yaml::YamlNode, Registry},
};
use scui::{MouseBehavior, MouseMods, Vec2D, Widget, WidgetImpl};
use shared_util::prelude::*;
use std::{collections::HashMap, f32::consts::TAU};

yaml_widget_boilerplate::make_widget_outline! {
    widget_struct: Input,
    constructor: new(
        parent: ParentRef,
        y: i32,
        control: InputControlRef,
        label: String,
        tooltip: String,
        icon: OptionalIcon,
    ),
}

#[derive(Clone, Debug)]
pub struct OptionalIcon(Option<usize>);

impl OptionalIcon {
    pub fn from_yaml(
        node: Option<&YamlNode>,
        icon_indexes: &HashMap<String, usize>,
    ) -> Result<OptionalIcon, String> {
        if let Some(node) = node {
            let mut icon_names: Vec<&str> = icon_indexes.keys().map(|k| &k[..]).collect();
            icon_names.sort_unstable(); // This is purely for the sake of a nicer error message.
            let name_index = node.parse_enumerated(&icon_names[..])?;
            let name = icon_names[name_index];
            Ok(Self(Some(*icon_indexes.get(name).unwrap())))
        } else {
            Ok(Self(None))
        }
    }
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
        graph: Rc<ModuleGraph>,
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
        optional_icon: OptionalIcon,
    ) -> Rc<Self> {
        let pos = Vec2D::new(0.0, coord(y));
        let int = parent.provide_gui_interface();
        let gui_state = int.state.borrow();
        let registry = gui_state.registry.borrow();
        let mut icon = registry
            .lookup_icon(control.borrow().get_type().icon_name())
            .unwrap();
        let mut small_icon = None;
        if let Some(index) = optional_icon.0 {
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

struct InputBehavior {
    engine: Rcrc<UiThreadEngine>,
    control: Rcrc<InputControl>,
    connector: Box<ConnectToControl>,
}

impl MouseBehavior<DropTarget> for InputBehavior {
    fn on_click(self: Box<Self>) {
        let mut control = self.control.borrow_mut();
        if control.get_used_default().is_some() {
            control.next_default();
        }
        drop(control);
        self.connector.on_click();
        self.engine.borrow_mut().regenerate_code();
    }

    fn on_drop(self: Box<Self>, drop_target: Option<DropTarget>) {
        self.connector.on_drop(drop_target)
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
        let engine = self.with_gui_state(|state| Rc::clone(&state.engine));
        let control = Rc::clone(&self.state.borrow().control);
        let pos = self.get_pos() + self.parents.module.get_pos() + self.get_size() / 2.0;
        let g = &self.parents.graph;
        let connector = g.connect_to_control_behavior(Rc::clone(&control) as _, pos);
        Some(Box::new(InputBehavior {
            engine,
            control,
            connector,
        }))
    }

    fn on_hover_impl(self: &Rc<Self>, _pos: Vec2D) -> Option<()> {
        let this_state = self.state.borrow();
        self.with_gui_state_mut(|state| {
            state.set_tooltip(Tooltip {
                text: this_state.tooltip.clone(),
                interaction: vec![
                    InteractionHint::LeftClick,
                    InteractionHint::LeftClickAndDrag,
                ],
            });
        });
        Some(())
    }

    fn draw_impl(self: &Rc<Self>, g: &mut Renderer) {
        let hovered = self.parents.module.is_hovered();
        let state = self.state.borrow();
        let hmode = self.parents.graph.get_highlight_mode();
        let dim = hmode.should_dim(&state.control);
        let control = state.control.borrow();
        const CS: f32 = CORNER_SIZE;
        const JS: f32 = JACK_SIZE;
        const JIP: f32 = JACK_ICON_PADDING;

        if dim {
            g.set_color(&COLOR_FG0);
        } else {
            g.set_color(&COLOR_FG1);
        }

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

impl ModuleWidgetImpl for Input {
    fn represented_control(self: &Rc<Self>) -> Option<Rcrc<dyn Control>> {
        Some(Rc::clone(&self.state.borrow().control) as _)
    }

    fn use_input_style_wires(self: &Rc<Self>) -> bool {
        true
    }
}
