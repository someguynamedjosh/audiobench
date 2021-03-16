use crate::{
    gui::{
        constants::*,
        ui_widgets::{IconButton, LinkButton, TabButton, TextBox},
        GuiTab, InteractionHint, TabArchetype, Tooltip,
    },
    registry::save_data::Patch,
    scui_config::{DropTarget, MaybeMouseBehavior, Renderer},
};
use clipboard::ClipboardProvider;
use observatory::{derivation_with_ptrs_dyn, DerivationDynPtr};
use scui::{ChildHolder, MouseMods, OnClickBehavior, Vec2D, Widget, WidgetImpl};
use shared_util::prelude::*;

scui::widget! {
    pub PatchBrowser
    State {
        delete_icon: usize,
        alphabetical_order: DerivationDynPtr<Vec<usize>>,
        patch_change_effect: Option<DerivationDynPtr<()>>,
        num_visible_entries: usize,
        scroll_offset: usize,
    }
    Children {
        name_box: ChildHolder<Rc<TextBox>>,
        tab_buttons: Vec<Rc<TabButton>>,
        link_button: ChildHolder<Rc<LinkButton>>,
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
        let current_patch = engine.borrow_current_patch().borrow_untracked();

        let alphabetical_order = derivation_with_ptrs_dyn! {
            state.patch_list; {
                let patch_list = patch_list.borrow();
                let mut indexes: Vec<_> = (0..patch_list.len()).collect();
                indexes.sort_by(|a, b| {
                    patch_list[*a]
                        .borrow()
                        .borrow_name()
                        .cmp(&patch_list[*b].borrow().borrow_name())
                });
                indexes
            }
        };

        let state = PatchBrowserState {
            delete_icon: registry.lookup_icon("Factory:delete").unwrap(),
            alphabetical_order,
            patch_change_effect: None,
            num_visible_entries: 0,
            scroll_offset: 0,
        };

        let this = Rc::new(Self::create(parent, state));

        let mut tab_buttons = Vec::new();
        let x = GRID_P + HW + GRID_P;
        tab_buttons.push(TabButton::new(
            &this,
            (x, 0.0),
            registry.lookup_icon("Factory:module").unwrap(),
            TabArchetype::NoteGraph,
            "Module Graph".into(),
            "Edit the modules in this patch".into(),
        ));
        let x = x + TabButton::SIZE + GRID_P;
        tab_buttons.push(TabButton::new(
            &this,
            (x, 0.0),
            registry.lookup_icon("Factory:library").unwrap(),
            TabArchetype::LibraryInfo,
            "Library Info".into(),
            "View information and updates for installed libraries (including the builtin factory library)".into(),
        ));
        let x = x + TabButton::SIZE + GRID_P;
        tab_buttons.push(TabButton::new(
            &this,
            (x, 0.0),
            registry.lookup_icon("Factory:message_log").unwrap(),
            TabArchetype::MessageLog,
            "Message Log".into(),
            "View a log of all info/warning/error messages from this session".into(),
        ));
        let x = x + TabButton::SIZE + GRID_P;
        let link_button = LinkButton::new(
            &this,
            (x, 0.0),
            registry.lookup_icon("Factory:github").unwrap(),
            format!("https://github.com/joshua-maros/audiobench/issues/new"),
            "Report A Bug".into(),
            "Submit a bug report or feature request on GitHub through your web browser.".into(),
        );

        let patch_name = current_patch.borrow().borrow_name().to_owned();
        let this2 = Rc::clone(&this);
        let name_box = TextBox::new(
            &this,
            (GRID_P, 0.0),
            (HW, NAME_BOX_HEIGHT),
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
        children.tab_buttons = tab_buttons;
        children.link_button = link_button.into();
        drop(children);

        let patch_change_effect = derivation_with_ptrs_dyn! {
            this, current_patch: *engine.borrow_current_patch(); {
                this.update_on_patch_change(&current_patch.borrow());
            }
        };
        this.state.borrow_mut().patch_change_effect = Some(patch_change_effect);

        this
    }

    fn on_load_patch(self: &Rc<Self>, patch: Rcrc<Patch>, index: usize) -> MaybeMouseBehavior {
        let this = Rc::clone(self);
        let engine = self.with_gui_state(|state| Rc::clone(&state.engine));
        OnClickBehavior::wrap(move || {
            this.with_gui_state_mut(|state| state.current_patch_index = Some(index));
            if engine.borrow_mut().load_patch(patch).is_ok() {
                this.with_gui_state_mut(|state| {
                    state.add_success_message(format!("Patch loaded."))
                });
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
    }

    fn update_on_patch_change(&self, new_patch: &Rcrc<Patch>) {
        let new_patch_ref = new_patch.borrow();
        let children = self.children.borrow_mut();
        children
            .name_box
            .set_text(new_patch_ref.borrow_name().to_owned());
        let enable = new_patch_ref.is_writable();
        children.name_box.set_enabled(enable);
    }

    fn on_delete_patch(self: &Rc<Self>, order_index: usize, index: usize) -> MaybeMouseBehavior {
        let this = Rc::clone(self);
        OnClickBehavior::wrap(move || {
            let mut state = this.state.borrow_mut();
            let gui_state = this.parents.gui.state.borrow();
            let res = gui_state.patch_list.borrow_mut()[index]
                .borrow_mut()
                .delete_from_disk();
            drop(gui_state);
            if let Err(err) = res {
                this.with_gui_state_mut(|state| {
                    state.add_error_message(format!("{}", err));
                });
                return;
            }
            let mut gui_state = this.parents.gui.state.borrow_mut();
            gui_state.patch_list.borrow_mut().remove(index);
            if let Some(current_patch_index) = gui_state.current_patch_index {
                if current_patch_index == index {
                    gui_state.current_patch_index = None;
                } else if current_patch_index > index {
                    // index cannot be smaller than zero.
                    debug_assert!(current_patch_index > 0);
                    gui_state.current_patch_index = Some(current_patch_index - 1);
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
        let gui_state = self.parents.gui.state.borrow();

        if mouse_pos.x <= HW && mouse_pos.y > NAME_BOX_HEIGHT + GRID_P {
            let entry_index = (mouse_pos.y - NAME_BOX_HEIGHT - GRID_P) / ENTRY_HEIGHT;
            if entry_index >= 0.0
                && entry_index < gui_state.patch_list.borrow_untracked().len() as f32
            {
                let order_index = entry_index as usize + state.scroll_offset;
                let entry_index = state.alphabetical_order.borrow_untracked()[order_index];
                let patch = Rc::clone(&gui_state.patch_list.borrow_untracked()[entry_index]);
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
                if state.scroll_offset + state.num_visible_entries
                    < state.alphabetical_order.borrow_untracked().len()
                {
                    state.scroll_offset += 1;
                }
            }
            Some(())
        } else {
            None
        }
    }

    fn on_hover_impl(self: &Rc<Self>, pos: Vec2D) -> Option<()> {
        ris!(self.on_hover_children(pos));
        if pos.x <= HW && pos.y > NAME_BOX_HEIGHT + GRID_P {
            self.with_gui_state_mut(|state| {
                state.set_tooltip(Tooltip {
                    text: "Click a patch to load it or click the trash icon to delete it"
                        .to_owned(),
                    interaction: vec![InteractionHint::LeftClick, InteractionHint::Scroll],
                });
            })
        }
        Some(())
    }

    fn draw_impl(self: &Rc<Self>, g: &mut Renderer) {
        const GP: f32 = GRID_P;
        let state = self.state.borrow();
        let gui_state = self.parents.gui.state.borrow();

        g.set_color(&COLOR_BG2);
        g.draw_rect(0, TAB_BODY_SIZE);
        self.draw_children(g);

        let y = CG + GP;
        g.set_color(&COLOR_BG0);
        let panel_height = TAB_BODY_HEIGHT - y - GP;
        g.draw_rounded_rect((GP, y), (HW, panel_height), CORNER_SIZE);
        g.set_color(&COLOR_FG1);
        let offset = state.scroll_offset;
        let order = state.alphabetical_order.borrow_untracked();
        let entries = gui_state.patch_list.borrow_untracked();
        let num_entries = order.len();
        let range = offset..(offset + state.num_visible_entries).min(num_entries);
        for index in range {
            let entry_index = order[index];
            let entry = &entries[entry_index];
            let x = GP;
            let y = y + ENTRY_HEIGHT * (index - offset) as f32;
            let item_index = order[index];
            if Some(item_index) == gui_state.current_patch_index {
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

impl GuiTab for Rc<PatchBrowser> {
    fn get_name(self: &Self) -> String {
        format!("Home")
    }

    fn get_archetype(&self) -> TabArchetype {
        TabArchetype::PatchBrowser
    }
}
