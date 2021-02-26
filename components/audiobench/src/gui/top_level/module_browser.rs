use crate::{
    engine::controls::AnyControl,
    gui::{
        constants::*, graphics::GrahpicsWrapper, top_level::graph::ModuleGraph, GuiTab,
        InteractionHint, TabArchetype, Tooltip,
    },
    registry::{module_template::ModuleTemplate, Registry},
    scui_config::{DropTarget, MaybeMouseBehavior, Renderer},
};
use owning_ref::OwningRef;
use scui::{MouseMods, OnClickBehavior, Vec2D, WidgetImpl};
use shared_util::prelude::*;
use std::cell::Ref;
use std::collections::HashSet;

struct ModuleBrowserEntry {
    name: String,
    category: String,
    input_icons: Vec<usize>,
    output_icons: Vec<usize>,
    template: Rcrc<ModuleTemplate>,
}

impl ModuleBrowserEntry {
    const WIDTH: f32 = fatgrid(6);
    const HEIGHT: f32 = fatgrid(1);

    fn from(registry: &Registry, template: &Rcrc<ModuleTemplate>) -> Self {
        let template_ref = template.borrow();
        let name = template_ref.label.clone();
        let category = template_ref.category.clone();
        let mut input_icons = Vec::new();
        for (_, control) in &template_ref.default_controls {
            if let AnyControl::Input(input) = control {
                let icon_name = input.borrow().get_type().icon_name();
                input_icons.push(registry.lookup_icon(icon_name).unwrap());
            }
        }
        let output_icons = template_ref.outputs.imc(|jack| jack.get_icon_index());
        drop(template_ref);
        Self {
            name,
            category,
            input_icons,
            output_icons,
            template: Rc::clone(template),
        }
    }

    fn draw(&self, g: &mut GrahpicsWrapper) {
        const CS: f32 = CORNER_SIZE;
        const BAND_SIZE: f32 = GRID_P;
        const ICON_SPACE: f32 = fatgrid(1) / 2.0;
        const ICON_PADDING: f32 = 2.0;
        const ICON_SIZE: f32 = (ICON_SPACE * 2.0 - ICON_PADDING * 4.0) / 2.0;

        let num_ports = self.input_icons.len().max(self.output_icons.len()) as f32;
        let port_space = ICON_PADDING + (ICON_PADDING + ICON_SIZE) * num_ports;
        g.set_color(&COLOR_BG2);
        let main_width = Self::WIDTH - port_space - BAND_SIZE;
        g.draw_rounded_rect(0, (main_width + BAND_SIZE, Self::HEIGHT), CS);
        g.set_color(&COLOR_FG1);
        g.draw_rounded_rect(
            (main_width, 0.0),
            (port_space + BAND_SIZE, Self::HEIGHT),
            CS,
        );
        g.set_color(&COLOR_BG1);
        g.draw_rect((main_width, 0.0), (BAND_SIZE, Self::HEIGHT));
        g.draw_rect(
            (main_width + BAND_SIZE, Self::HEIGHT / 2.0),
            (port_space, 1.0),
        );

        g.set_color(&COLOR_FG1);
        g.draw_text(
            FONT_SIZE,
            (GRID_P, 0.0),
            (main_width - GRID_P / 2.0, Self::HEIGHT),
            (-1, 0),
            1,
            &self.name,
        );
        for (index, icon) in self.input_icons.iter().enumerate() {
            let index = index as f32;
            let x = main_width + BAND_SIZE + ICON_PADDING + (ICON_SIZE + ICON_PADDING) * index;
            g.draw_icon(*icon, (x, ICON_PADDING), ICON_SIZE);
        }
        for (index, icon) in self.output_icons.iter().enumerate() {
            let index = index as f32;
            let x = main_width + BAND_SIZE + ICON_PADDING + (ICON_SIZE + ICON_PADDING) * index;
            g.draw_icon(*icon, (x, ICON_SIZE + ICON_PADDING * 3.0), ICON_SIZE);
        }
    }
}

enum VisualEntry {
    RealEntry(usize),
    Label(String),
}

#[derive(Clone, Copy)]
enum SortMethod {
    Alphabetical,
    Categorical,
}

scui::widget! {
    pub ModuleBrowser
    State {
        add_to_graph: Rc<ModuleGraph>,
        vertical_stacking: usize,
        entries: Vec<ModuleBrowserEntry>,
        alphabetical_list: Vec<VisualEntry>,
        categorical_list: Vec<VisualEntry>,
        current_sort: SortMethod,
    }
}

