use crate::engine::parts as ep;
use crate::engine::registry::Registry;
use crate::gui::constants::*;
use crate::gui::graphics::GrahpicsWrapper;
use crate::gui::{MouseAction, MouseMods};

fn bound_check(coord: (i32, i32), bounds: (i32, i32)) -> bool {
    coord.0 >= 0 && coord.1 >= 0 && coord.0 <= bounds.0 && coord.1 <= bounds.1
}

pub struct MenuBar {
    tab_icons: Vec<usize>,
}

impl MenuBar {
    pub const HEIGHT: i32 = FATGRID_1;

    pub fn create(registry: &Registry) -> Self {
        Self {
            tab_icons: vec![
                registry.lookup_icon("base:module").unwrap(),
                registry.lookup_icon("base:add").unwrap(),
            ],
        }
    }

    pub fn respond_to_mouse_press(&self, mouse_pos: (i32, i32), mods: &MouseMods) -> MouseAction {
        if !bound_check(mouse_pos, (99999, Self::HEIGHT)) {
            return MouseAction::None;
        }
        let new_screen = mouse_pos.0 / (GRID_P + GRID_1);
        if new_screen < self.tab_icons.len() as i32 {
            MouseAction::SwitchScreen(new_screen as usize)
        } else {
            MouseAction::None
        }
    }

    pub fn draw(&self, width: i32, current_screen_index: usize, g: &mut GrahpicsWrapper) {
        let current_screen_index = current_screen_index as i32;

        g.set_color(&COLOR_SURFACE);
        g.fill_rect(0, 0, width, Self::HEIGHT);

        const GP: i32 = GRID_P;
        const GP2: i32 = GRID_P / 2;
        const MCS: i32 = MODULE_CORNER_SIZE;

        g.set_color(&COLOR_IO_AREA);
        g.fill_rounded_rect(
            coord(0) - GP2,
            coord(0) - GP2,
            grid(self.tab_icons.len() as i32) + GP,
            grid(1) + GP,
            MCS,
        );

        g.fill_rounded_rect(
            coord(self.tab_icons.len() as i32),
            coord(0) - GP2,
            width,
            grid(1) + GP,
            MCS,
        );

        for (index, icon) in self.tab_icons.iter().enumerate() {
            let index = index as i32;
            if current_screen_index == index {
                g.set_color(&COLOR_TEXT);
                g.fill_rounded_rect(
                    coord(index) - GP2,
                    coord(0) - GP2,
                    grid(1) + GP,
                    grid(1) + GP,
                    MCS,
                );
            }
            g.draw_icon(*icon, coord(index), coord(0), grid(1));
        }
    }
}

struct ModuleLibraryEntry {
    name: String,
    input_icons: Vec<usize>,
    output_icons: Vec<usize>,
    prototype: ep::Module,
}

impl ModuleLibraryEntry {
    const WIDTH: i32 = fatgrid(6);
    const HEIGHT: i32 = FATGRID_1;

    fn from(module: &ep::Module) -> Self {
        let name = module.gui_outline.borrow().label.clone();
        let input_icons = module
            .input_jacks
            .iter()
            .map(|jack| jack.get_icon_index())
            .collect();
        let output_icons = module
            .output_jacks
            .iter()
            .map(|jack| jack.get_icon_index())
            .collect();
        Self {
            name,
            input_icons,
            output_icons,
            prototype: module.clone(),
        }
    }

    fn draw(&self, g: &mut GrahpicsWrapper) {
        const MCS: i32 = MODULE_CORNER_SIZE;
        const BAND_SIZE: i32 = GRID_P;
        const ICON_SPACE: i32 = FATGRID_1 / 2;
        const ICON_PADDING: i32 = 2;
        const ICON_SIZE: i32 = (ICON_SPACE * 2 - ICON_PADDING * 4) / 2;

        let num_ports = self.input_icons.len().max(self.output_icons.len()) as i32;
        let port_space = ICON_PADDING + (ICON_PADDING + ICON_SIZE) * num_ports;
        g.set_color(&COLOR_SURFACE);
        let main_width = Self::WIDTH - port_space;
        g.fill_rounded_rect(0, 0, main_width + BAND_SIZE, Self::HEIGHT, MCS);
        g.set_color(&COLOR_TEXT);
        g.fill_rounded_rect(main_width, 0, port_space + BAND_SIZE, Self::HEIGHT, MCS);
        g.set_color(&COLOR_IO_AREA);
        g.fill_rect(main_width, 0, BAND_SIZE, Self::HEIGHT);
        g.fill_rect(main_width + BAND_SIZE, Self::HEIGHT / 2, port_space, 1);

        g.set_color(&COLOR_TEXT);
        g.write_label(GRID_P, 10, main_width - GRID_P / 2, &self.name);
        for (index, icon) in self.input_icons.iter().enumerate() {
            let index = index as i32;
            let x = main_width + BAND_SIZE + ICON_PADDING + (ICON_SIZE + ICON_PADDING) * index;
            g.draw_icon(*icon, x, ICON_PADDING, ICON_SIZE);
        }
        for (index, icon) in self.output_icons.iter().enumerate() {
            let index = index as i32;
            let x = main_width + BAND_SIZE + ICON_PADDING + (ICON_SIZE + ICON_PADDING) * index;
            g.draw_icon(*icon, x, ICON_SIZE + ICON_PADDING * 3, ICON_SIZE);
        }
    }
}

pub struct ModuleLibrary {
    pos: (i32, i32),
    size: (i32, i32),
    vertical_stacking: i32,
    entries: Vec<ModuleLibraryEntry>,
}

impl ModuleLibrary {
    pub fn create(registry: &Registry, pos: (i32, i32), size: (i32, i32)) -> Self {
        let entries = registry
            .iterate_over_modules()
            .map(|module| ModuleLibraryEntry::from(module))
            .collect();
        let vertical_stacking = size.1 / (ModuleLibraryEntry::HEIGHT + GRID_P);
        Self {
            pos,
            size,
            vertical_stacking,
            entries,
        }
    }

    pub fn respond_to_mouse_press(
        &mut self,
        mouse_pos: (i32, i32),
        mods: &MouseMods,
    ) -> MouseAction {
        let mouse_pos = (mouse_pos.0 - self.pos.0, mouse_pos.1 - self.pos.1);
        let clicked_index = mouse_pos.0 / (ModuleLibraryEntry::WIDTH + GRID_P)
            * self.vertical_stacking
            + mouse_pos.1 / (ModuleLibraryEntry::HEIGHT + GRID_P);
        let clicked_index = clicked_index as usize;
        if clicked_index < self.entries.len() {
            MouseAction::AddModule(self.entries[clicked_index].prototype.clone())
        } else {
            MouseAction::None
        }
    }

    pub fn draw(&self, g: &mut GrahpicsWrapper) {
        g.push_state();
        g.apply_offset(self.pos.0, self.pos.1);

        for (index, entry) in self.entries.iter().enumerate() {
            let index = index as i32;
            let (x, y) = (
                (index / self.vertical_stacking) * (ModuleLibraryEntry::WIDTH + GRID_P) + GRID_P,
                (index % self.vertical_stacking) * (ModuleLibraryEntry::HEIGHT + GRID_P) + GRID_P,
            );
            g.push_state();
            g.apply_offset(x, y);
            entry.draw(g);
            g.pop_state();
        }

        g.pop_state();
    }
}
