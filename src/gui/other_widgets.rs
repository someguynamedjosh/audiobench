use crate::engine::parts as ep;
use crate::engine::registry::Registry;
use crate::gui::action::MouseAction;
use crate::gui::constants::*;
use crate::gui::graphics::{GrahpicsWrapper, HAlign, VAlign};
use crate::gui::{GuiScreen, InteractionHint, MouseMods, Tooltip};
use crate::util::*;
use std::collections::HashSet;

fn bound_check(coord: (i32, i32), bounds: (i32, i32)) -> bool {
    coord.0 >= 0 && coord.1 >= 0 && coord.0 <= bounds.0 && coord.1 <= bounds.1
}

pub struct MenuBar {
    screens: Vec<GuiScreen>,
    screen_icons: Vec<usize>,
    lclick: usize,
    rclick: usize,
    drag: usize,
    and: usize,
    tooltip: Tooltip,
}

impl MenuBar {
    pub const HEIGHT: i32 = grid(1) + GRID_P * 3;

    pub fn create(registry: &Registry, screens: Vec<GuiScreen>) -> Self {
        let screen_icons = screens
            .iter()
            .map(|s| registry.lookup_icon(s.get_icon_name()).unwrap())
            .collect();
        Self {
            screens,
            screen_icons,
            lclick: registry.lookup_icon("base:left_click").unwrap(),
            rclick: registry.lookup_icon("base:right_click").unwrap(),
            drag: registry.lookup_icon("base:move").unwrap(),
            and: registry.lookup_icon("base:add").unwrap(),
            tooltip: Default::default(),
        }
    }

    pub fn get_tooltip_at(&self, mouse_pos: (i32, i32)) -> Option<Tooltip> {
        if mouse_pos.1 < Self::HEIGHT {
            let screen_index = ((mouse_pos.0 - GRID_P) / (GRID_P + grid(1))) as usize;
            if screen_index < self.screens.len() {
                return Some(Tooltip {
                    text: self.screens[screen_index].get_tooltip_text().to_owned(),
                    interaction: InteractionHint::LeftClick.into(),
                });
            }
        }
        None
    }

    pub fn set_tooltip(&mut self, tooltip: Tooltip) {
        self.tooltip = tooltip;
    }

    pub fn respond_to_mouse_press(&self, mouse_pos: (i32, i32), mods: &MouseMods) -> MouseAction {
        if !bound_check(mouse_pos, (99999, Self::HEIGHT)) {
            return MouseAction::None;
        }
        let new_screen = ((mouse_pos.0 - GRID_P) / (GRID_P + grid(1))) as usize;
        if new_screen < self.screens.len() {
            MouseAction::SwitchScreen(self.screens[new_screen])
        } else {
            MouseAction::None
        }
    }

    pub fn draw(&self, width: i32, current_screen: GuiScreen, g: &mut GrahpicsWrapper) {
        let current_screen_index = self
            .screens
            .iter()
            .position(|e| e == &current_screen)
            .unwrap() as i32;

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
            grid(self.screens.len() as i32) + GP,
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

        let tooltip_x = coord(self.screens.len() as i32) + GP;
        let tooltip_width = width - HINT_AREA_WIDTH - tooltip_x;
        g.set_color(&COLOR_BG);
        g.fill_rounded_rect(tooltip_x, coord(0), tooltip_width, ITEM_HEIGHT, CS);
        let text_x = tooltip_x + GP;
        let text_width = tooltip_width - GP * 2;
        g.set_color(&COLOR_TEXT);
        g.write_text(
            FONT_SIZE,
            text_x,
            coord(0) + GP2,
            text_width,
            grid(1),
            HAlign::Left,
            VAlign::Center,
            1,
            &self.tooltip.text,
        );

        for (index, icon) in self.screen_icons.iter().enumerate() {
            let index = index as i32;
            if current_screen_index == index {
                g.set_color(&COLOR_TEXT);
                g.fill_rounded_rect(coord(index), coord(0), grid(1) + GP, grid(1) + GP, CS);
                g.draw_icon(*icon, coord(index) + GP2, coord(0) + GP2, grid(1));
            } else {
                g.draw_white_icon(*icon, coord(index) + GP2, coord(0) + GP2, grid(1));
            }
        }
    }
}

pub struct TextBox {
    pos: (i32, i32),
    size: (i32, i32),
    field: Rcrc<(String, bool)>,
}

impl TextBox {
    const HEIGHT: i32 = grid(1);
    pub fn create(pos: (i32, i32), width: i32) -> Self {
        Self {
            pos,
            size: (width, Self::HEIGHT),
            field: rcrc(("".to_owned(), false)),
        }
    }

    pub fn respond_to_mouse_press(
        &mut self,
        mouse_pos: (i32, i32),
        mods: &MouseMods,
    ) -> MouseAction {
        MouseAction::FocusTextField(Rc::clone(&self.field))
    }

    pub fn draw(&self, g: &mut GrahpicsWrapper) {
        const GP: i32 = GRID_P;
        let field = self.field.borrow();
        let text = &field.0;
        let focused = field.1;
        g.push_state();
        g.apply_offset(self.pos.0, self.pos.1);

        g.set_color(if focused { &COLOR_IO_AREA } else { &COLOR_BG });
        g.fill_rounded_rect(0, 0, self.size.0, self.size.1, CORNER_SIZE);
        g.set_color(&COLOR_TEXT);
        const H: HAlign = HAlign::Left;
        const V: VAlign = VAlign::Center;
        let w = self.size.0 - GP * 2;
        g.write_text(FONT_SIZE, GP, 0, w, self.size.1, H, V, 1, text);

        g.pop_state();
    }
}

