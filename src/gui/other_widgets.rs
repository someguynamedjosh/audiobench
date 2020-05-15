use crate::engine::registry::Registry;
use crate::gui::constants::*;
use crate::gui::graphics::GrahpicsWrapper;
use crate::gui::{MouseAction, MouseMods};

fn bound_check(coord: (i32, i32), bounds: (i32, i32)) -> bool {
    coord.0 >= 0 && coord.1 >= 0 && coord.0 <= bounds.0 && coord.1 <= bounds.1
}

pub struct MenuBar {
    module_graph_icon: usize,
    module_library_icon: usize,
}

impl MenuBar {
    pub const HEIGHT: i32 = FATGRID_1;

    pub fn create(registry: &Registry) -> Self {
        Self {
            module_graph_icon: registry.lookup_icon("base:module").unwrap(),
            module_library_icon: registry.lookup_icon("base:add").unwrap(),
        }
    }

    pub fn respond_to_mouse_press(&self, mouse_pos: (i32, i32), mods: &MouseMods) -> MouseAction {
        if !bound_check(mouse_pos, (99999, Self::HEIGHT)) {
            return MouseAction::None;
        }
        MouseAction::None
    }

    pub fn draw(&self, width: i32, current_screen_index: i32, g: &mut GrahpicsWrapper) {
        g.set_color(&COLOR_SURFACE);
        g.fill_rect(0, 0, width, Self::HEIGHT);

        const GP: i32 = GRID_P;
        const GP2: i32 = GRID_P / 2;
        const MCS: i32 = MODULE_CORNER_SIZE;

        fn draw_button(g: &mut GrahpicsWrapper, selected_index: i32, index: i32, icon: usize) {
            if selected_index == index {
                g.set_color(&COLOR_TEXT);
                g.fill_rounded_rect(
                    coord(index) - GP2,
                    coord(0) - GP2,
                    grid(1) + GP,
                    grid(1) + GP,
                    MCS,
                );
            }
            g.draw_icon(icon, coord(index), coord(0), grid(1));
        }

        g.set_color(&COLOR_IO_AREA);
        g.fill_rounded_rect(
            coord(0) - GP2,
            coord(0) - GP2,
            grid(3) + GP,
            grid(1) + GP,
            MCS,
        );

        g.fill_rounded_rect(coord(3), coord(0) - GP2, width, grid(1) + GP, MCS);

        draw_button(g, current_screen_index, 0, self.module_graph_icon);
        draw_button(g, current_screen_index, 1, self.module_library_icon);
        draw_button(g, current_screen_index, 2, self.module_library_icon);
    }
}
