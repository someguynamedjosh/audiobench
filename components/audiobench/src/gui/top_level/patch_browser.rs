use crate::gui::constants::*;
use crate::gui::ui_widgets::{IconButton, TextBox};
use crate::gui::{GuiTab, InteractionHint, Tooltip};
use crate::registry::save_data::Patch;
use crate::scui_config::{DropTarget, MaybeMouseBehavior, Renderer};
use clipboard::ClipboardProvider;
use scui::{ChildHolder, MouseMods, OnClickBehavior, Vec2D, Widget, WidgetImpl};
use shared_util::prelude::*;

scui::widget! {
    pub PatchBrowser
    State {
        delete_icon: usize,
        entries: Vec<Rcrc<Patch>>,
        alphabetical_order: Vec<usize>,
        current_entry_index: Option<usize>,
        num_visible_entries: usize,
        scroll_offset: usize,
    }
    Children {
        name_box: ChildHolder<Rc<TextBox>>,
        save_button: ChildHolder<Rc<IconButton>>,
        new_button: ChildHolder<Rc<IconButton>>,
        copy_button: ChildHolder<Rc<IconButton>>,
        paste_button: ChildHolder<Rc<IconButton>>,
    }
}

/// A slightly larger grid size.
const CG: f32 = grid(1) + GRID_P;
const ENTRY_HEIGHT: f32 = CG;
const NAME_BOX_HEIGHT: f32 = CG;
/// How large each half of the GUI takes.
const HW: f32 = (TAB_BODY_WIDTH - GRID_P * 3.0) / 2.0;

impl PatchBrowser {
    pub fn new(parent: &impl PatchBrowserParent) -> Rc<Self> {
        let inter = parent.provide_gui_interface();
        let state = inter.state.borrow();
        let engine = state.engine.borrow();
        let registry = engine.borrow_registry().borrow_mut();
        let current_patch = engine.borrow_current_patch();
        // How many icon buttons to the right of the name box.
        const NUM_ICONS: f32 = 4.0;
        // Width of the name box.
        let namew = HW - (CG + GRID_P) * NUM_ICONS;

        let entries = registry.borrow_patches().clone();
        let current_entry_index = registry
            .borrow_patches()
            .iter()
            .position(|patch| std::ptr::eq(patch.as_ref(), current_patch.as_ref()));
        let mut save_enabled = true;
        if let Some(index) = &current_entry_index {
            if !entries[*index].borrow().is_writable() {
                save_enabled = false;
            }
        } else {
            save_enabled = false;
        }

        let alphabetical_order = (0..entries.len()).collect();
        let entries = entries;

        let state = PatchBrowserState {
            delete_icon: registry.lookup_icon("Factory:delete").unwrap(),
            entries,
            alphabetical_order,
            current_entry_index,
            num_visible_entries: 0,
            scroll_offset: 0,
        };

        let this = Rc::new(Self::create(parent, state));
        this.sort();

        let patch_name = current_patch.borrow().borrow_name().to_owned();
        let save_icon = registry.lookup_icon("Factory:save").unwrap();
        let this2 = Rc::clone(&this);
        let save_button = IconButton::new(&this, (GRID_P, 0.0), CG, save_icon, move |_| {
            this2.on_save_patch()
        });
        save_button.set_enabled(save_enabled);
        let new_icon = registry.lookup_icon("Factory:add").unwrap();
        let pos = (GRID_P + HW - CG * 3.0 - GRID_P * 2.0, 0.0);
        let this2 = Rc::clone(&this);
        let new_button = IconButton::new(&this, pos, CG, new_icon, move |_| {
            this2.on_save_patch_copy()
        });
        let copy_icon = registry.lookup_icon("Factory:copy").unwrap();
        let pos = (GRID_P + HW - CG * 2.0 - GRID_P, 0.0);
        let this2 = Rc::clone(&this);
        let copy_button = IconButton::new(&this, pos, CG, copy_icon, move |_| {
            this2.on_copy_patch_to_clipboard()
        });
        let paste_icon = registry.lookup_icon("Factory:paste").unwrap();
        let pos = (GRID_P + HW - CG, 0.0);
        let this2 = Rc::clone(&this);
        let paste_button = IconButton::new(&this, pos, CG, paste_icon, move |_| {
            this2.on_paste_patch_from_clipboard()
        });

        let this2 = Rc::clone(&this);
        let name_box = TextBox::new(
            &this,
            (GRID_P + CG + GRID_P, 0.0),
            (namew, NAME_BOX_HEIGHT),
            patch_name,
            Box::new(move |text| this2.on_rename_patch(text)),
        );
        name_box.set_enabled(current_patch.borrow().is_writable());

        // Extra +GRID_P because the padding under the last patch in the list shouldn't be
        // rendered.
        let patch_list_height = TAB_BODY_HEIGHT - GRID_P * 3.0 - name_box.get_size().y + GRID_P;
        let num_visible_entries = (patch_list_height / ENTRY_HEIGHT) as usize;

        this.state.borrow_mut().num_visible_entries = num_visible_entries;

        let mut children = this.children.borrow_mut();
        children.name_box = name_box.into();
        children.save_button = save_button.into();
        children.new_button = new_button.into();
        children.copy_button = copy_button.into();
        children.paste_button = paste_button.into();
        drop(children);

        this
    }

