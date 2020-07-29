use super::{IconButton, TextBox};
use crate::gui::action::{GuiAction, MouseAction};
use crate::gui::constants::*;
use crate::gui::graphics::{GrahpicsWrapper, HAlign, VAlign};
use crate::gui::{InteractionHint, MouseMods, Tooltip};
use crate::registry::save_data::Patch;
use crate::registry::Registry;
use crate::util::*;

pub struct PatchBrowser {
    pos: (f32, f32),
    size: (f32, f32),
    name_box: TextBox,
    save_button: IconButton,
    new_button: IconButton,
    copy_button: IconButton,
    paste_button: IconButton,
    entries: Rcrc<Vec<Rcrc<Patch>>>,
    alphabetical_order: Rcrc<Vec<usize>>,
    current_entry_index: Option<usize>,
    num_visible_entries: usize,
    scroll_offset: usize,
}

impl PatchBrowser {
    // A slightly larger grid size.
    const CG: f32 = grid(1) + GRID_P;
    const ENTRY_HEIGHT: f32 = Self::CG;

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
        const NUM_ICONS: f32 = 4.0;
        // Width of the name box.
        let namew = hw - (CG + GRID_P) * NUM_ICONS;
        let patch_name = current_patch.borrow().borrow_name().to_owned();
        let save_icon = registry.lookup_icon("factory:save").unwrap();
        let mut save_button =
            IconButton::create((GRID_P + hw - CG * 4.0 - GRID_P * 3.0, 0.0), CG, save_icon);
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

        let alphabetical_order = rcrc((0..entries.len()).collect());
        let entries = rcrc(entries);
        Self::sort(&entries, &alphabetical_order);

        let (entries2, order2) = (Rc::clone(&entries), Rc::clone(&alphabetical_order));
        let name_box = TextBox::create(
            (GRID_P, 0.0),
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
            entries,
            alphabetical_order,
            current_entry_index,
            num_visible_entries,
            scroll_offset: 3,
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

    fn sort_self(&mut self) {
        Self::sort(&self.entries, &self.alphabetical_order);
    }

    fn update_on_patch_change(&mut self) {
        let entries_ref = self.entries.borrow();
        if let Some(index) = self.current_entry_index {
            let entry_ref = entries_ref[index].borrow();
            self.name_box.field.borrow_mut().text = entry_ref.borrow_name().to_owned();
            self.save_button.enabled = entry_ref.is_writable();
        } else {
            self.name_box.field.borrow_mut().text = "External Preset".to_owned();
            self.save_button.enabled = false;
        }
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
            self.sort_self();
            return MouseAction::SavePatch;
        }
        let new_patch_callback: Box<dyn Fn(&Rcrc<Patch>)> = {
            let entries = Rc::clone(&self.entries);
            let alphabetical_order = Rc::clone(&self.alphabetical_order);
            let name_field = Rc::clone(&self.name_box.field);
            Box::new(move |new_patch| {
                alphabetical_order.borrow_mut().push(entries.borrow().len());
                entries.borrow_mut().push(Rc::clone(new_patch));
                name_field.borrow_mut().text = new_patch.borrow().borrow_name().to_owned();
                Self::sort(&entries, &alphabetical_order);
            })
        };
        if self.new_button.mouse_in_bounds(mouse_pos) {
            self.current_entry_index = Some(self.entries.borrow().len());
            self.save_button.enabled = true;
            return MouseAction::NewPatch(new_patch_callback);
        } else if self.paste_button.mouse_in_bounds(mouse_pos) {
            self.current_entry_index = Some(self.entries.borrow().len());
            self.save_button.enabled = true;
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
                let entry_index =
                    self.alphabetical_order.borrow()[entry_index as usize + self.scroll_offset];
                self.current_entry_index = Some(entry_index);
                self.update_on_patch_change();
                return MouseAction::LoadPatch(Rc::clone(&self.entries.borrow()[entry_index]));
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

        g.set_color(&COLOR_SURFACE);
        g.fill_rect(0.0, 0.0, self.size.0, self.size.1);
        self.name_box.draw(g);
        self.save_button.draw(g);
        self.new_button.draw(g);
        self.copy_button.draw(g);
        self.paste_button.draw(g);

        const CG: f32 = PatchBrowser::CG;
        let y = CG + GP;
        g.set_color(&COLOR_BG);
        let panel_height = self.size.1 - y - GP;
        g.fill_rounded_rect(GP, y, hw, panel_height, CORNER_SIZE);
        g.set_color(&COLOR_TEXT);
        let offset = self.scroll_offset;
        let num_entries = self.alphabetical_order.borrow().len();
        let range = offset..(offset + self.num_visible_entries).min(num_entries);
        for index in range {
            let entry = &self.entries.borrow()[self.alphabetical_order.borrow()[index]];
            const HEIGHT: f32 = PatchBrowser::ENTRY_HEIGHT;
            let x = GP;
            let y = y + HEIGHT * (index - offset) as f32;
            if Some(self.alphabetical_order.borrow()[index]) == self.current_entry_index {
                g.set_color(&COLOR_IO_AREA);
                g.fill_rounded_rect(x, y, hw, HEIGHT, CORNER_SIZE);
                g.set_color(&COLOR_TEXT);
            }
            let entry = entry.borrow();
            const H: HAlign = HAlign::Left;
            const V: VAlign = VAlign::Center;
            let name = entry.borrow_name();
            g.write_text(FONT_SIZE, x + GP, y, hw - GP * 2.0, HEIGHT, H, V, 1, name);
            let width = if num_entries > self.num_visible_entries {
                hw - GP * 3.0 // Make room for scrollbar.
            } else {
                hw - GP * 2.0
            };
            if !entry.is_writable() {
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
            g.set_color(&COLOR_IO_AREA);
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
