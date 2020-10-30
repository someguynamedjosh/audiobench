use crate::gui::constants::*;
use crate::gui::graphics::{GrahpicsWrapper, HAlign, VAlign};
use crate::gui::ui_widgets::{IconButton, TextBox};
use crate::gui::{InteractionHint, Tooltip};
use crate::registry::save_data::Patch;
use crate::registry::Registry;
use crate::scui_config::Renderer;
use crate::util::*;
use scui::{ChildHolder, MaybeMouseBehavior, MouseMods, OnClickBehavior, Vec2D, WidgetImpl};
use shared_util::prelude::*;

scui::widget! {
    pub PatchBrowser
    State {
        delete_icon: usize,
        entries: Rcrc<Vec<Rcrc<Patch>>>,
        alphabetical_order: Rcrc<Vec<usize>>,
        current_entry_index: Rcrc<Option<usize>>,
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
    pub fn new(
        parent: &impl PatchBrowserParent,
        current_patch: &Rcrc<Patch>,
        registry: &Rc<Registry>,
    ) -> Rc<Self> {
        const CG: f32 = CG;
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
        let current_entry_index = rcrc(current_entry_index);

        let alphabetical_order = rcrc((0..entries.len()).collect());
        let entries = rcrc(entries);
        Self::sort(&entries, &alphabetical_order);

        // Extra +GRID_P because the padding under the last patch in the list shouldn't be
        // rendered.
        let patch_list_height = TAB_BODY_HEIGHT - GRID_P * 3.0 - name_box.size.1 + GRID_P;
        let num_visible_entries = (patch_list_height / ENTRY_HEIGHT) as usize;

        let state = PatchBrowserState {
            pos: 0.into(),
            delete_icon: registry.lookup_icon("factory:delete").unwrap(),
            entries,
            alphabetical_order,
            current_entry_index,
            num_visible_entries,
            scroll_offset: 0,
        };

        let this = Rc::new(Self::create(parent, state));

        let patch_name = current_patch.borrow().borrow_name().to_owned();
        let save_icon = registry.lookup_icon("factory:save").unwrap();
        let mut save_button = IconButton::new(&this, (GRID_P, 0.0), CG, save_icon, |_| None);
        save_button.set_enabled(save_enabled);
        let new_icon = registry.lookup_icon("factory:add").unwrap();
        let pos = (GRID_P + HW - CG * 3.0 - GRID_P * 2.0, 0.0);
        let new_button = IconButton::new(&this, pos, CG, new_icon, |_| None);
        let copy_icon = registry.lookup_icon("factory:copy").unwrap();
        let pos = (GRID_P + HW - CG * 2.0 - GRID_P, 0.0);
        let copy_button = IconButton::new(&this, pos, CG, copy_icon, |_| None);
        let paste_icon = registry.lookup_icon("factory:paste").unwrap();
        let pos = (GRID_P + HW - CG, 0.0);
        let paste_button = IconButton::new(&this, pos, CG, paste_icon, |_| None);

        let (entries2, order2) = (Rc::clone(&entries), Rc::clone(&alphabetical_order));
        let name_box = TextBox::new(
            &this,
            (GRID_P + CG + GRID_P, 0.0),
            (namew, NAME_BOX_HEIGHT),
            patch_name,
            Box::new(move |text| {
                let (entries3, order3) = (Rc::clone(&entries2), Rc::clone(&order2));
                // TODO: This.
                // MouseAction::Sequence(vec![
                //     MouseAction::RenamePatch(text.to_owned()),
                //     MouseAction::SimpleCallback(Box::new(move || Self::sort(&entries3, &order3))),
                // ])
            }),
        );

        this
    }

    fn sort(&self) {
        let mut state = self.state.borrow_mut();
        state.alphabetical_order.sort_by(|a, b| {
            state.entries[*a]
                .borrow()
                .borrow_name()
                .cmp(&state.entries[*b].borrow().borrow_name())
        });
    }

    fn update_on_patch_change(&self, new_patch: &Rcrc<Patch>) {
        let new_patch_ref = new_patch.borrow();
        let mut children = self.children.borrow_mut();
        children
            .name_box
            .set_text(new_patch_ref.borrow_name().to_owned());
        children
            .save_button
            .set_enabled(new_patch_ref.is_writable());
    }
}

impl WidgetImpl<Renderer> for PatchBrowser {
    fn get_mouse_behavior(
        self: &Rc<Self>,
        mouse_pos: Vec2D,
        mods: &MouseMods,
    ) -> MaybeMouseBehavior {
        let state = self.state.borrow();

        let new_patch_callback: Box<dyn FnMut(&Rcrc<Patch>)> = {
            let entries = Rc::clone(&self.entries);
            let next_entry_index = self.entries.borrow().len();
            let current_entry_index = Rcrc::clone(&self.current_entry_index);
            let alphabetical_order = Rc::clone(&self.alphabetical_order);
            let name_field = Rc::clone(&self.name_box.field);
            let save_button = Rcrc::clone(&self.save_button);
            Box::new(move |new_patch| {
                if new_patch.borrow().exists_on_disk() {
                    alphabetical_order.borrow_mut().push(entries.borrow().len());
                    entries.borrow_mut().push(Rc::clone(new_patch));
                    Self::sort(&entries, &alphabetical_order);
                    *current_entry_index.borrow_mut() = Some(next_entry_index);
                } else {
                    *current_entry_index.borrow_mut() = None;
                }
                Self::update_on_patch_change(&new_patch, &name_field, &save_button);
            })
        };

        if self.save_button.borrow().mouse_in_bounds(mouse_pos) {
            let mut patch_already_existed_on_disk = false;
            if let Some(index) = *self.current_entry_index.borrow() {
                patch_already_existed_on_disk =
                    self.entries.borrow()[index].borrow().exists_on_disk();
            }
            return MouseAction::SavePatch(if patch_already_existed_on_disk {
                Box::new(|_patch| {})
            } else {
                new_patch_callback
            });
        }
        if self.new_button.mouse_in_bounds(mouse_pos) {
            return MouseAction::NewPatch(new_patch_callback);
        }
        if self.paste_button.mouse_in_bounds(mouse_pos) {
            return MouseAction::PastePatchFromClipboard(new_patch_callback);
        }
        if self.copy_button.mouse_in_bounds(mouse_pos) {
            return MouseAction::CopyPatchToClipboard;
        }

        if mouse_pos.x <= HW && mouse_pos.y > NAME_BOX_HEIGHT + GRID_P {
            let entry_index = (mouse_pos.y - NAME_BOX_HEIGHT - GRID_P) / ENTRY_HEIGHT;
            if entry_index >= 0.0 && entry_index < state.entries.borrow().len() as f32 {
                let order_index = entry_index as usize + state.scroll_offset;
                let entry_index = state.alphabetical_order.borrow()[order_index];
                let patch = Rc::clone(&state.entries.borrow()[entry_index]);
                // Delete the patch. The threshold is deliberately shorter than the actual area the
                // icon technically occupies to hopefully make misclicks less likely.
                if mouse_pos.0 > HW - grid(1) && patch.borrow().is_writable() {
                    let mut entries_ref = state.entries.borrow_mut();
                    let res = entries_ref[entry_index].borrow_mut().delete_from_disk();
                    if let Err(err) = res {
                        eprintln!("TODO: Nice error, failed to delete patch: {}", err);
                    } else {
                        entries_ref.remove(entry_index);
                        let mut order_ref = state.alphabetical_order.borrow_mut();
                        order_ref.remove(order_index);
                        for index in &mut *order_ref {
                            if *index > entry_index {
                                *index -= 1;
                            }
                        }
                        let mut current_entry_ref = state.current_entry_index.borrow_mut();
                        if let Some(current_index) = *current_entry_ref {
                            if current_index == entry_index {
                                *current_entry_ref = None;
                            } else if current_index > entry_index {
                                // entry_index cannot be smaller than zero.
                                debug_assert!(current_index > 0);
                                *current_entry_ref = Some(current_index - 1);
                            }
                        }
                    }
                } else {
                    let patch2 = Rc::clone(&patch);
                    let current_entry_index = Rcrc::clone(&state.current_entry_index);
                    let name_box_field = Rcrc::clone(&self.name_box.field);
                    let save_button = Rcrc::clone(&self.save_button);
                    let callback = move || {
                        *current_entry_index.borrow_mut() = Some(entry_index);
                        Self::update_on_patch_change(&patch2, &name_box_field, &save_button);
                    };
                    return MouseAction::LoadPatch(patch, Box::new(callback));
                }
            }
        }
        None
    }

    fn on_scroll(self: &Rc<Self>, mouse_pos: Vec2D, delta: f32) -> bool {
        if mouse_pos.x <= HW && mouse_pos.y > NAME_BOX_HEIGHT + GRID_P {
            let mut state = self.state.borrow_mut();
            if delta > 0.0 {
                if state.scroll_offset > 0 {
                    state.scroll_offset -= 1;
                }
            } else {
                if state.scroll_offset + state.num_visible_entries
                    < state.alphabetical_order.borrow().len()
                {
                    state.scroll_offset += 1;
                }
            }
            true
        } else {
            false
        }
    }

    fn on_hover(self: &Rc<Self>, mouse_pos: Vec2D) -> bool {
        if mouse_pos.x <= HW && mouse_pos.y > NAME_BOX_HEIGHT + GRID_P {
            self.with_gui_state_mut(|state| {
                state.set_tooltip(Tooltip {
                    text: "Click a patch to load it or click the trash icon to delete it"
                        .to_owned(),
                    interaction: InteractionHint::LeftClick | InteractionHint::Scroll,
                });
            })
        }
        true
    }

    fn draw(self: &Rc<Self>, g: &mut Renderer) {
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
        let num_entries = state.alphabetical_order.borrow().len();
        let range = offset..(offset + state.num_visible_entries).min(num_entries);
        for index in range {
            let entry_index = state.alphabetical_order.borrow()[index];
            let entry = &state.entries.borrow()[entry_index];
            let x = GP;
            let y = y + ENTRY_HEIGHT * (index - offset) as f32;
            let item_index = state.alphabetical_order.borrow()[index];
            if Some(item_index) == *state.current_entry_index.borrow() {
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
                const H: HAlign = HAlign::Right;
                g.set_alpha(0.5);
                let t = "[Factory]";
                g.draw_text(FONT_SIZE, (x + GP, y), (width, ENTRY_HEIGHT), (-1, 0), 1, t);
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
