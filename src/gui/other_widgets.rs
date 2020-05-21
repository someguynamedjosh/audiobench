use crate::engine::parts as ep;
use crate::engine::registry::Registry;
use crate::gui::action::MouseAction;
use crate::gui::constants::*;
use crate::gui::graphics::{GrahpicsWrapper, HAlign, VAlign};
use crate::gui::{InteractionHint, MouseMods, Tooltip};

fn bound_check(coord: (i32, i32), bounds: (i32, i32)) -> bool {
    coord.0 >= 0 && coord.1 >= 0 && coord.0 <= bounds.0 && coord.1 <= bounds.1
}

pub struct MenuBar {
    tab_icons: Vec<usize>,
    lclick: usize,
    rclick: usize,
    drag: usize,
    and: usize,
    tooltip: Tooltip,
}

impl MenuBar {
    pub const HEIGHT: i32 = grid(1) + GRID_P * 3;

    pub fn create(registry: &Registry) -> Self {
        Self {
            tab_icons: vec![
                registry.lookup_icon("base:note").unwrap(),
                registry.lookup_icon("base:add").unwrap(),
            ],
            lclick: registry.lookup_icon("base:left_click").unwrap(),
            rclick: registry.lookup_icon("base:right_click").unwrap(),
            drag: registry.lookup_icon("base:move").unwrap(),
            and: registry.lookup_icon("base:add").unwrap(),
            tooltip: Default::default(),
        }
    }

    pub fn set_tooltip(&mut self, tooltip: Tooltip) {
        self.tooltip = tooltip;
    }

