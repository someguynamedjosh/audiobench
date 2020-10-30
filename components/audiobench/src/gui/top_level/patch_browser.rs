use crate::gui::constants::*;
use crate::gui::graphics::{GrahpicsWrapper, HAlign, VAlign};
use crate::gui::ui_widgets::{IconButton, TextBox};
use crate::gui::{InteractionHint, Tooltip};
use crate::registry::save_data::Patch;
use crate::registry::Registry;
use crate::util::*;

pub struct PatchBrowser {
    pos: (f32, f32),
    size: (f32, f32),
    name_box: TextBox,
    save_button: Rcrc<IconButton>,
    new_button: IconButton,
    copy_button: IconButton,
    paste_button: IconButton,
    delete_icon: usize,
    entries: Rcrc<Vec<Rcrc<Patch>>>,
    alphabetical_order: Rcrc<Vec<usize>>,
    current_entry_index: Rcrc<Option<usize>>,
    num_visible_entries: usize,
    scroll_offset: usize,
}

impl PatchBrowser {
    // A slightly larger grid size.
    const CG: f32 = grid(1) + GRID_P;
    const ENTRY_HEIGHT: f32 = Self::CG;

    pub fn new(current_patch: &Rcrc<Patch>, registry: &Rc<Registry>) -> Self {
        // How large each half of the GUI takes.
        let hw = (TAB_BODY_WIDTH - GRID_P * 3.0) / 2.0;
        const CG: f32 = PatchBrowser::CG;
        // How many icon buttons to the right of the name box.
        const NUM_ICONS: f32 = 4.0;
        // Width of the name box.
        let namew = hw - (CG + GRID_P) * NUM_ICONS;
        let patch_name = current_patch.borrow().borrow_name().to_owned();
        let save_icon = registry.lookup_icon("factory:save").unwrap();
        let mut save_button = IconButton::new((GRID_P, 0.0), CG, save_icon);
        let new_icon = registry.lookup_icon("factory:add").unwrap();
        let new_button =
            IconButton::create((GRID_P + hw - CG * 3.0 - GRID_P * 2.0, 0.0), CG, new_icon);
        let copy_icon = registry.lookup_icon("factory:copy").unwrap();
        let copy_button = IconButton::create((GRID_P + hw - CG * 2.0 - GRID_P, 0.0), CG, copy_icon);
        let paste_icon = registry.lookup_icon("factory:paste").unwrap();
        let paste_button = IconButton::create((GRID_P + hw - CG, 0.0), CG, paste_icon);

        let entries = registry.borrow_patches().clone();
        let current_entry_index = registry
            .borrow_patches()
            .iter()
            .position(|patch| std::ptr::eq(patch.as_ref(), current_patch.as_ref()));
        if let Some(index) = &current_entry_index {
            if !entries[*index].borrow().is_writable() {
                save_button.enabled = false;
            }
        } else {
            save_button.enabled = false;
        }
        let save_button = rcrc(save_button);
        let current_entry_index = rcrc(current_entry_index);

        let alphabetical_order = rcrc((0..entries.len()).collect());
        let entries = rcrc(entries);
        Self::sort(&entries, &alphabetical_order);

        let (entries2, order2) = (Rc::clone(&entries), Rc::clone(&alphabetical_order));
        let name_box = TextBox::create(
            (GRID_P + CG + GRID_P, 0.0),
            (namew, CG),
            patch_name,
            Box::new(move |text| {
                let (entries3, order3) = (Rc::clone(&entries2), Rc::clone(&order2));
                MouseAction::Sequence(vec![
                    MouseAction::RenamePatch(text.to_owned()),
                    MouseAction::SimpleCallback(Box::new(move || Self::sort(&entries3, &order3))),
                ])
            }),
        );
        // Extra +GRID_P because the padding under the last patch in the list shouldn't be
        // rendered.
        let patch_list_height = size.1 - GRID_P * 3.0 - name_box.size.1 + GRID_P;
        let num_visible_entries = (patch_list_height / Self::ENTRY_HEIGHT) as usize;

        Self {
            pos,
            size,
            name_box,
            save_button,
            new_button,
            copy_button,
            paste_button,
            delete_icon: registry.lookup_icon("factory:delete").unwrap(),
            entries,
            alphabetical_order,
            current_entry_index,
            num_visible_entries,
            scroll_offset: 0,
        }
    }

