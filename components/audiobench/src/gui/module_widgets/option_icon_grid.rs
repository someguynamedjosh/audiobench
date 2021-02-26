use std::collections::HashMap;

use crate::{
    engine::controls::OptionChoiceControl,
    gui::{
        constants::*, module_widgets::ModuleWidgetImpl, mouse_behaviors::MutateControl,
        InteractionHint, Tooltip,
    },
    registry::yaml::YamlNode,
    scui_config::{DropTarget, MaybeMouseBehavior, Renderer},
};
use scui::{MouseMods, Vec2D, Widget, WidgetImpl};
use shared_util::prelude::*;

yaml_widget_boilerplate::make_widget_outline! {
    widget_struct: OptionIconGrid,
    constructor: new(
        parent: ParentRef,
        pos: GridPos,
        size: GridSize,
        control: OptionChoiceControlRef,
        icons: IconList,
        label: String,
        tooltip: String,
    ),
}

#[derive(Clone, Debug)]
pub struct IconList(Vec<usize>);

impl IconList {
    pub fn from_yaml(
        node: Option<&YamlNode>,
        icon_indexes: &HashMap<String, usize>,
    ) -> Result<IconList, String> {
        let node = node.ok_or_else(|| format!("Missing child 'icons'"))?;
        let mut icon_names: Vec<&str> = icon_indexes.keys().map(|k| &k[..]).collect();
        icon_names.sort_unstable(); // This is purely for the sake of a nicer error message.
        let icon_indexes = node.parse_custom(|content| {
            let pieces = content.split(',');
            pieces
                .map(|piece| {
                    if let Some(index) = icon_indexes.get(piece.trim()) {
                        Ok(*index)
                    } else {
                        Err(format!(
                            "{} is not a valid icon name, expected one of: {}",
                            piece,
                            icon_names.join(", ")
                        ))
                    }
                })
                .collect()
        })?;
        Ok(Self(icon_indexes))
    }
}

scui::widget! {
    pub OptionIconGrid
    State {
        pos: Vec2D,
        size: Vec2D,
        control: Rcrc<OptionChoiceControl>,
        icons: IconList,
        label: String,
        tooltip: String,
    }
}

impl OptionIconGrid {
    fn new(
        parent: &impl OptionIconGridParent,
        pos: Vec2D,
        size: Vec2D,
        control: Rcrc<OptionChoiceControl>,
        icons: IconList,
        label: String,
        tooltip: String,
    ) -> Rc<Self> {
        let int = parent.provide_gui_interface();
        let gui_state = int.state.borrow();
        let registry = gui_state.registry.borrow();
        let state = OptionIconGridState {
            pos,
            size,
            control,
            icons,
            label,
            tooltip,
        };
        Rc::new(Self::create(parent, state))
    }
}

const TARGET_ICON_WIDTH: f32 = grid(1) * 3.8 / 4.0;

impl WidgetImpl<Renderer, DropTarget> for OptionIconGrid {
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
        let width = self.get_size().x;
        let icons_per_line = (width / TARGET_ICON_WIDTH).round() as usize;
        let icon_size = width / icons_per_line as f32;
        let option = (pos.x / icon_size).floor() as usize
            + (pos.y / icon_size).floor() as usize * icons_per_line;
        let state = self.state.borrow();
        let control = state.control.borrow();
        if option < control.get_options().len() {
            drop(control);
            let cref = Rc::clone(&state.control);
            MutateControl::wrap(self, move || cref.borrow_mut().set_selected_option(option))
        } else {
            None
        }
    }

    fn on_hover_impl(self: &Rc<Self>, _pos: Vec2D) -> Option<()> {
        let tooltip = Tooltip {
            text: self.state.borrow().tooltip.clone(),
            interaction: vec![InteractionHint::LeftClick, InteractionHint::DoubleClick],
        };
        self.with_gui_state_mut(|state| {
            state.set_tooltip(tooltip);
        });
        Some(())
    }

    fn draw_impl(self: &Rc<Self>, g: &mut Renderer) {
        let state = self.state.borrow();
        const CS: f32 = CORNER_SIZE;
        const GP: f32 = GRID_P;
        g.set_color(&COLOR_BG0);
        let width = self.get_size().x;
        let icons_per_line = (width / TARGET_ICON_WIDTH).round() as usize;
        let icon_size = width / icons_per_line as f32;
        // Don't ask why GP / 2 and not just GP, it just looks better and I don't know why.
        let h = state.size.y - FONT_SIZE - GRID_P / 2.0;
        let h = (h / icon_size).floor() * icon_size;
        g.draw_rounded_rect(0, (state.size.x, h), CS);
        let current_option = state.control.borrow().get_selected_option();
        let mut x = 0.0;
        let mut y = 0.0;
        g.set_color(&COLOR_FG1);
        for (index, option) in state.control.borrow().get_options().iter().enumerate() {
            if index == current_option {
                g.draw_rounded_rect((x, y), icon_size, CORNER_SIZE);
                g.draw_icon(
                    state.icons.0.get(index).map(Clone::clone).unwrap_or(0),
                    (x, y),
                    icon_size,
                );
            } else {
                g.draw_white_icon(
                    state.icons.0.get(index).map(Clone::clone).unwrap_or(0),
                    (x, y),
                    icon_size,
                );
            }
            x += icon_size;
            if (index + 1) % icons_per_line == 0 {
                x = 0.0;
                y += icon_size;
            }
        }
        let label = format!(
            "{}: {}",
            state.label,
            state.control.borrow().get_options()[current_option]
        );
        g.draw_text(FONT_SIZE, 0.0, state.size, (0, 1), 1, &label[..]);
    }
}

impl ModuleWidgetImpl for OptionIconGrid {}
