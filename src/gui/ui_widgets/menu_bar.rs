use crate::engine::registry::Registry;
use crate::gui::action::MouseAction;
use crate::gui::constants::*;
use crate::gui::graphics::{GrahpicsWrapper, HAlign, VAlign};
use crate::gui::{GuiScreen, InteractionHint, MouseMods, Status, Tooltip};
use crate::util::*;

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