    fn after_new_patch(self: &Rc<Self>, new_patch: &Rcrc<Patch>) {
        let mut state = self.state.borrow_mut();
        let next_entry_index = state.entries.len();
        if new_patch.borrow().exists_on_disk() {
            state.alphabetical_order.push(next_entry_index);
            state.entries.push(Rc::clone(new_patch));
            state.current_entry_index = Some(next_entry_index);
            drop(state);
            self.sort();
        } else {
            state.current_entry_index = None;
            drop(state);
        }
        self.update_on_patch_change(new_patch);
    }

    fn on_save_patch(self: &Rc<Self>) -> MaybeMouseBehavior {
        let mut patch_already_existed_on_disk = false;
        let state = self.state.borrow();
        if let Some(index) = state.current_entry_index {
            patch_already_existed_on_disk = state.entries[index].borrow().exists_on_disk();
        }
        let this = Rc::clone(self);
        let engine = self.with_gui_state(|state| Rc::clone(&state.engine));
        OnClickBehavior::wrap(move || {
            let mut engine = engine.borrow_mut();
            engine.save_current_patch();
            if !patch_already_existed_on_disk {
                this.after_new_patch(engine.borrow_current_patch());
            }
        })
    }

    fn on_save_patch_copy(self: &Rc<Self>) -> MaybeMouseBehavior {
        let this = Rc::clone(self);
        let engine = self.with_gui_state(|state| Rc::clone(&state.engine));
        OnClickBehavior::wrap(move || {
            let mut engine = engine.borrow_mut();
            engine.save_current_patch_with_new_name();
            this.after_new_patch(engine.borrow_current_patch());
        })
    }

    fn on_copy_patch_to_clipboard(self: &Rc<Self>) -> MaybeMouseBehavior {
        let this = Rc::clone(self);
        let engine = self.with_gui_state(|state| Rc::clone(&state.engine));
        OnClickBehavior::wrap(move || {
            let patch_data = engine.borrow().serialize_current_patch();
            let mut clipboard: clipboard::ClipboardContext =
                clipboard::ClipboardProvider::new().unwrap();
            clipboard.set_contents(patch_data).unwrap();
            this.with_gui_state_mut(|state| {
                state.add_success_status("Patch data copied to clipboard!".to_owned())
            })
        })
    }

    fn on_paste_patch_from_clipboard(self: &Rc<Self>) -> MaybeMouseBehavior {
        let this = Rc::clone(self);
        let engine = self.with_gui_state(|state| Rc::clone(&state.engine));
        OnClickBehavior::wrap(move || {
            let mut clipboard: clipboard::ClipboardContext =
                clipboard::ClipboardProvider::new().unwrap();
            let data = clipboard.get_contents().unwrap();
            // We use the URL-safe dataset, so letters, numbers, - and _.
            // is_digit(36) checks for numbers and a-z case insensitive.
            let data: String = data
                .chars()
                .filter(|character| {
                    character.is_digit(36) || *character == '-' || *character == '_'
                })
                .collect();
            let mut engine = engine.borrow_mut();
            let res = engine.new_patch_from_clipboard(data.as_bytes());
            if let Ok(patch) = res {
                this.after_new_patch(patch);
                this.with_gui_state_mut(|state| {
                    state.add_success_status(
                        concat!(
                            "Patch data loaded from clipboard! (Click the save button if you want",
                            "to keep it.)"
                        )
                        .to_owned(),
                    );
                });
            } else if let Err(err) = res {
                this.with_gui_state_mut(|state| {
                    state.add_error_status(err);
                });
            }
        })
    }

