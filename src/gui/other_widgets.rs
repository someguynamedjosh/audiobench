use crate::engine::parts as ep;
use crate::engine::registry::Registry;
use crate::engine::save_data::Patch;
use crate::gui::action::MouseAction;
use crate::gui::constants::*;
use crate::gui::graphics::{GrahpicsWrapper, HAlign, VAlign};
use crate::gui::{GuiScreen, InteractionHint, MouseMods, Status, Tooltip};
use crate::util::*;
use std::collections::HashSet;

pub struct MenuBar {
    screens: Vec<GuiScreen>,
    screen_icons: Vec<usize>,
    lclick: usize,
    rclick: usize,
    drag: usize,
    and: usize,
    tooltip: Tooltip,
    status: Option<Status>,
}

impl MenuBar {
    pub const HEIGHT: f32 = grid(1) + GRID_P * 3.0;

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
            status: None,
        }
    }

    pub fn get_tooltip_at(&self, mouse_pos: (f32, f32)) -> Option<Tooltip> {
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

    pub fn set_status(&mut self, status: Status) {
        self.status = Some(status);
    }

    pub fn clear_status(&mut self) {
        self.status = None;
    }

    pub fn respond_to_mouse_press(&self, mouse_pos: (f32, f32), mods: &MouseMods) -> MouseAction {
        if !mouse_pos.inside((99999.0, Self::HEIGHT)) {
            return MouseAction::None;
        }
        let new_screen = ((mouse_pos.0 - GRID_P) / (GRID_P + grid(1))) as usize;
        if new_screen < self.screens.len() {
            MouseAction::SwitchScreen(self.screens[new_screen])
        } else {
            MouseAction::None
        }
    }

    pub fn draw(&self, width: f32, current_screen: GuiScreen, g: &mut GrahpicsWrapper) {
        let current_screen_index = self
            .screens
            .iter()
            .position(|e| e == &current_screen)
            .unwrap() as i32;

        g.set_color(&COLOR_SURFACE);
        g.fill_rect(0.0, 0.0, width, Self::HEIGHT);

        const GP: f32 = GRID_P;
        const GP2: f32 = GRID_P / 2.0;
        const CS: f32 = CORNER_SIZE;
        const HEIGHT: f32 = MenuBar::HEIGHT;
        const ITEM_HEIGHT: f32 = HEIGHT - GP * 2.0;

        g.set_color(&COLOR_BG);
        g.fill_rounded_rect(
            coord(0),
            coord(0),
            grid(self.screens.len() as i32) + GP,
            ITEM_HEIGHT,
            CS,
        );

        const HINT_HEIGHT: f32 = grid(1) - 2.0;
        const ICON_PAD: f32 = 1.0;
        const ICON_SIZE: f32 = HINT_HEIGHT - ICON_PAD * 2.0;
        const HINT_AREA_WIDTH: f32 = ICON_SIZE * 4.0 + GP * 3.0 + ICON_PAD * 6.0;
        let status = self.status.is_some();
        {
            g.set_color(&COLOR_SURFACE);
            g.fill_rounded_rect(
                width - HINT_AREA_WIDTH,
                0.0,
                HINT_AREA_WIDTH,
                HINT_HEIGHT * 2.0 + GP * 3.0,
                CS,
            );

            const Y1: f32 = GP;
            const Y2: f32 = Y1 + HINT_HEIGHT + GP;
            fn hint_width(num_icons: f32) -> f32 {
                (ICON_SIZE + ICON_PAD) * num_icons + ICON_PAD
            }
            fn draw_hint(
                active: bool,
                g: &mut GrahpicsWrapper,
                rx: f32,
                y: f32,
                icons: &[usize],
            ) -> f32 {
                let w = hint_width(icons.len() as f32);
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
                            x + ICON_PAD + (ICON_PAD + ICON_SIZE) * index as f32,
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
                .contains(InteractionHint::LeftClickAndDrag)
                && !status;
            let x = draw_hint(active, g, width, Y1, &[self.lclick, self.and, self.drag]);
            let active = self
                .tooltip
                .interaction
                .contains(InteractionHint::LeftClick)
                || status;
            draw_hint(active, g, x, Y1, &[self.lclick]);

            let active = self
                .tooltip
                .interaction
                .contains(InteractionHint::DoubleClick)
                && !status;
            let x = draw_hint(active, g, width, Y2, &[self.lclick, self.and, self.lclick]);
            let active = self
                .tooltip
                .interaction
                .contains(InteractionHint::RightClick)
                && !status;
            draw_hint(active, g, x, Y2, &[self.rclick]);
        }

        let tooltip_x = coord(self.screens.len() as i32) + GP;
        let tooltip_width = width - HINT_AREA_WIDTH - tooltip_x;
        if let Some(status) = self.status.as_ref() {
            g.set_color(&status.color);
        } else {
            g.set_color(&COLOR_BG);
        }
        g.fill_rounded_rect(tooltip_x, coord(0), tooltip_width, ITEM_HEIGHT, CS);
        let text_x = tooltip_x + GP;
        let text_width = tooltip_width - GP * 2.0;
        g.set_color(&COLOR_TEXT);
        let text = if let Some(status) = self.status.as_ref() {
            &status.text
        } else {
            &self.tooltip.text
        };
        g.write_text(
            FONT_SIZE,
            text_x,
            coord(0) + GP2,
            text_width,
            grid(1),
            HAlign::Left,
            VAlign::Center,
            1,
            text,
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

pub struct TextField {
    pub text: String,
    focused: bool,
    defocus_action: fn(&str) -> MouseAction,
}

impl TextField {
    fn new(start_value: String, defocus_action: fn(&str) -> MouseAction) -> Self {
        Self {
            text: start_value,
            focused: false,
            defocus_action,
        }
    }

    pub fn focus(&mut self) {
        debug_assert!(!self.focused);
        self.focused = true;
    }

    pub fn defocus(&mut self) -> MouseAction {
        debug_assert!(self.focused);
        self.focused = false;
        (self.defocus_action)(&self.text)
    }
}

pub struct TextBox {
    pos: (f32, f32),
    size: (f32, f32),
    field: Rcrc<TextField>,
}

impl TextBox {
    pub fn create(
        pos: (f32, f32),
        size: (f32, f32),
        start_value: String,
        defocus_action: fn(&str) -> MouseAction,
    ) -> Self {
        Self {
            pos,
            size,
            field: rcrc(TextField::new(start_value, defocus_action)),
        }
    }

    pub fn respond_to_mouse_press(
        &mut self,
        mouse_pos: (f32, f32),
        mods: &MouseMods,
    ) -> MouseAction {
        MouseAction::FocusTextField(Rc::clone(&self.field))
    }

    pub fn draw(&self, g: &mut GrahpicsWrapper) {
        const GP: f32 = GRID_P;
        let field = self.field.borrow();
        let text = &field.text;
        let focused = field.focused;
        g.push_state();
        g.apply_offset(self.pos.0, self.pos.1);

        g.set_color(if focused { &COLOR_IO_AREA } else { &COLOR_BG });
        g.fill_rounded_rect(0.0, 0.0, self.size.0, self.size.1, CORNER_SIZE);
        g.set_color(&COLOR_TEXT);
        const H: HAlign = HAlign::Left;
        const V: VAlign = VAlign::Center;
        let w = self.size.0 - GP * 2.0;
        g.write_text(FONT_SIZE, GP, 0.0, w, self.size.1, H, V, 1, text);

        g.pop_state();
    }
}

pub struct IconButton {
    pos: (f32, f32),
    size: f32,
    icon: usize,
    enabled: bool,
}

impl IconButton {
    pub fn create(pos: (f32, f32), size: f32, icon: usize) -> Self {
        Self {
            pos,
            size,
            icon,
            enabled: true,
        }
    }

    pub fn mouse_in_bounds(&self, mouse_pos: (f32, f32)) -> bool {
        self.enabled && mouse_pos.sub(self.pos).inside((self.size, self.size))
    }

    pub fn draw(&self, g: &mut GrahpicsWrapper) {
        g.push_state();
        g.apply_offset(self.pos.0, self.pos.1);

        g.set_color(&COLOR_BG);
        g.fill_rounded_rect(0.0, 0.0, self.size, self.size, CORNER_SIZE);
        const IP: f32 = GRID_P / 2.0;
        g.draw_white_icon(self.icon, IP, IP, self.size - IP * 2.0);
        if !self.enabled {
            g.set_color(&COLOR_BG);
            g.set_alpha(0.5);
            g.fill_rounded_rect(0.0, 0.0, self.size, self.size, CORNER_SIZE);
        }

        g.pop_state();
    }
}

pub struct PatchBrowser {
    pos: (f32, f32),
    size: (f32, f32),
    name_box: TextBox,
    save_button: IconButton,
    new_button: IconButton,
    entries: Rcrc<Vec<Rcrc<Patch>>>,
    current_entry_index: usize,
}

impl PatchBrowser {
    pub fn create(
        current_patch: &Rcrc<Patch>,
        registry: &Registry,
        pos: (f32, f32),
        size: (f32, f32),
    ) -> Self {
        // How large each half of the GUI takes.
        let hw = (size.0 - GRID_P * 3.0) / 2.0;
        const CG: f32 = PatchBrowser::CG;
        // How many icon buttons to the right of the name box.
        const NUM_ICONS: f32 = 2.0;
        // Width of the name box.
        let namew = hw - (CG + GRID_P) * NUM_ICONS;
        let patch_name = current_patch.borrow().borrow_name().to_owned();
        let name_box = TextBox::create((GRID_P, 0.0), (namew, CG), patch_name, |text| {
            MouseAction::RenamePatch(text.to_owned())
        });
        let save_icon = registry.lookup_icon("base:save").unwrap();
        let mut save_button = IconButton::create((GRID_P + hw - CG * 2.0 - GRID_P, 0.0), CG, save_icon);
        let new_icon = registry.lookup_icon("base:add").unwrap();
        let new_button = IconButton::create((GRID_P + hw - CG, 0.0), CG, new_icon);

        let entries = registry.borrow_patches().clone();
        let current_entry_index = registry
            .borrow_patches()
            .iter()
            .position(|patch| std::ptr::eq(patch.as_ref(), current_patch.as_ref()))
            .unwrap();
        if !entries[current_entry_index].borrow().is_writable() {
            save_button.enabled = false;
        }

        Self {
            pos,
            size,
            name_box,
            save_button,
            new_button,
            entries: rcrc(entries),
            current_entry_index,
        }
    }

    fn update_on_patch_change(&mut self) {
        let entries_ref = self.entries.borrow();
        let entry_ref = entries_ref[self.current_entry_index].borrow();
        self.name_box.field.borrow_mut().text = entry_ref.borrow_name().to_owned();
        self.save_button.enabled = entry_ref.is_writable();
    }

    pub fn get_tooltip_at(&self, mouse_pos: (f32, f32)) -> Option<Tooltip> {
        let mouse_pos = mouse_pos.sub(self.pos);
        {
            let mouse_pos = mouse_pos.sub(self.name_box.pos);
            if mouse_pos.inside(self.name_box.size) {
                return Some(if self.save_button.enabled {
                    Tooltip {
                        text: "Edit the name of the current patch".to_owned(),
                        interaction: InteractionHint::LeftClick.into(),
                    }
                } else {
                    Tooltip {
                        text: "The current patch is a factory patch, so you cannot edit its name"
                            .to_owned(),
                        interaction: Default::default(),
                    }
                });
            }
        }
        if self.save_button.mouse_in_bounds(mouse_pos) {
            return Some(Tooltip {
                text: "Save the current patch".to_owned(),
                interaction: InteractionHint::LeftClick.into(),
            });
        }
        if self.new_button.mouse_in_bounds(mouse_pos) {
            return Some(Tooltip {
                text: "Create a new patch containing the current settings and note graph"
                    .to_owned(),
                interaction: InteractionHint::LeftClick.into(),
            });
        }
        let hw = (self.size.0 - GRID_P * 3.0) / 2.0;
        if mouse_pos.0 <= hw && mouse_pos.1 > self.name_box.size.1 + GRID_P {
            return Some(Tooltip {
                text: "Click a patch to load it".to_owned(),
                interaction: InteractionHint::LeftClick.into(),
            });
        }
        None
    }

    pub fn respond_to_mouse_press(
        &mut self,
        mouse_pos: (f32, f32),
        mods: &MouseMods,
    ) -> MouseAction {
        let mouse_pos = mouse_pos.sub(self.pos);
        // Only enabled if we can modify the current patch.
        if self.save_button.enabled {
            let mouse_pos = mouse_pos.sub(self.name_box.pos);
            if mouse_pos.inside(self.name_box.size) {
                return self.name_box.respond_to_mouse_press(mouse_pos, mods);
            }
        }
        if self.save_button.mouse_in_bounds(mouse_pos) {
            return MouseAction::SavePatch;
        }
        if self.new_button.mouse_in_bounds(mouse_pos) {
            self.current_entry_index = self.entries.borrow().len();
            let entries = Rc::clone(&self.entries);
            self.save_button.enabled = true;
            self.name_box.field.borrow_mut().text = "New Patch".to_owned();
            return MouseAction::NewPatch(Box::new(move |new_patch| {
                entries.borrow_mut().push(Rc::clone(new_patch))
            }));
        }
        // How large each half of the GUI takes.
        let hw = (self.size.0 - GRID_P * 3.0) / 2.0;
        if mouse_pos.0 <= hw && mouse_pos.1 > self.name_box.size.1 + GRID_P {
            let entry_index =
                (mouse_pos.1 - self.name_box.size.1 - GRID_P) / PatchBrowser::ENTRY_HEIGHT;
            if entry_index >= 0.0 && entry_index < self.entries.borrow().len() as f32 {
                self.current_entry_index = entry_index as usize;
                self.update_on_patch_change();
                return MouseAction::LoadPatch(Rc::clone(
                    &self.entries.borrow()[self.current_entry_index],
                ));
            }
        }
        MouseAction::None
    }

    // A slightly larger grid size.
    const CG: f32 = grid(1) + GRID_P;
    const ENTRY_HEIGHT: f32 = Self::CG;

    pub fn draw(&self, g: &mut GrahpicsWrapper) {
        g.push_state();
        g.apply_offset(self.pos.0, self.pos.1);

        // How large each half of the GUI takes.
        let hw = (self.size.0 - GRID_P * 3.0) / 2.0;
        const GP: f32 = GRID_P;

        g.set_color(&COLOR_SURFACE);
        g.fill_rect(0.0, 0.0, self.size.0, self.size.1);
        self.name_box.draw(g);
        self.save_button.draw(g);
        self.new_button.draw(g);

        const CG: f32 = PatchBrowser::CG;
        let y = CG + GP;
        g.set_color(&COLOR_BG);
        g.fill_rounded_rect(GP, y, hw, self.size.1 - y - GP, CORNER_SIZE);
        g.set_color(&COLOR_TEXT);
        for (index, entry) in self.entries.borrow().iter().enumerate() {
            const HEIGHT: f32 = PatchBrowser::ENTRY_HEIGHT;
            let x = GP;
            let y = y + HEIGHT * index as f32;
            if index == self.current_entry_index {
                g.set_color(&COLOR_IO_AREA);
                g.fill_rounded_rect(x, y, hw, HEIGHT, CORNER_SIZE);
                g.set_color(&COLOR_TEXT);
            }
            let entry = entry.borrow();
            const H: HAlign = HAlign::Left;
            const V: VAlign = VAlign::Center;
            let name = entry.borrow_name();
            g.write_text(FONT_SIZE, x + GP, y, hw - GP * 2.0, HEIGHT, H, V, 1, name);
            if !entry.is_writable() {
                const H: HAlign = HAlign::Right;
                g.set_alpha(0.5);
                let t = "[Factory]";
                g.write_text(FONT_SIZE, x + GP, y, hw - GP * 2.0, HEIGHT, H, V, 1, t);
                g.set_alpha(1.0);
            }
        }

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
    vertical_stacking: f32,
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

    fn get_entry_at(&self, mouse_pos: (f32, f32)) -> Option<&ModuleBrowserEntry> {
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
            let index = index as f32;
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
