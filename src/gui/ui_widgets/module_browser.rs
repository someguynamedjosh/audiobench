use crate::engine::parts as ep;
use crate::registry::Registry;
use crate::gui::action::MouseAction;
use crate::gui::constants::*;
use crate::gui::graphics::{GrahpicsWrapper, HAlign, VAlign};
use crate::gui::{InteractionHint, MouseMods, Tooltip};
use crate::util::*;
use std::collections::HashSet;

struct ModuleBrowserEntry {
    name: String,
    category: String,
    input_icons: Vec<usize>,
    output_icons: Vec<usize>,
    prototype: ep::Module,
}

impl ModuleBrowserEntry {
    const WIDTH: f32 = fatgrid(6);
    const HEIGHT: f32 = fatgrid(1);

    fn from(module: &ep::Module) -> Self {
        let template_ref = module.template.borrow();
        let name = template_ref.label.clone();
        let category = template_ref.category.clone();
        let input_icons = template_ref
            .inputs
            .iter()
            .map(|jack| jack.get_icon_index())
            .collect();
        let output_icons = template_ref
            .outputs
            .iter()
            .map(|jack| jack.get_icon_index())
            .collect();
        Self {
            name,
            category,
            input_icons,
            output_icons,
            prototype: module.clone(),
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
        g.set_color(&COLOR_SURFACE);
        let main_width = Self::WIDTH - port_space - BAND_SIZE;
        g.fill_rounded_rect(0.0, 0.0, main_width + BAND_SIZE, Self::HEIGHT, CS);
        g.set_color(&COLOR_TEXT);
        g.fill_rounded_rect(main_width, 0.0, port_space + BAND_SIZE, Self::HEIGHT, CS);
        g.set_color(&COLOR_IO_AREA);
        g.fill_rect(main_width, 0.0, BAND_SIZE, Self::HEIGHT);
        g.fill_rect(main_width + BAND_SIZE, Self::HEIGHT / 2.0, port_space, 1.0);

        g.set_color(&COLOR_TEXT);
        g.write_text(
            FONT_SIZE,
            GRID_P,
            0.0,
            main_width - GRID_P / 2.0,
            Self::HEIGHT,
            HAlign::Left,
            VAlign::Center,
            1,
            &self.name,
        );
        for (index, icon) in self.input_icons.iter().enumerate() {
            let index = index as f32;
            let x = main_width + BAND_SIZE + ICON_PADDING + (ICON_SIZE + ICON_PADDING) * index;
            g.draw_icon(*icon, x, ICON_PADDING, ICON_SIZE);
        }
        for (index, icon) in self.output_icons.iter().enumerate() {
            let index = index as f32;
            let x = main_width + BAND_SIZE + ICON_PADDING + (ICON_SIZE + ICON_PADDING) * index;
            g.draw_icon(*icon, x, ICON_SIZE + ICON_PADDING * 3.0, ICON_SIZE);
        }
    }
}

enum VisualEntry {
    RealEntry(usize),
    Label(String),
}

enum SortMethod {
    Alphabetical,
    Categorical,
}

pub struct ModuleBrowser {
    pos: (f32, f32),
    size: (f32, f32),
    vertical_stacking: usize,
    entries: Vec<ModuleBrowserEntry>,
    alphabetical_list: Vec<VisualEntry>,
    categorical_list: Vec<VisualEntry>,
    current_sort: SortMethod,
}

impl ModuleBrowser {
    pub fn create(registry: &Registry, pos: (f32, f32), size: (f32, f32)) -> Self {
        let entries: Vec<_> = registry
            .borrow_modules()
            .imc(|module| ModuleBrowserEntry::from(module));
        let vertical_stacking = (size.1 / (ModuleBrowserEntry::HEIGHT + GRID_P)).floor() as usize;

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

        Self {
            pos,
            size,
            vertical_stacking,
            entries,
            alphabetical_list,
            categorical_list,
            current_sort: SortMethod::Categorical,
        }
    }

    fn get_current_list(&self) -> &Vec<VisualEntry> {
        match self.current_sort {
            SortMethod::Alphabetical => &self.alphabetical_list,
            SortMethod::Categorical => &self.categorical_list,
        }
    }

    fn get_entry_at(&self, mouse_pos: (f32, f32)) -> Option<&ModuleBrowserEntry> {
        let mouse_pos = (mouse_pos.0 - self.pos.0, mouse_pos.1 - self.pos.1);
        let clicked_index = (mouse_pos.0 / (ModuleBrowserEntry::WIDTH + GRID_P)) as usize
            * self.vertical_stacking
            + (mouse_pos.1 / (ModuleBrowserEntry::HEIGHT + GRID_P)) as usize;
        let list = self.get_current_list();
        if clicked_index < list.len() {
            let entry = &list[clicked_index];
            if let VisualEntry::RealEntry(index) = entry {
                Some(&self.entries[*index])
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn get_tooltip_at(&self, mouse_pos: (f32, f32)) -> Option<Tooltip> {
        if let Some(entry) = self.get_entry_at(mouse_pos) {
            Some(Tooltip {
                text: entry.prototype.template.borrow().tooltip.clone(),
                interaction: InteractionHint::LeftClick.into(),
            })
        } else {
            None
        }
    }

    pub fn respond_to_mouse_press(
        &mut self,
        mouse_pos: (f32, f32),
        mods: &MouseMods,
    ) -> MouseAction {
        if let Some(entry) = self.get_entry_at(mouse_pos) {
            MouseAction::AddModule(entry.prototype.clone())
        } else {
            MouseAction::None
        }
    }

    pub fn draw(&self, g: &mut GrahpicsWrapper) {
        g.push_state();
        g.apply_offset(self.pos.0, self.pos.1);

        let list = self.get_current_list();
        for (index, entry) in list.iter().enumerate() {
            let (x, y) = (
                (index / self.vertical_stacking) as f32 * (ModuleBrowserEntry::WIDTH + GRID_P)
                    + GRID_P,
                (index % self.vertical_stacking) as f32 * (ModuleBrowserEntry::HEIGHT + GRID_P)
                    + GRID_P,
            );
            g.push_state();
            g.apply_offset(x, y);
            match entry {
                VisualEntry::RealEntry(index) => self.entries[*index].draw(g),
                VisualEntry::Label(text) => {
                    g.set_color(&COLOR_TEXT);
                    g.write_text(
                        BIG_FONT_SIZE,
                        0.0,
                        0.0,
                        ModuleBrowserEntry::WIDTH,
                        ModuleBrowserEntry::HEIGHT,
                        HAlign::Center,
                        VAlign::Center,
                        1,
                        text,
                    )
                }
            }
            g.pop_state();
        }

        g.pop_state();
    }
}
