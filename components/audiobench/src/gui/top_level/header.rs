use crate::{
    engine::parts::JackType,
    gui::{
        constants::*,
        ui_widgets::{IconButton, LinkButton, TabButton, TextBox},
        InteractionHint,
    },
    scui_config::{DropTarget, MaybeMouseBehavior, Renderer},
};
use clipboard::ClipboardProvider;
use observatory::{derivation_with_ptrs_dyn, DerivationDynPtr};
use scui::{ChildHolder, MouseMods, OnClickBehavior, Vec2D, Widget, WidgetImpl};
use shared_util::prelude::*;
use std::collections::HashMap;

scui::widget! {
    pub Header
    State {
        hint_icons: HashMap<InteractionHint, Vec<usize>>,
        on_patch_change_effect: Option<DerivationDynPtr<()>>,
    }
    Children {
        save_button: ChildHolder<Rc<IconButton>>,
        new_button: ChildHolder<Rc<IconButton>>,
        copy_button: ChildHolder<Rc<IconButton>>,
        paste_button: ChildHolder<Rc<IconButton>>,
    }
}

const TAB_SIZE: Vec2D = Vec2D::new(grid(4), grid(1));
const TAB_PADDING: f32 = GRID_P * 0.5;
const TAB_HEIGHT: f32 = grid(1);
const TOOLTIP_START: f32 = TAB_HEIGHT + GRID_P;

impl Header {
    pub fn new(parent: &impl HeaderParent) -> Rc<Self> {
        let inter = parent.provide_gui_interface();
        let state = inter.state.borrow();
        let engine = state.engine.borrow();
        let registry = state.registry.borrow();

        let mut hint_icons = HashMap::new();
        let i = |name: &str| registry.lookup_icon(name).unwrap();
        use InteractionHint::*;
        hint_icons.insert(LeftClick, vec![i("Factory:left_click")]);
        hint_icons.insert(
            LeftClickAndDrag,
            vec![i("Factory:left_click"), i("Factory:move")],
        );
        hint_icons.insert(
            DoubleClick,
            vec![i("Factory:left_click"), i("Factory:left_click")],
        );
        hint_icons.insert(RightClick, vec![i("Factory:right_click")]);
        hint_icons.insert(Scroll, vec![i("Factory:scroll")]);
        hint_icons.insert(PrecisionModifier, vec![i("Factory:alt")]);
        hint_icons.insert(SnappingModifier, vec![i("Factory:shift")]);

        for jt in &[
            JackType::Audio,
            JackType::Pitch,
            JackType::Trigger,
            JackType::Waveform,
        ] {
            let ji = i(&jt.icon_name());
            let arrow = i("Factory:arrow_right");
            hint_icons.insert(TakesInput(*jt), vec![ji, arrow]);
            hint_icons.insert(ProducesOutput(*jt), vec![arrow, ji]);
        }

        let state = HeaderState {
            hint_icons,
            on_patch_change_effect: None,
        };
        let this = Rc::new(Self::create(parent, state));

        let mut children = this.children.borrow_mut();
        let this2 = Rc::clone(&this);
        let save_button = IconButton::new(
            &this,
            (GRID_P, TOOLTIP_START),
            TOOLTIP_HEIGHT,
            registry.lookup_icon("Factory:save").unwrap(),
            move |_| this2.on_save_patch(),
            "Save the current patch",
        );
        let this2 = Rc::clone(&this);
        let new_button = IconButton::new(
            &this,
            (GRID_P + TOOLTIP_HEIGHT, TOOLTIP_START),
            TOOLTIP_HEIGHT,
            registry.lookup_icon("Factory:add").unwrap(),
            move |_| this2.on_save_patch_copy(),
            "Create a new patch based on the current patch (including unsaved changes)",
        );
        let this2 = Rc::clone(&this);
        let copy_button = IconButton::new(
            &this,
            (GRID_P + TOOLTIP_HEIGHT * 2.0, TOOLTIP_START),
            TOOLTIP_HEIGHT,
            registry.lookup_icon("Factory:copy").unwrap(),
            move |_| this2.on_copy_patch_to_clipboard(),
            "Copy the current patch to the clipboard (including unsaved changes)",
        );
        let this2 = Rc::clone(&this);
        let paste_button = IconButton::new(
            &this,
            (GRID_P + TOOLTIP_HEIGHT * 3.0, TOOLTIP_START),
            TOOLTIP_HEIGHT,
            registry.lookup_icon("Factory:paste").unwrap(),
            move |_| this2.on_paste_patch_from_clipboard(),
            "Paste and load a patch from your clipboard",
        );

        let on_patch_change_effect = derivation_with_ptrs_dyn!(
            save_button, current_patch: *engine.borrow_current_patch(); {
                save_button.set_enabled(current_patch.borrow().borrow().is_writable());
            }
        );

        children.save_button = save_button.into();
        children.new_button = new_button.into();
        children.copy_button = copy_button.into();
        children.paste_button = paste_button.into();
        drop(children);

        this.state.borrow_mut().on_patch_change_effect = Some(on_patch_change_effect);

        this
    }