    fn on_load_patch(self: &Rc<Self>, patch: Rcrc<Patch>, index: usize) -> MaybeMouseBehavior {
        let this = Rc::clone(self);
        let engine = self.with_gui_state(|state| Rc::clone(&state.engine));
        OnClickBehavior::wrap(move || {
            let mut state = this.state.borrow_mut();
            state.current_entry_index = Some(index);
            drop(state);
            this.update_on_patch_change(&patch);
            if let Err(err) = engine.borrow_mut().load_patch(patch) {
                this.with_gui_state_mut(|state| state.add_error_status(err));
            }
        })
    }

    fn on_rename_patch(self: &Rc<Self>, new_name: &str) {
        self.with_gui_state_mut(|state| {
            state
                .engine
                .borrow_mut()
                .rename_current_patch(new_name.to_owned());
        });
        self.sort();
    }

    fn sort(&self) {
        let mut state = self.state.borrow_mut();
        let PatchBrowserState {
            alphabetical_order,
            entries,
            ..
        } = &mut *state;
        alphabetical_order.sort_by(|a, b| {
            entries[*a]
                .borrow()
                .borrow_name()
                .cmp(&entries[*b].borrow().borrow_name())
        });
    }

    fn update_on_patch_change(&self, new_patch: &Rcrc<Patch>) {
        let new_patch_ref = new_patch.borrow();
        let children = self.children.borrow_mut();
        children
            .name_box
            .set_text(new_patch_ref.borrow_name().to_owned());
        let enable = new_patch_ref.is_writable();
        children.name_box.set_enabled(enable);
        children.save_button.set_enabled(enable);
    }

    fn on_delete_patch(self: &Rc<Self>, order_index: usize, index: usize) -> MaybeMouseBehavior {
        let this = Rc::clone(self);
        OnClickBehavior::wrap(move || {
            let mut state = this.state.borrow_mut();
            let res = state.entries[index].borrow_mut().delete_from_disk();
            if let Err(err) = res {
                eprintln!("TODO: Nice error, failed to delete patch: {}", err);
            } else {
                state.entries.remove(index);
                state.alphabetical_order.remove(order_index);
                for other_index in &mut state.alphabetical_order {
                    if *other_index > index {
                        *other_index -= 1;
                    }
                }
                if let Some(current_entry_index) = state.current_entry_index {
                    if current_entry_index == index {
                        state.current_entry_index = None;
                    } else if current_entry_index > index {
                        // index cannot be smaller than zero.
                        debug_assert!(current_entry_index > 0);
                        state.current_entry_index = Some(current_entry_index - 1);
                    }
                }
            }
        })
    }
}

impl WidgetImpl<Renderer, DropTarget> for PatchBrowser {
    fn get_pos_impl(self: &Rc<Self>) -> Vec2D {
        (0.0, HEADER_HEIGHT).into()
    }

    fn get_size_impl(self: &Rc<Self>) -> Vec2D {
        (TAB_BODY_WIDTH, TAB_BODY_HEIGHT).into()
    }

    fn get_mouse_behavior_impl(
        self: &Rc<Self>,
        mouse_pos: Vec2D,
        mods: &MouseMods,
    ) -> MaybeMouseBehavior {
        ris!(self.get_mouse_behavior_children(mouse_pos, mods));
        let state = self.state.borrow();

        if mouse_pos.x <= HW && mouse_pos.y > NAME_BOX_HEIGHT + GRID_P {
            let entry_index = (mouse_pos.y - NAME_BOX_HEIGHT - GRID_P) / ENTRY_HEIGHT;
            if entry_index >= 0.0 && entry_index < state.entries.len() as f32 {
                let order_index = entry_index as usize + state.scroll_offset;
                let entry_index = state.alphabetical_order[order_index];
                let patch = Rc::clone(&state.entries[entry_index]);
                // Delete the patch. The threshold is deliberately shorter than the actual area the
                // icon technically occupies to hopefully make misclicks less likely.
                if mouse_pos.x > HW - grid(1) && patch.borrow().is_writable() {
                    return self.on_delete_patch(order_index, entry_index);
                } else {
                    return self.on_load_patch(patch, entry_index);
                }
            }
        }
        None
    }