    fn sort(entries: &Rcrc<Vec<Rcrc<Patch>>>, alphabetical_order: &Rcrc<Vec<usize>>) {
        let entries = entries.borrow();
        alphabetical_order.borrow_mut().sort_by(|a, b| {
            entries[*a]
                .borrow()
                .borrow_name()
                .cmp(&entries[*b].borrow().borrow_name())
        });
    }

    fn update_on_patch_change(
        new_patch: &Rcrc<Patch>,
        name_box_field: &Rcrc<TextField>,
        save_button: &Rcrc<IconButton>,
    ) {
        let new_patch_ref = new_patch.borrow();
        name_box_field.borrow_mut().text = new_patch_ref.borrow_name().to_owned();
        save_button.borrow_mut().enabled = new_patch_ref.is_writable();
    }

    pub fn get_tooltip_at(&self, mouse_pos: (f32, f32)) -> Option<Tooltip> {
        let mouse_pos = mouse_pos.sub(self.pos);
        {
            let mouse_pos = mouse_pos.sub(self.name_box.pos);
            if mouse_pos.inside(self.name_box.size) {
                return Some(if self.save_button.borrow().enabled {
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
        if self.save_button.borrow().mouse_in_bounds(mouse_pos) {
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
        if self.copy_button.mouse_in_bounds(mouse_pos) {
            return Some(Tooltip {
                text: "Copy the current patch to the clipboard (includes unsaved changes)"
                    .to_owned(),
                interaction: InteractionHint::LeftClick.into(),
            });
        }
        if self.paste_button.mouse_in_bounds(mouse_pos) {
            return Some(Tooltip {
                text: "Paste clipboard data as a new patch".to_owned(),
                interaction: InteractionHint::LeftClick.into(),
            });
        }
        let hw = (self.size.0 - GRID_P * 3.0) / 2.0;
        if mouse_pos.0 <= hw && mouse_pos.1 > self.name_box.size.1 + GRID_P {
            return Some(Tooltip {
                text: "Click a patch to load it or click the trash icon to delete it".to_owned(),
                interaction: InteractionHint::LeftClick | InteractionHint::Scroll,
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
        if self.save_button.borrow().enabled {
            let mouse_pos = mouse_pos.sub(self.name_box.pos);
            if mouse_pos.inside(self.name_box.size) {
                return self.name_box.respond_to_mouse_press(mouse_pos, mods);
            }
        }

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

        // How large each half of the GUI takes.
        let hw = (self.size.0 - GRID_P * 3.0) / 2.0;
        if mouse_pos.0 <= hw && mouse_pos.1 > self.name_box.size.1 + GRID_P {
            let entry_index =
                (mouse_pos.1 - self.name_box.size.1 - GRID_P) / PatchBrowser::ENTRY_HEIGHT;
            if entry_index >= 0.0 && entry_index < self.entries.borrow().len() as f32 {
                let order_index = entry_index as usize + self.scroll_offset;
                let entry_index = self.alphabetical_order.borrow()[order_index];
                let patch = Rc::clone(&self.entries.borrow()[entry_index]);
                // Delete the patch. The threshold is deliberately shorter than the actual area the
                // icon technically occupies to hopefully make misclicks less likely.
                if mouse_pos.0 > hw - grid(1) && patch.borrow().is_writable() {
                    let mut entries_ref = self.entries.borrow_mut();
                    let res = entries_ref[entry_index].borrow_mut().delete_from_disk();
                    if let Err(err) = res {
                        eprintln!("TODO: Nice error, failed to delete patch: {}", err);
                    } else {
                        entries_ref.remove(entry_index);
                        let mut order_ref = self.alphabetical_order.borrow_mut();
                        order_ref.remove(order_index);
                        for index in &mut *order_ref {
                            if *index > entry_index {
                                *index -= 1;
                            }
                        }
                        let mut current_entry_ref = self.current_entry_index.borrow_mut();
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
                    let current_entry_index = Rcrc::clone(&self.current_entry_index);
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
        MouseAction::None
    }

    pub fn on_scroll(&mut self, mouse_pos: (f32, f32), delta: f32) -> Option<GuiAction> {
        let hw = (self.size.0 - GRID_P * 3.0) / 2.0;
        if mouse_pos.0 <= hw && mouse_pos.1 > self.name_box.size.1 + GRID_P {
            if delta > 0.0 {
                if self.scroll_offset > 0 {
                    self.scroll_offset -= 1;
                }
            } else {
                if self.scroll_offset + self.num_visible_entries
                    < self.alphabetical_order.borrow().len()
                {
                    self.scroll_offset += 1;
                }
            }
        }
        None
    }

    pub fn draw(&self, g: &mut GrahpicsWrapper) {
        g.push_state();
        g.apply_offset(self.pos.0, self.pos.1);

        // How large each half of the GUI takes.
        let hw = (self.size.0 - GRID_P * 3.0) / 2.0;
        const GP: f32 = GRID_P;

        g.set_color(&COLOR_BG2);
        g.fill_rect(0.0, 0.0, self.size.0, self.size.1);
        self.name_box.draw(g);
        self.save_button.borrow().draw(g);
        self.new_button.draw(g);
        self.copy_button.draw(g);
        self.paste_button.draw(g);

        const CG: f32 = PatchBrowser::CG;
        let y = CG + GP;
        g.set_color(&COLOR_BG0);
        let panel_height = self.size.1 - y - GP;
        g.fill_rounded_rect(GP, y, hw, panel_height, CORNER_SIZE);
        g.set_color(&COLOR_FG1);
        let offset = self.scroll_offset;
        let num_entries = self.alphabetical_order.borrow().len();
        let range = offset..(offset + self.num_visible_entries).min(num_entries);
        for index in range {
            let entry_index = self.alphabetical_order.borrow()[index];
            let entry = &self.entries.borrow()[entry_index];
            const HEIGHT: f32 = PatchBrowser::ENTRY_HEIGHT;
            let x = GP;
            let y = y + HEIGHT * (index - offset) as f32;
            if Some(self.alphabetical_order.borrow()[index]) == *self.current_entry_index.borrow() {
                g.set_color(&COLOR_BG1);
                g.fill_rounded_rect(x, y, hw, HEIGHT, CORNER_SIZE);
                g.set_color(&COLOR_FG1);
            }
            let entry = entry.borrow();
            const H: HAlign = HAlign::Left;
            const V: VAlign = VAlign::Center;
            let name = entry.borrow_name();
            let width = if num_entries > self.num_visible_entries {
                hw - GP * 3.0 // Make room for scrollbar.
            } else {
                hw - GP * 2.0
            };
            g.write_text(FONT_SIZE, x + GP, y, width, HEIGHT, H, V, 1, name);
            if entry.is_writable() {
                const ICON_SIZE: f32 = grid(1);
                const ICON_PADDING: f32 = (HEIGHT - ICON_SIZE) / 2.0;
                g.draw_white_icon(
                    self.delete_icon,
                    // Don't ask me why it just works
                    x + width - ICON_SIZE * 0.5,
                    y + ICON_PADDING,
                    ICON_SIZE,
                );
            } else {
                const H: HAlign = HAlign::Right;
                g.set_alpha(0.5);
                let t = "[Factory]";
                g.write_text(FONT_SIZE, x + GP, y, width, HEIGHT, H, V, 1, t);
                g.set_alpha(1.0);
            }
        }

        if num_entries > self.num_visible_entries {
            let visible_percent = self.num_visible_entries as f32 / num_entries as f32;
            let offset_percent = offset as f32 / num_entries as f32;
            g.set_color(&COLOR_BG1);
            g.fill_rounded_rect(
                GP + hw - CORNER_SIZE,
                y + panel_height * offset_percent,
                CORNER_SIZE,
                panel_height * visible_percent,
                CORNER_SIZE,
            );
        }

        g.pop_state();
    }
}