    fn on_save_patch(self: &Rc<Self>) -> MaybeMouseBehavior {
        let mut patch_already_existed_on_disk = false;
        let state = self.state.borrow();
        let gstate = self.parents.gui.state.borrow();
        if let Some(index) = gstate.current_patch_index {
            patch_already_existed_on_disk = gstate.patch_list.borrow_untracked()[index]
                .borrow()
                .exists_on_disk();
        }
        let engine = self.with_gui_state(|state| Rc::clone(&state.engine));
        let gui = Rc::clone(&self.parents.gui);
        OnClickBehavior::wrap(move || {
            let mut engine = engine.borrow_mut();
            engine.save_current_patch();
            let new_patch = engine.borrow_current_patch();
            let new_patch = Rc::clone(&*new_patch.borrow_untracked());
            drop(engine);
            let mut gstate = gui.state.borrow_mut();
            gstate.add_success_message("Patch saved successfully.".to_owned());
            if !patch_already_existed_on_disk {
                println!("After new patch");
                gstate.after_new_patch(&new_patch);
            }
        })
    }

    fn on_save_patch_copy(self: &Rc<Self>) -> MaybeMouseBehavior {
        let this = Rc::clone(self);
        let engine = self.with_gui_state(|state| Rc::clone(&state.engine));
        OnClickBehavior::wrap(move || {
            let mut engine = engine.borrow_mut();
            engine.save_current_patch_with_new_name();
            let new_patch = Rc::clone(&*engine.borrow_current_patch().borrow_untracked());
            this.with_gui_state_mut(|state| state.after_new_patch(&new_patch));
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
                state.add_success_message("Patch data copied to clipboard.".to_owned())
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
                this.with_gui_state_mut(|state| {
                    state.current_patch_index = None;
                    state.add_success_message(
                        concat!(
                            "Patch data loaded from clipboard. (Click the save button if you want",
                            "to keep it.)"
                        )
                        .to_owned(),
                    );
                });
            }
        })
    }
}

impl WidgetImpl<Renderer, DropTarget> for Header {
    fn get_pos_impl(self: &Rc<Self>) -> Vec2D {
        0.into()
    }

    fn get_size_impl(self: &Rc<Self>) -> Vec2D {
        (ROOT_WIDTH, HEADER_HEIGHT).into()
    }

    fn get_mouse_behavior_impl(
        self: &Rc<Self>,
        pos: Vec2D,
        mods: &MouseMods,
    ) -> MaybeMouseBehavior {
        ris!(self.get_mouse_behavior_children(pos, mods));
        if pos.y > TAB_HEIGHT {
            return None;
        }

        let tab_index = (pos.x / (TAB_SIZE.x + TAB_PADDING)) as usize;
        let this = Rc::clone(self);
        OnClickBehavior::wrap(move || {
            this.with_gui_state_mut(|state| {
                if tab_index < state.num_tabs() {
                    state.focus_tab_by_index(tab_index);
                }
            });
        })
    }