    fn on_scroll_impl(self: &Rc<Self>, mouse_pos: Vec2D, delta: f32) -> Option<()> {
        if mouse_pos.x <= HW && mouse_pos.y > NAME_BOX_HEIGHT + GRID_P {
            let mut state = self.state.borrow_mut();
            if delta > 0.0 {
                if state.scroll_offset > 0 {
                    state.scroll_offset -= 1;
                }
            } else {
                if state.scroll_offset + state.num_visible_entries < state.alphabetical_order.len()
                {
                    state.scroll_offset += 1;
                }
            }
            Some(())
        } else {
            None
        }
    }

    fn on_hover_impl(self: &Rc<Self>, mouse_pos: Vec2D) -> Option<()> {
        if mouse_pos.x <= HW && mouse_pos.y > NAME_BOX_HEIGHT + GRID_P {
            self.with_gui_state_mut(|state| {
                state.set_tooltip(Tooltip {
                    text: "Click a patch to load it or click the trash icon to delete it"
                        .to_owned(),
                    interaction: InteractionHint::LeftClick | InteractionHint::Scroll,
                });
            })
        }
        Some(())
    }

    fn draw_impl(self: &Rc<Self>, g: &mut Renderer) {
        const GP: f32 = GRID_P;
        let state = self.state.borrow();

        g.set_color(&COLOR_BG2);
        g.draw_rect(0, TAB_BODY_SIZE);
        self.draw_children(g);

        let y = CG + GP;
        g.set_color(&COLOR_BG0);
        let panel_height = TAB_BODY_HEIGHT - y - GP;
        g.draw_rounded_rect((GP, y), (HW, panel_height), CORNER_SIZE);
        g.set_color(&COLOR_FG1);
        let offset = state.scroll_offset;
        let num_entries = state.alphabetical_order.len();
        let range = offset..(offset + state.num_visible_entries).min(num_entries);
        for index in range {
            let entry_index = state.alphabetical_order[index];
            let entry = &state.entries[entry_index];
            let x = GP;
            let y = y + ENTRY_HEIGHT * (index - offset) as f32;
            let item_index = state.alphabetical_order[index];
            if Some(item_index) == state.current_entry_index {
                g.set_color(&COLOR_BG1);
                g.draw_rounded_rect((x, y), (HW, ENTRY_HEIGHT), CORNER_SIZE);
                g.set_color(&COLOR_FG1);
            }
            let entry = entry.borrow();
            let name = entry.borrow_name();
            let width = if num_entries > state.num_visible_entries {
                HW - GP * 3.0 // Make room for scrollbar.
            } else {
                HW - GP * 2.0
            };
            g.draw_text(
                FONT_SIZE,
                (x + GP, y),
                (width, ENTRY_HEIGHT),
                (-1, 0),
                1,
                name,
            );
            if entry.is_writable() {
                const ICON_SIZE: f32 = grid(1);
                const ICON_PADDING: f32 = (ENTRY_HEIGHT - ICON_SIZE) / 2.0;
                g.draw_white_icon(
                    state.delete_icon,
                    // Don't ask me why it just works
                    (x + width - ICON_SIZE * 0.5, y + ICON_PADDING),
                    ICON_SIZE,
                );
            } else {
                g.set_alpha(0.5);
                let t = "[Factory]";
                g.draw_text(FONT_SIZE, (x + GP, y), (width, ENTRY_HEIGHT), (1, 0), 1, t);
                g.set_alpha(1.0);
            }
        }

        if num_entries > state.num_visible_entries {
            let visible_percent = state.num_visible_entries as f32 / num_entries as f32;
            let offset_percent = offset as f32 / num_entries as f32;
            g.set_color(&COLOR_BG1);
            g.draw_rounded_rect(
                (GP + HW - CORNER_SIZE, y + panel_height * offset_percent),
                (CORNER_SIZE, panel_height * visible_percent),
                CORNER_SIZE,
            );
        }
    }
}

impl GuiTab for Rc<PatchBrowser> {}