    pub fn respond_to_mouse_press(&self, mouse_pos: (i32, i32), mods: &MouseMods) -> MouseAction {
        if !bound_check(mouse_pos, (99999, Self::HEIGHT)) {
            return MouseAction::None;
        }
        let new_screen = (mouse_pos.0 - GRID_P) / (GRID_P + grid(1));
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
        const CS: i32 = CORNER_SIZE;
        const HEIGHT: i32 = MenuBar::HEIGHT;
        const ITEM_HEIGHT: i32 = HEIGHT - GP * 2;

        g.set_color(&COLOR_BG);
        g.fill_rounded_rect(
            coord(0),
            coord(0),
            grid(self.tab_icons.len() as i32) + GP,
            ITEM_HEIGHT,
            CS,
        );

        const HINT_HEIGHT: i32 = grid(1) - 2;
        const ICON_PAD: i32 = 1;
        const ICON_SIZE: i32 = HINT_HEIGHT - ICON_PAD * 2;
        const HINT_AREA_WIDTH: i32 = ICON_SIZE * 4 + GP * 3 + ICON_PAD * 6;
        {
            g.set_color(&COLOR_SURFACE);
            g.fill_rounded_rect(
                width - HINT_AREA_WIDTH,
                0,
                HINT_AREA_WIDTH,
                HINT_HEIGHT * 2 + GP * 3,
                CS,
            );

            const Y1: i32 = GP;
            const Y2: i32 = Y1 + HINT_HEIGHT + GP;
            fn hint_width(num_icons: i32) -> i32 {
                (ICON_SIZE + ICON_PAD) * num_icons + ICON_PAD
            }
            fn draw_hint(
                active: bool,
                g: &mut GrahpicsWrapper,
                rx: i32,
                y: i32,
                icons: &[usize],
            ) -> i32 {
                let w = hint_width(icons.len() as i32);
                let x = rx - GP - w;
                if active {
                    g.set_color(&COLOR_TEXT);
                } else {
                    g.set_color(&COLOR_BG);
                }
                g.fill_rounded_rect(x, y, w, HINT_HEIGHT, CS);
                if active {
                    for (index, icon) in icons.iter().enumerate() {
                        g.draw_icon(
                            *icon,
                            x + ICON_PAD + (ICON_PAD + ICON_SIZE) * index as i32,
                            y + ICON_PAD,
                            ICON_SIZE,
                        );
                    }
                }
                x
            }

            g.set_color(&COLOR_BG);
            let active = self
                .tooltip
                .interaction
                .contains(InteractionHint::LeftClickAndDrag);
            let x = draw_hint(active, g, width, Y1, &[self.lclick, self.and, self.drag]);
            let active = self
                .tooltip
                .interaction
                .contains(InteractionHint::LeftClick);
            draw_hint(active, g, x, Y1, &[self.lclick]);

            let active = self
                .tooltip
                .interaction
                .contains(InteractionHint::DoubleClick);
            let x = draw_hint(active, g, width, Y2, &[self.lclick, self.and, self.lclick]);
            let active = self
                .tooltip
                .interaction
                .contains(InteractionHint::RightClick);
            draw_hint(active, g, x, Y2, &[self.rclick]);
        }

        let tooltip_x = coord(self.tab_icons.len() as i32) + GP;
        let tooltip_width = width - HINT_AREA_WIDTH - tooltip_x;
        g.set_color(&COLOR_BG);
        g.fill_rounded_rect(tooltip_x, coord(0), tooltip_width, ITEM_HEIGHT, CS);
        let text_x = tooltip_x + GP;
        let text_width = tooltip_width - GP * 2;
        g.set_color(&COLOR_TEXT);
        g.write_text(
            12,
            text_x,
            coord(0) + GP2,
            text_width,
            grid(1),
            HAlign::Left,
            VAlign::Center,
            1,
            &self.tooltip.text,
        );

        for (index, icon) in self.tab_icons.iter().enumerate() {
            let index = index as i32;
            if current_screen_index == index {
                g.set_color(&COLOR_TEXT);
                g.fill_rounded_rect(
                    coord(index),
                    coord(0),
                    grid(1) + GP,
                    grid(1) + GP,
                    CS,
                );
            }
            g.draw_icon(*icon, coord(index) + GP2, coord(0) + GP2, grid(1));
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
    const HEIGHT: i32 = fatgrid(1);

    fn from(module: &ep::Module) -> Self {
        let template_ref = module.template.borrow();
        let name = template_ref.label.clone();
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
            input_icons,
            output_icons,
            prototype: module.clone(),
        }
    }

    fn draw(&self, g: &mut GrahpicsWrapper) {
        const CS: i32 = CORNER_SIZE;
        const BAND_SIZE: i32 = GRID_P;
        const ICON_SPACE: i32 = fatgrid(1) / 2;
        const ICON_PADDING: i32 = 2;
        const ICON_SIZE: i32 = (ICON_SPACE * 2 - ICON_PADDING * 4) / 2;

        let num_ports = self.input_icons.len().max(self.output_icons.len()) as i32;
        let port_space = ICON_PADDING + (ICON_PADDING + ICON_SIZE) * num_ports;
        g.set_color(&COLOR_SURFACE);
        let main_width = Self::WIDTH - port_space;
        g.fill_rounded_rect(0, 0, main_width + BAND_SIZE, Self::HEIGHT, CS);
        g.set_color(&COLOR_TEXT);
        g.fill_rounded_rect(main_width, 0, port_space + BAND_SIZE, Self::HEIGHT, CS);
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
    alphabetical_order: Vec<usize>,
}

impl ModuleLibrary {
    pub fn create(registry: &Registry, pos: (i32, i32), size: (i32, i32)) -> Self {
        let entries: Vec<_> = registry
            .iterate_over_modules()
            .map(|module| ModuleLibraryEntry::from(module))
            .collect();
        let vertical_stacking = size.1 / (ModuleLibraryEntry::HEIGHT + GRID_P);
        let mut alphabetical_order: Vec<_> = (0..entries.len()).collect();
        alphabetical_order.sort_by(|a, b| entries[*a].name.cmp(&entries[*b].name));
        Self {
            pos,
            size,
            vertical_stacking,
            entries,
            alphabetical_order,
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
            let entry_index = self.alphabetical_order[clicked_index];
            MouseAction::AddModule(self.entries[entry_index].prototype.clone())
        } else {
            MouseAction::None
        }
    }

    pub fn draw(&self, g: &mut GrahpicsWrapper) {
        g.push_state();
        g.apply_offset(self.pos.0, self.pos.1);

        for (index, entry_index) in self.alphabetical_order.iter().enumerate() {
            let entry = &self.entries[*entry_index];
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