    fn draw_impl(self: &Rc<Self>, r: &mut Renderer) {
        let self_state = self.state.borrow();
        const BFS: f32 = BIG_FONT_SIZE;
        const CS: f32 = CORNER_SIZE;
        const GP: f32 = GRID_P;
        const FS: f32 = FONT_SIZE;

        r.set_color(&COLOR_BG2);
        r.draw_rect((0.0, TAB_HEIGHT), (ROOT_WIDTH, HEADER_HEIGHT - grid(1)));
        r.set_color(&COLOR_BG0);
        r.draw_rect(0, (ROOT_WIDTH, grid(1)));
        let mouse_in_header = self.parents.gui.get_mouse_pos().y <= HEADER_HEIGHT;

        let show_buttons = self.with_gui_state(|state| {
            let tooltip = &state.borrow_tooltip();
            let show_buttons = tooltip.text.len() == 0 && tooltip.interaction.len() > 0
                || mouse_in_header
                || state.borrow_last_message().is_some();
            let button_size = if show_buttons {
                r.set_color(&COLOR_BG0);
                r.draw_rounded_rect(
                    (GP, GP + TAB_HEIGHT),
                    (TOOLTIP_HEIGHT * 4.0, TOOLTIP_HEIGHT),
                    CS,
                );
                TOOLTIP_HEIGHT * 4.0 + GP
            } else {
                0.0
            };
            let (text, color) = if let Some(status) = state.borrow_last_message() {
                (status.text.split("\n").next().unwrap(), &status.color)
            } else {
                (&tooltip.text[..], &COLOR_BG0)
            };
            r.set_color(color);
            let tooltip_size: Vec2D = (ROOT_WIDTH - GP * 2.0 - button_size, TOOLTIP_HEIGHT).into();
            r.draw_rounded_rect((GP + button_size, GP + TAB_HEIGHT), tooltip_size, CS);

            let textbox_size = tooltip_size - GP * 2.0;
            r.set_color(&COLOR_FG1);
            r.draw_text(
                BFS,
                (GP * 2.0 + button_size, GP * 2.0 + TAB_HEIGHT),
                textbox_size,
                (-1, -1),
                1,
                text,
            );

            let mut hints = tooltip.interaction.clone();
            hints.sort();
            const OUTSIDE_BIAS: f32 = 1.3;
            const IP: f32 = TOOLTIP_HEIGHT * 0.11;
            const IIP: f32 = IP * (2.0 - OUTSIDE_BIAS);
            const OIP: f32 = IP * OUTSIDE_BIAS;
            const IS: f32 = TOOLTIP_HEIGHT - IP * 4.0;
            let mut pos = Vec2D::new(
                tooltip_size.x + button_size + GP - IS - IIP - OIP,
                GP + TAB_HEIGHT + IIP + OIP,
            );
            for hint in hints.iter().rev() {
                if let Some(icons) = self_state.hint_icons.get(hint) {
                    let width = icons.len() as f32 * (IS + IIP) + IIP;
                    r.draw_rounded_rect(
                        pos + (IS + IIP - width, -IIP),
                        (width, TOOLTIP_HEIGHT - OIP * 2.0),
                        CS,
                    );
                    for icon in icons.iter().rev() {
                        r.draw_icon(*icon, pos, IS);
                        pos.x -= IS + IIP;
                    }
                    pos.x -= IIP + OIP;
                }
            }

            let mut pos: Vec2D = 0.into();
            let mut index = 0;
            let active_index = state.get_current_tab_index();
            for tab in state.all_tabs() {
                if index == active_index {
                    r.set_color(&COLOR_BG2);
                } else {
                    r.set_color(&COLOR_BG1);
                }
                r.draw_rect(pos, TAB_SIZE);
                r.set_color(&COLOR_FG1);
                r.draw_text(FS, pos, TAB_SIZE, (0, 0), 1, &tab.get_name());
                pos.x += TAB_SIZE.x + TAB_PADDING;
                index += 1;
            }

            show_buttons
        });

        if show_buttons {
            self.draw_children(r);
        }
    }
}
