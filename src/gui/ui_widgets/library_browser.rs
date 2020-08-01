use crate::gui::action::{GuiAction, MouseAction};
use crate::gui::constants::*;
use crate::gui::graphics::{GrahpicsWrapper, HAlign, VAlign};
use crate::gui::{InteractionHint, MouseMods, Tooltip};
use crate::registry::Registry;
use crate::util::*;

struct LibraryEntry {
    name: String,
    version: u16,
    description: String,
}

enum UpdateInfo<'a> {
    /// Either the update info is still loading or there is no update info for the particular lib.
    None,
    NewUpdate {
        header: String,
        changes: &'a [String],
        download_url: &'a str,
    },
    FailedToRetrieve,
}

#[derive(Debug)]
enum LibraryBrowserMouseAction {
    None,
    OpenWebpage(String),
}

pub struct LibraryBrowser {
    pos: (f32, f32),
    size: (f32, f32),
    scroll_offset: f32,
    max_scroll_offset: f32,
    libraries: Vec<LibraryEntry>,
    mouse_actions: Vec<(f32, LibraryBrowserMouseAction)>,
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
            scroll_offset: 0.0,
            max_scroll_offset: 0.0,
            libraries,
            mouse_actions: Vec::new(),
        }
    }

    pub fn get_tooltip_at(&self, mouse_pos: (f32, f32)) -> Option<Tooltip> {
        let mouse_pos = mouse_pos.sub(self.pos);
        for (end_y, action) in self.mouse_actions.iter() {
            if mouse_pos.1 + self.scroll_offset <= *end_y {
                return match action {
                    LibraryBrowserMouseAction::None => Some(Tooltip {
                        text: "".to_owned(),
                        interaction: InteractionHint::Scroll.into(),
                    }),
                    LibraryBrowserMouseAction::OpenWebpage(..) => Some(Tooltip {
                        text: "".to_owned(),
                        interaction: InteractionHint::Scroll | InteractionHint::LeftClick,
                    }),
                };
            }
        }
        Some(Tooltip {
            text: "".to_owned(),
            interaction: InteractionHint::Scroll.into(),
        })
    }

    pub fn respond_to_mouse_press(
        &mut self,
        mouse_pos: (f32, f32),
        _mods: &MouseMods,
    ) -> MouseAction {
        let mouse_pos = mouse_pos.sub(self.pos);
        for (end_y, action) in self.mouse_actions.iter() {
            if mouse_pos.1 + self.scroll_offset <= *end_y {
                return match action {
                    LibraryBrowserMouseAction::None => MouseAction::None,
                    LibraryBrowserMouseAction::OpenWebpage(url) => {
                        MouseAction::OpenWebpage(url.clone())
                    }
                };
            }
        }
        MouseAction::None
    }

    pub fn on_scroll(&mut self, delta: f32) -> Option<GuiAction> {
        self.scroll_offset = (self.scroll_offset - delta * 100.0).clam(0.0, self.max_scroll_offset);
        None
    }

    fn draw_library_entry(
        &self,
        g: &mut GrahpicsWrapper,
        y: f32,
        name: &str,
        version: u16,
        description: &str,
        update: UpdateInfo,
    ) -> (f32, Vec<(f32, LibraryBrowserMouseAction)>) {
        const GP: f32 = GRID_P;
        const CS: f32 = CORNER_SIZE;
        let x = GP;
        let width = self.size.0 - GP * 2.0;
        const FS: f32 = BIG_FONT_SIZE;
        const TFS: f32 = TITLE_FONT_SIZE;
        const L: HAlign = HAlign::Left;
        const R: HAlign = HAlign::Right;
        const TOP: VAlign = VAlign::Top;
        let tw = width - GP * 2.0;

        g.set_color(&COLOR_SURFACE);
        g.fill_rounded_rect(x, y, width, Self::ENTRY_HEIGHT, CS);
        g.set_color(&COLOR_IO_AREA);
        g.fill_rounded_rect(x, y, width, fatgrid(1), CS);
        let mut total_height = Self::ENTRY_HEIGHT;
        let mut actions = Vec::new();
        actions.push((y + total_height, LibraryBrowserMouseAction::None));

        if let UpdateInfo::FailedToRetrieve = update {
            g.set_color(&COLOR_ERROR);
            let topy = y;
            let y = y + Self::ENTRY_HEIGHT - CS;
            g.fill_rounded_rect(x, y, width, FS + CS + GP * 2.0, CS);
            // This gets rid of the bottom corners on the main text box.
            g.set_color(&COLOR_SURFACE);
            g.fill_rect(x, y, width, CS);
            g.set_color(&COLOR_TEXT);
            let ty = y + CS + GRID_P;
            let text = "Failed to check for updates, see console for details.";
            g.write_text(FS, x + GRID_P, ty, tw, FS, L, TOP, 1, text);

            total_height += FS + GRID_P * 2.0;
            actions.push((topy + total_height, LibraryBrowserMouseAction::None));
        } else if let UpdateInfo::NewUpdate {
            header,
            changes,
            download_url,
        } = update
        {
            g.set_color(&COLOR_SUCCESS);
            let topy = y;
            let y = y + Self::ENTRY_HEIGHT - CS;
            let height = GP + (FS + GP) * (changes.len() + 1) as f32;
            g.fill_rounded_rect(x, y, width, height + CS, CS);
            g.set_color(&COLOR_SURFACE);
            g.fill_rect(x, y, width, CS);
            g.set_color(&COLOR_TEXT);
            let mut ty = y + CS + GRID_P;
            g.write_text(FS, x + GRID_P, ty, tw, FS, L, TOP, 1, &header);
            for change in changes {
                ty += FS + GRID_P;
                g.write_text(FS, x + GRID_P, ty, tw, FS, L, TOP, 1, change);
            }

            total_height += height;
            actions.push((
                topy + total_height,
                LibraryBrowserMouseAction::OpenWebpage(download_url.to_owned()),
            ));
        }

        g.set_color(&COLOR_TEXT);
        g.write_text(TFS, x + GP, y + GP, tw, grid(1), L, TOP, 1, name);
        let version = format!("v{}", version);
        g.write_text(TFS, x + GP, y + GP, tw, grid(1), R, TOP, 1, &version);
        let ty = y + fatgrid(1) + GP;
        // TODO: Text wrapping is broke af here. Bug has been reported at:
        // https://github.com/juce-framework/JUCE/issues/768
        g.write_text(FS, x + GP, ty, tw, grid(3), L, TOP, 5, description);

        (total_height + GRID_P, actions)
    }

    // Updates are checked asynchronously so we have to read them from the registry.
    pub fn draw(&mut self, g: &mut GrahpicsWrapper, registry: &Registry) {
        g.push_state();
        g.apply_offset(self.pos.0, self.pos.1 - self.scroll_offset);

        g.set_color(&COLOR_BG);
        g.fill_rect(0.0, 0.0, self.size.0, self.size.1);

        self.mouse_actions.clear();
        let mut y = GRID_P;
        let engine_update = match registry.borrow_update_info(ENGINE_UPDATE_URL) {
            None => UpdateInfo::None,
            Some(None) => UpdateInfo::FailedToRetrieve,
            Some(Some(info)) => {
                if info.version > ENGINE_VERSION {
                    UpdateInfo::NewUpdate {
                        header: format!(
                            concat!(
                                "Version {} is now available! Click to open the download page. ",
                                "Changes include:"
                            ),
                            info.version
                        ),
                        changes: &info.changes[..],
                        download_url: &info.download_url,
                    }
                } else {
                    UpdateInfo::None
                }
            }
        };
        let (height, mut new_actions) = self.draw_library_entry(
            g,
            y,
            "Audiobench Engine",
            ENGINE_VERSION,
            ENGINE_INFO,
            engine_update,
        );
        y += height;
        self.mouse_actions.append(&mut new_actions);
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
        self.mouse_actions
            .push((y, LibraryBrowserMouseAction::None));
        for library in &self.libraries {
            let (height, mut new_actions) = self.draw_library_entry(
                g,
                y,
                &library.name,
                library.version,
                &library.description,
                UpdateInfo::None,
            );
            y += height;
            self.mouse_actions.append(&mut new_actions);
        }
        self.max_scroll_offset = (y - self.size.1).max(0.0);

        g.pop_state();
    }
}