impl ModuleBrowser {
    pub fn new(parent: &impl ModuleBrowserParent, add_to_graph: Rc<ModuleGraph>) -> Rc<Self> {
        let inter = parent.provide_gui_interface();
        let state = inter.state.borrow();
        let registry = state.registry.borrow();

        let entries: Vec<_> = registry
            .borrow_templates()
            .imc(|module| ModuleBrowserEntry::from(&*registry, module));
        let vertical_stacking =
            (TAB_BODY_HEIGHT / (ModuleBrowserEntry::HEIGHT + GRID_P)).floor() as usize;

        let mut alphabetical_order: Vec<_> = (0..entries.len()).collect();
        alphabetical_order.sort_by(|a, b| entries[*a].name.cmp(&entries[*b].name));
        let mut alphabetical_list = Vec::with_capacity(entries.len() + 26);
        let mut last_starting_char = 'Z';
        for entry_index in alphabetical_order.iter().cloned() {
            let starting_char = entries[entry_index].name.chars().next().unwrap_or('Z');
            let starting_char = starting_char.to_ascii_uppercase();
            if starting_char != last_starting_char {
                last_starting_char = starting_char;
                alphabetical_list.push(VisualEntry::Label(format!("{}", starting_char)));
            }
            alphabetical_list.push(VisualEntry::RealEntry(entry_index));
        }

        let categories: HashSet<_> = entries.iter().map(|e| e.category.clone()).collect();
        let mut categories: Vec<_> = categories.iter().collect();
        categories.sort_unstable();
        let mut categorical_list = Vec::with_capacity(entries.len() + categories.len());
        for category in categories {
            categorical_list.push(VisualEntry::Label(category.clone()));
            for index in alphabetical_order.iter().cloned() {
                if entries[index].category == *category {
                    categorical_list.push(VisualEntry::RealEntry(index));
                }
            }
        }

        let state = ModuleBrowserState {
            add_to_graph,
            vertical_stacking,
            entries,
            alphabetical_list,
            categorical_list,
            current_sort: SortMethod::Categorical,
        };

        Rc::new(Self::create(parent, state))
    }

    fn get_current_list(&self) -> OwningRef<Ref<ModuleBrowserState>, Vec<VisualEntry>> {
        let state = self.state.borrow();
        let sort = state.current_sort;
        let oref = OwningRef::from(state);
        match sort {
            SortMethod::Alphabetical => oref.map(|s| &s.alphabetical_list),
            SortMethod::Categorical => oref.map(|s| &s.categorical_list),
        }
    }

    fn get_entry_at(
        &self,
        mouse_pos: Vec2D,
    ) -> Option<OwningRef<Ref<ModuleBrowserState>, ModuleBrowserEntry>> {
        let state = self.state.borrow();
        if mouse_pos.x < 0.0 || mouse_pos.y < 0.0 {
            return None;
        }
        let clicked_index = (mouse_pos.x / (ModuleBrowserEntry::WIDTH + GRID_P)) as usize
            * state.vertical_stacking
            + (mouse_pos.y / (ModuleBrowserEntry::HEIGHT + GRID_P)) as usize;
        let list = self.get_current_list();
        if clicked_index < list.len() {
            let entry = &list[clicked_index];
            if let VisualEntry::RealEntry(index) = entry {
                let oref = OwningRef::from(state);
                Some(oref.map(|s| &s.entries[*index]))
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl WidgetImpl<Renderer, DropTarget> for ModuleBrowser {
    fn get_pos_impl(self: &Rc<Self>) -> Vec2D {
        (0.0, HEADER_HEIGHT).into()
    }

    fn get_size_impl(self: &Rc<Self>) -> Vec2D {
        (TAB_BODY_WIDTH, TAB_BODY_HEIGHT).into()
    }

    fn get_mouse_behavior_impl(
        self: &Rc<Self>,
        pos: Vec2D,
        _mods: &MouseMods,
    ) -> MaybeMouseBehavior {
        if let Some(entry) = self.get_entry_at(pos) {
            let add_to_graph = Rc::clone(&self.state.borrow().add_to_graph);
            let template = Rc::clone(&entry.template);
            OnClickBehavior::wrap(move || {
                add_to_graph.add_module(template);
            })
        } else {
            None
        }
    }

    fn on_hover_impl(self: &Rc<Self>, pos: Vec2D) -> Option<()> {
        if let Some(entry) = self.get_entry_at(pos) {
            let tt = Tooltip {
                text: entry.template.borrow().tooltip.clone(),
                interaction: vec![InteractionHint::LeftClick],
            };
            self.with_gui_state_mut(|state| state.set_tooltip(tt));
            Some(())
        } else {
            None
        }
    }

    fn draw_impl(self: &Rc<Self>, g: &mut GrahpicsWrapper) {
        let state = self.state.borrow();
        let list = self.get_current_list();
        for (index, entry) in list.iter().enumerate() {
            let (x, y) = (
                (index / state.vertical_stacking) as f32 * (ModuleBrowserEntry::WIDTH + GRID_P)
                    + GRID_P,
                (index % state.vertical_stacking) as f32 * (ModuleBrowserEntry::HEIGHT + GRID_P)
                    + GRID_P,
            );
            g.push_state();
            g.translate((x, y));
            match entry {
                VisualEntry::RealEntry(index) => state.entries[*index].draw(g),
                VisualEntry::Label(text) => {
                    g.set_color(&COLOR_FG1);
                    g.draw_text(
                        BIG_FONT_SIZE,
                        0,
                        (ModuleBrowserEntry::WIDTH, ModuleBrowserEntry::HEIGHT),
                        (0, 0),
                        1,
                        text,
                    )
                }
            }
            g.pop_state();
        }
    }
}

impl GuiTab for Rc<ModuleBrowser> {
    fn get_name(self: &Self) -> String {
        format!("Add A Module")
    }

    fn get_archetype(&self) -> TabArchetype {
        TabArchetype::ModuleBrowser(Rc::clone(&self.state.borrow().add_to_graph))
    }
}
