use crate::gui::action::MouseAction;
use crate::gui::constants::*;
use crate::gui::graphics::{GrahpicsWrapper, HAlign, VAlign};
use crate::gui::{GuiScreen, InteractionHint, MouseMods, Status, Tooltip};
use crate::registry::Registry;
use crate::util::*;

pub struct MenuBar {
    screens: Vec<GuiScreen>,
    screen_icons: Vec<usize>,
    lclick: usize,
    rclick: usize,
    scroll: usize,
    drag: usize,
    alt: usize,
    shift: usize,
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
            lclick: registry.lookup_icon("factory:left_click").unwrap(),
            rclick: registry.lookup_icon("factory:right_click").unwrap(),
            scroll: registry.lookup_icon("factory:scroll").unwrap(),
            drag: registry.lookup_icon("factory:move").unwrap(),
            alt: registry.lookup_icon("factory:alt").unwrap(),
            shift: registry.lookup_icon("factory:shift").unwrap(),
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

    fn draw_hint(
        &self,
        hint: InteractionHint,
        g: &mut GrahpicsWrapper,
        rx: f32,
        y: f32,
        icons: &[usize],
    ) -> f32 {
        const HINT_HEIGHT: f32 = grid(1) - 2.0;
        const ICON_PAD: f32 = 1.0;
        const ICON_SIZE: f32 = HINT_HEIGHT - ICON_PAD * 2.0;
        fn hint_width(num_icons: f32) -> f32 {
            (ICON_SIZE + ICON_PAD) * num_icons + ICON_PAD
        }

        let active = self.tooltip.interaction.contains(hint) && !self.status.is_some();

        let w = hint_width(icons.len() as f32);
        if active {
            let x = rx - GRID_P - w;
            g.set_color(&COLOR_TEXT);
            g.fill_rounded_rect(x, y, w, HINT_HEIGHT, CORNER_SIZE);
            for (index, icon) in icons.iter().enumerate() {
                g.draw_icon(
                    *icon,
                    x + ICON_PAD + (ICON_PAD + ICON_SIZE) * index as f32,
                    y + ICON_PAD,
                    ICON_SIZE,
                );
            }
            x
        } else {
            rx
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

        let tooltip_x = coord(self.screens.len() as i32) + GP;
        let tooltip_width = width - tooltip_x;
        if let Some(status) = self.status.as_ref() {
            g.set_color(&status.color);
        } else {
            g.set_color(&COLOR_BG);
        }
        g.fill_rounded_rect(tooltip_x, coord(0), tooltip_width + CS, ITEM_HEIGHT, CS);
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

        g.set_color(&COLOR_BG);
        let mut x = width + GP2;
        let hints = [
            (InteractionHint::Alt, vec![self.alt]),
            (InteractionHint::Shift, vec![self.shift]),
            (
                InteractionHint::LeftClickAndDrag,
                vec![self.lclick, self.drag],
            ),
            (InteractionHint::DoubleClick, vec![self.lclick, self.lclick]),
            (InteractionHint::Scroll, vec![self.scroll]),
            (InteractionHint::RightClick, vec![self.rclick]),
            (InteractionHint::LeftClick, vec![self.lclick]),
        ];
        for (hint, icons) in &hints {
            // No idea why the extra +0.5 is necessary.
            x = self.draw_hint(*hint, g, x, GP * 1.5 + 0.5, icons);
        }

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
