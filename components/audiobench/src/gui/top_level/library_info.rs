use crate::{config::{ENGINE_INFO, ENGINE_UPDATE_URL, ENGINE_VERSION}, engine::controls::AnyControl, gui::{GuiTab, InteractionHint, TabArchetype, Tooltip, constants::*, graphics::GrahpicsWrapper}, registry::{module_template::ModuleTemplate, Registry}, scui_config::{DropTarget, MaybeMouseBehavior, Renderer}};
use scui::{MouseMods, OnClickBehavior, Vec2D, WidgetImpl};
use shared_util::prelude::*;

struct LibraryEntry {
    name: String,
    version: Version,
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
enum LibraryInfoMouseAction {
    None,
    OpenWebpage(String),
}

scui::widget! {
    pub LibraryInfo
    State {
        scroll_offset: f32,
        max_scroll_offset: f32,
        libraries: Vec<LibraryEntry>,
        mouse_actions: Vec<(f32, LibraryInfoMouseAction)>,
    }
}

const ENTRY_HEIGHT: f32 = fatgrid(1) + fatgrid(3);

impl LibraryInfo {
    pub fn new(parent: &impl LibraryInfoParent) -> Rc<Self> {
        let inter = parent.provide_gui_interface();
        let state = inter.state.borrow();
        let registry = state.registry.borrow();

        let mut libraries: Vec<_> = registry
            .borrow_library_infos()
            .map(|(name, info)| LibraryEntry {
                name: info.pretty_name.clone(),
                description: info.description.clone(),
                version: info.version,
            })
            .collect();
        libraries.sort_by(|a, b| a.name.cmp(&b.name));
        let state = LibraryInfoState {
            scroll_offset: 0.0,
            max_scroll_offset: 0.0,
            libraries,
            mouse_actions: Vec::new(),
        };
        Rc::new(Self::create(parent, state))
    }

    fn draw_library_entry(
        &self,
        g: &mut Renderer,
        y: f32,
        name: &str,
        version: Version,
        description: &str,
        update: UpdateInfo,
    ) -> (f32, Vec<(f32, LibraryInfoMouseAction)>) {
        const GP: f32 = GRID_P;
        const CS: f32 = CORNER_SIZE;
        let x = GP;
        let width = TAB_BODY_WIDTH - GP * 2.0;
        const FS: f32 = BIG_FONT_SIZE;
        const TFS: f32 = TITLE_FONT_SIZE;
        let tw = width - GP * 2.0;

        g.set_color(&COLOR_BG2);
        g.draw_rounded_rect((x, y), (width, ENTRY_HEIGHT), CS);
        g.set_color(&COLOR_BG1);
        g.draw_rounded_rect((x, y), (width, fatgrid(1)), CS);
        let mut total_height = ENTRY_HEIGHT;
        let mut actions = Vec::new();
        actions.push((y + total_height, LibraryInfoMouseAction::None));

        if let UpdateInfo::FailedToRetrieve = update {
            g.set_color(&COLOR_ERROR);
            let topy = y;
            let y = y + ENTRY_HEIGHT - CS;
            g.draw_rounded_rect((x, y), (width, FS + CS + GP * 2.0), CS);
            // This gets rid of the bottom corners on the main text box.
            g.set_color(&COLOR_BG2);
            g.draw_rect((x, y), (width, CS));
            g.set_color(&COLOR_FG1);
            let ty = y + CS + GRID_P;
            let text = "Failed to check for updates, see console for details.";
            g.draw_text(FS, (x + GRID_P, ty), (tw, FS), (-1, -1), 1, text);

            total_height += FS + GRID_P * 2.0;
            actions.push((topy + total_height, LibraryInfoMouseAction::None));
        } else if let UpdateInfo::NewUpdate {
            header,
            changes,
            download_url,
        } = update
        {
            g.set_color(&COLOR_SUCCESS);
            let topy = y;
            let y = y + ENTRY_HEIGHT - CS;
            let height = GP + (FS + GP / 2.0) * (changes.len() + 1) as f32;
            g.draw_rounded_rect((x, y), (width, height + CS), CS);
            g.set_color(&COLOR_BG2);
            g.draw_rect((x, y), (width, CS));
            g.set_color(&COLOR_FG1);
            let mut ty = y + CS + GRID_P;
            g.draw_text(FS, (x + GRID_P, ty), (tw, FS), (-1, -1), 1, &header);
            // TODO: Nicer formatting?
            for change in changes {
                ty += FS + GRID_P / 2.0;
                let text = format!("- {}", change);
                g.draw_text(FS, (x + GRID_P, ty), (tw, FS), (-1, -1), 1, &text);
            }

            total_height += height;
            actions.push((
                topy + total_height,
                LibraryInfoMouseAction::OpenWebpage(download_url.to_owned()),
            ));
        }

        g.set_color(&COLOR_FG1);
        g.draw_text(TFS, (x + GP, y + GP), (tw, grid(1)), (-1, -1), 1, name);
        let version = format!("v{}", version);
        g.draw_text(TFS, (x + GP, y + GP), (tw, grid(1)), (1, -1), 1, &version);
        let ty = y + fatgrid(1) + GP;
        // TODO: Text wrapping is broke af here. Bug has been reported at:
        // https://github.com/juce-framework/JUCE/issues/768
        g.draw_text(FS, (x + GP, ty), (tw, grid(3)), (-1, -1), 5, description);

        (total_height + GRID_P, actions)
    }
}

fn open_browser(this: Rc<LibraryInfo>, url: String) {
    if let Err(err) = webbrowser::open(&url) {
        this.with_gui_state_mut(|state| {
            state.add_error_status(format!(
                "Failed to open web browser, see console for details."
            ));
            eprintln!("WARNING: Failed to open web browser, caused by:\n{}", err);
        })
    }
}

impl WidgetImpl<Renderer, DropTarget> for LibraryInfo {
    fn get_pos_impl(self: &Rc<Self>) -> Vec2D {
        (0.0, HEADER_HEIGHT).into()
    }

