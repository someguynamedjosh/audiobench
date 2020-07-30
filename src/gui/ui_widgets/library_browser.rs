use super::{IconButton, TextBox, TextField};
use crate::gui::action::{GuiAction, MouseAction};
use crate::gui::constants::*;
use crate::gui::graphics::{GrahpicsWrapper, HAlign, VAlign};
use crate::gui::{InteractionHint, MouseMods, Tooltip};
use crate::registry::save_data::Patch;
use crate::registry::Registry;
use crate::util::*;

struct LibraryEntry {
    name: String,
    version: u16,
    description: String,
}

pub struct LibraryBrowser {
    pos: (f32, f32),
    size: (f32, f32),
    libraries: Vec<LibraryEntry>,
}

impl LibraryBrowser {
    const ENTRY_HEIGHT: f32 = fatgrid(1) + fatgrid(3);

    pub fn create(registry: &Registry, pos: (f32, f32), size: (f32, f32)) -> Self {
        let mut libraries: Vec<_> = registry
            .borrow_library_infos()
            .map(|info| LibraryEntry {
                name: info.pretty_name.clone(),
                description: info.description.clone(),
                version: info.version,
            })
            .collect();
        libraries.sort_by(|a, b| a.name.cmp(&b.name));
        Self {
            pos,
            size,
            libraries,
        }
    }

    pub fn on_scroll(&mut self, _delta: f32) -> Option<GuiAction> {
        None
    }

    fn draw_library_entry(
        &self,
        g: &mut GrahpicsWrapper,
        y: f32,
        name: &str,
        version: u16,
        description: &str,
    ) {
        const GP: f32 = GRID_P;
        const CS: f32 = CORNER_SIZE;
        let x = GP;
        g.set_color(&COLOR_SURFACE);
        g.fill_rounded_rect(x, y, self.size.0 - GP * 2.0, Self::ENTRY_HEIGHT, CS);
        g.set_color(&COLOR_IO_AREA);
        g.fill_rounded_rect(x, y, self.size.0 - GP * 2.0, fatgrid(1), CS);

        g.set_color(&COLOR_TEXT);
        const FS: f32 = BIG_FONT_SIZE;
        const TFS: f32 = TITLE_FONT_SIZE;
        const L: HAlign = HAlign::Left;
        const R: HAlign = HAlign::Right;
        const TOP: VAlign = VAlign::Top;
        let tw = self.size.0 - GP * 4.0;
        g.write_text(TFS, x + GP, y + GP, tw, grid(1), L, TOP, 1, name);
        let version = format!("v{}", version);
        g.write_text(TFS, x + GP, y + GP, tw, grid(1), R, TOP, 1, &version);
        let ty = y + fatgrid(1) + GP;
        // TODO: Text wrapping is broke af here. Bug has been reported at:
        // https://github.com/juce-framework/JUCE/issues/768
        g.write_text(FS, x + GP, ty, tw, grid(3), L, TOP, 5, description);
    }

    pub fn draw(&self, g: &mut GrahpicsWrapper) {
        g.push_state();
        g.apply_offset(self.pos.0, self.pos.1);

        g.set_color(&COLOR_BG);
        g.fill_rect(0.0, 0.0, self.size.0, self.size.1);

        let mut y = GRID_P;
        self.draw_library_entry(g, y, "Audiobench Engine", ENGINE_VERSION, ENGINE_INFO);
        y += Self::ENTRY_HEIGHT + GRID_P;
        g.write_text(
            TITLE_FONT_SIZE,
            0.0,
            y,
            self.size.0,
            grid(1),
            HAlign::Center,
            VAlign::Center,
            1,
            "Installed Libraries",
        );
        y += TITLE_FONT_SIZE + GRID_P;
        for library in &self.libraries {
            self.draw_library_entry(g, y, &library.name, library.version, &library.description);
            y += Self::ENTRY_HEIGHT + GRID_P;
        }

        g.pop_state();
    }
}