pub struct PatchBrowser {
    pos: (i32, i32),
    size: (i32, i32),
    name_box: TextBox,
}

impl PatchBrowser {
    pub fn create(registry: &Registry, pos: (i32, i32), size: (i32, i32)) -> Self {
        Self {
            pos,
            size,
            name_box: TextBox::create((coord(0), coord(0)), grid(8)),
        }
    }

    pub fn get_tooltip_at(&self, mouse_pos: (i32, i32)) -> Option<Tooltip> {
        None
    }

    pub fn respond_to_mouse_press(
        &mut self,
        mouse_pos: (i32, i32),
        mods: &MouseMods,
    ) -> MouseAction {
        let mouse_pos = mouse_pos.sub(self.pos);
        {
            let mouse_pos = mouse_pos.sub(self.name_box.pos);
            if mouse_pos.inside(self.name_box.size) {
                return self.name_box.respond_to_mouse_press(mouse_pos, mods);
            }
        }
        MouseAction::SavePatch
    }

    pub fn draw(&self, g: &mut GrahpicsWrapper) {
        g.push_state();
        g.apply_offset(self.pos.0, self.pos.1);

        g.set_color(&COLOR_SURFACE);
        g.fill_rect(0, 0, self.size.0, self.size.1);
        self.name_box.draw(g);

        g.pop_state();
    }
}

struct ModuleBrowserEntry {
    name: String,
    category: String,
    input_icons: Vec<usize>,
    output_icons: Vec<usize>,
    prototype: ep::Module,
}

impl ModuleBrowserEntry {
    const WIDTH: i32 = fatgrid(6);
    const HEIGHT: i32 = fatgrid(1);

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
        const CS: i32 = CORNER_SIZE;
        const BAND_SIZE: i32 = GRID_P;
        const ICON_SPACE: i32 = fatgrid(1) / 2;
        const ICON_PADDING: i32 = 2;
        const ICON_SIZE: i32 = (ICON_SPACE * 2 - ICON_PADDING * 4) / 2;

        let num_ports = self.input_icons.len().max(self.output_icons.len()) as i32;
        let port_space = ICON_PADDING + (ICON_PADDING + ICON_SIZE) * num_ports;
        g.set_color(&COLOR_SURFACE);
        let main_width = Self::WIDTH - port_space - BAND_SIZE;
        g.fill_rounded_rect(0, 0, main_width + BAND_SIZE, Self::HEIGHT, CS);
        g.set_color(&COLOR_TEXT);
        g.fill_rounded_rect(main_width, 0, port_space + BAND_SIZE, Self::HEIGHT, CS);
        g.set_color(&COLOR_IO_AREA);
        g.fill_rect(main_width, 0, BAND_SIZE, Self::HEIGHT);
        g.fill_rect(main_width + BAND_SIZE, Self::HEIGHT / 2, port_space, 1);

        g.set_color(&COLOR_TEXT);
        g.write_text(
            FONT_SIZE,
            GRID_P,
            0,
            main_width - GRID_P / 2,
            Self::HEIGHT,
            HAlign::Left,
            VAlign::Center,
            1,
            &self.name,
        );
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

enum VisualEntry {
    RealEntry(usize),
    Label(String),
}

enum SortMethod {
    Alphabetical,
    Categorical,
}

pub struct ModuleBrowser {
    pos: (i32, i32),
    size: (i32, i32),
    vertical_stacking: i32,
    entries: Vec<ModuleBrowserEntry>,
    alphabetical_list: Vec<VisualEntry>,
    categorical_list: Vec<VisualEntry>,
    current_sort: SortMethod,
}

impl ModuleBrowser {
    pub fn create(registry: &Registry, pos: (i32, i32), size: (i32, i32)) -> Self {
        let entries: Vec<_> = registry
            .iterate_over_modules()
            .map(|module| ModuleBrowserEntry::from(module))
            .collect();
        let vertical_stacking = size.1 / (ModuleBrowserEntry::HEIGHT + GRID_P);

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

    fn get_entry_at(&self, mouse_pos: (i32, i32)) -> Option<&ModuleBrowserEntry> {
        let mouse_pos = (mouse_pos.0 - self.pos.0, mouse_pos.1 - self.pos.1);
        let clicked_index = mouse_pos.0 / (ModuleBrowserEntry::WIDTH + GRID_P)
            * self.vertical_stacking
            + mouse_pos.1 / (ModuleBrowserEntry::HEIGHT + GRID_P);
        let clicked_index = clicked_index as usize;
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

    pub fn get_tooltip_at(&self, mouse_pos: (i32, i32)) -> Option<Tooltip> {
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
        mouse_pos: (i32, i32),
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
            let index = index as i32;
            let (x, y) = (
                (index / self.vertical_stacking) * (ModuleBrowserEntry::WIDTH + GRID_P) + GRID_P,
                (index % self.vertical_stacking) * (ModuleBrowserEntry::HEIGHT + GRID_P) + GRID_P,
            );
            g.push_state();
            g.apply_offset(x, y);
            match entry {
                VisualEntry::RealEntry(index) => self.entries[*index].draw(g),
                VisualEntry::Label(text) => {
                    g.set_color(&COLOR_TEXT);
                    g.write_text(
                        BIG_FONT_SIZE,
                        0,
                        0,
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