    fn get_size_impl(self: &Rc<Self>) -> Vec2D {
        TAB_BODY_SIZE
    }

    fn get_mouse_behavior_impl(
        self: &Rc<Self>,
        mouse_pos: Vec2D,
        _mods: &MouseMods,
    ) -> MaybeMouseBehavior {
        let state = self.state.borrow();
        for (end_y, action) in state.mouse_actions.iter() {
            if mouse_pos.y + state.scroll_offset <= *end_y {
                return match action {
                    LibraryInfoMouseAction::None => None,
                    LibraryInfoMouseAction::OpenWebpage(url) => {
                        let this = Rc::clone(self);
                        let url = url.clone();
                        println!("{}", url);
                        OnClickBehavior::wrap(move || open_browser(this, url))
                    }
                };
            }
        }
        None
    }

    fn on_hover_impl(self: &Rc<Self>, mouse_pos: Vec2D) -> Option<()> {
        let state = self.state.borrow();
        for (end_y, action) in state.mouse_actions.iter() {
            if mouse_pos.y + state.scroll_offset <= *end_y {
                let tooltip = match action {
                    LibraryInfoMouseAction::None => Tooltip {
                        text: "".to_owned(),
                        interaction: vec![InteractionHint::Scroll],
                    },
                    LibraryInfoMouseAction::OpenWebpage(..) => Tooltip {
                        text: "".to_owned(),
                        interaction: vec![InteractionHint::Scroll, InteractionHint::LeftClick],
                    },
                };
                self.with_gui_state_mut(|state| state.set_tooltip(tooltip));
                return Some(());
            }
        }
        self.with_gui_state_mut(|state| {
            state.set_tooltip(Tooltip {
                text: "".to_owned(),
                interaction: vec![InteractionHint::Scroll],
            })
        });
        Some(())
    }

    fn on_scroll_impl(self: &Rc<Self>, _pos: Vec2D, delta: f32) -> Option<()> {
        let mut state = self.state.borrow_mut();
        state.scroll_offset =
            (state.scroll_offset - delta * 100.0).clam(0.0, state.max_scroll_offset);
        Some(())
    }

    // Updates are checked asynchronously so we have to read them from the registry.
    fn draw_impl(self: &Rc<Self>, g: &mut GrahpicsWrapper) {
        let registry_ptr = self.with_gui_state(|state| Rc::clone(&state.registry));
        let mut registry = registry_ptr.borrow_mut();
        registry.poll_update_checker();
        drop(registry);
        let registry = registry_ptr.borrow();
        let mut state = self.state.borrow_mut();
        g.translate((0.0, -state.scroll_offset));

        g.set_color(&COLOR_BG0);
        g.draw_rect(0, TAB_BODY_SIZE);

        state.mouse_actions.clear();
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
        state.mouse_actions.append(&mut new_actions);
        g.draw_text(
            TITLE_FONT_SIZE,
            (0.0, y),
            (TAB_BODY_WIDTH, grid(1)),
            (0, 0),
            1,
            "Installed Libraries",
        );
        y += TITLE_FONT_SIZE + GRID_P;
        state
            .mouse_actions
            .push((y, LibraryInfoMouseAction::None));
        let LibraryInfoState {
            libraries,
            mouse_actions,
            ..
        } = &mut *state;
        for library in libraries {
            let (height, mut new_actions) = self.draw_library_entry(
                g,
                y,
                &library.name,
                library.version,
                &library.description,
                UpdateInfo::None,
            );
            y += height;
            // TODO: This is a gross, lazy hack.
            mouse_actions.append(&mut new_actions);
        }
        state.max_scroll_offset = (y - TAB_BODY_HEIGHT).max(0.0);
    }
}

impl GuiTab for Rc<LibraryInfo> {
    fn get_archetype(&self) -> TabArchetype {
        TabArchetype::LibraryInfo
    }
}
