use super::{IconButton, TextBox};
use crate::gui::action::MouseAction;
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
    entries: Rcrc<Vec<Rcrc<Patch>>>,
    alphabetical_order: Rcrc<Vec<usize>>,
    current_entry_index: usize,
}

impl PatchBrowser {
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
        const NUM_ICONS: f32 = 2.0;
        // Width of the name box.
        let namew = hw - (CG + GRID_P) * NUM_ICONS;
        let patch_name = current_patch.borrow().borrow_name().to_owned();
        let save_icon = registry.lookup_icon("factory:save").unwrap();
        let mut save_button =
            IconButton::create((GRID_P + hw - CG * 2.0 - GRID_P, 0.0), CG, save_icon);
        let new_icon = registry.lookup_icon("factory:add").unwrap();
        let new_button = IconButton::create((GRID_P + hw - CG, 0.0), CG, new_icon);

        let entries = registry.borrow_patches().clone();
        let current_entry_index = registry
            .borrow_patches()
            .iter()
            .position(|patch| std::ptr::eq(patch.as_ref(), current_patch.as_ref()))
            .unwrap();
        if !entries[current_entry_index].borrow().is_writable() {
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

        Self {
            pos,
            size,
            name_box,
            save_button,
            new_button,
            entries,
            alphabetical_order,
            current_entry_index,
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
        let entry_ref = entries_ref[self.current_entry_index].borrow();
        self.name_box.field.borrow_mut().text = entry_ref.borrow_name().to_owned();
        self.save_button.enabled = entry_ref.is_writable();
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
        if self.new_button.mouse_in_bounds(mouse_pos) {
            self.current_entry_index = self.entries.borrow().len();
            let entries = Rc::clone(&self.entries);
            self.save_button.enabled = true;
            self.name_box.field.borrow_mut().text = "New Patch".to_owned();
            self.sort_self();
            return MouseAction::NewPatch(Box::new(move |new_patch| {
                entries.borrow_mut().push(Rc::clone(new_patch))
            }));
        }
        // How large each half of the GUI takes.
        let hw = (self.size.0 - GRID_P * 3.0) / 2.0;
        if mouse_pos.0 <= hw && mouse_pos.1 > self.name_box.size.1 + GRID_P {
            let entry_index =
                (mouse_pos.1 - self.name_box.size.1 - GRID_P) / PatchBrowser::ENTRY_HEIGHT;
            if entry_index >= 0.0 && entry_index < self.entries.borrow().len() as f32 {
                let entry_index = self.alphabetical_order.borrow()[entry_index as usize];
                self.current_entry_index = entry_index;
                self.update_on_patch_change();
                return MouseAction::LoadPatch(Rc::clone(
                    &self.entries.borrow()[self.current_entry_index],
                ));
            }
        }
        MouseAction::None
    }

    // A slightly larger grid size.
    const CG: f32 = grid(1) + GRID_P;
    const ENTRY_HEIGHT: f32 = Self::CG;

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

        const CG: f32 = PatchBrowser::CG;
        let y = CG + GP;
        g.set_color(&COLOR_BG);
        g.fill_rounded_rect(GP, y, hw, self.size.1 - y - GP, CORNER_SIZE);
        g.set_color(&COLOR_TEXT);
        for index in 0..self.alphabetical_order.borrow().len() {
            let entry = &self.entries.borrow()[self.alphabetical_order.borrow()[index]];
            const HEIGHT: f32 = PatchBrowser::ENTRY_HEIGHT;
            let x = GP;
            let y = y + HEIGHT * index as f32;
            if self.alphabetical_order.borrow()[index] == self.current_entry_index {
                g.set_color(&COLOR_IO_AREA);
                g.fill_rounded_rect(x, y, hw, HEIGHT, CORNER_SIZE);
                g.set_color(&COLOR_TEXT);
            }
            let entry = entry.borrow();
            const H: HAlign = HAlign::Left;
            const V: VAlign = VAlign::Center;
            let name = entry.borrow_name();
            g.write_text(FONT_SIZE, x + GP, y, hw - GP * 2.0, HEIGHT, H, V, 1, name);
            if !entry.is_writable() {
                const H: HAlign = HAlign::Right;
                g.set_alpha(0.5);
                let t = "[Factory]";
                g.write_text(FONT_SIZE, x + GP, y, hw - GP * 2.0, HEIGHT, H, V, 1, t);
                g.set_alpha(1.0);
            }
        }

        g.pop_state();
    }
}
