use crate::{
    engine::parts::JackType,
    gui::{constants::*, InteractionHint},
    scui_config::{DropTarget, MaybeMouseBehavior, Renderer},
};
use scui::{MouseMods, OnClickBehavior, Vec2D, WidgetImpl};
use shared_util::prelude::*;
use std::collections::HashMap;

scui::widget! {
    pub Header
    State {
        hint_icons: HashMap<InteractionHint, Vec<usize>>,
    }
}

const TAB_SIZE: Vec2D = Vec2D::new(grid(4), grid(1));
const TAB_PADDING: f32 = GRID_P * 0.5;
const TAB_HEIGHT: f32 = grid(1);

impl Header {
    pub fn new(parent: &impl HeaderParent) -> Rc<Self> {
        let inter = parent.provide_gui_interface();
        let state = inter.state.borrow();
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

        let state = HeaderState { hint_icons };
        let this = Rc::new(Self::create(parent, state));
        this
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

        let tab_index = (pos.x / (TAB_SIZE.x + TAB_PADDING)) as usize;
        let this = Rc::clone(self);
        OnClickBehavior::wrap(move || {
            this.with_gui_state_mut(|state| {
                state.focus_tab_by_index(tab_index);
            });
        })
    }

    fn draw_impl(self: &Rc<Self>, renderer: &mut Renderer) {
        let self_state = self.state.borrow();
        const BFS: f32 = BIG_FONT_SIZE;
        const CS: f32 = CORNER_SIZE;
        const GP: f32 = GRID_P;
        const FS: f32 = FONT_SIZE;

        renderer.set_color(&COLOR_BG2);
        renderer.draw_rect((0.0, TAB_HEIGHT), (ROOT_WIDTH, HEADER_HEIGHT - grid(1)));
        renderer.set_color(&COLOR_BG0);
        renderer.draw_rect(0, (ROOT_WIDTH, grid(1)));

        renderer.set_color(&COLOR_BG0);
        let tooltip_size: Vec2D = (ROOT_WIDTH - GP * 2.0, TOOLTIP_HEIGHT).into();
        renderer.draw_rounded_rect((GP, GP + TAB_HEIGHT), tooltip_size, CS);
        let textbox_size = tooltip_size - GP * 2.0;
        self.with_gui_state(|state| {
            let tooltip = &state.borrow_tooltip();
            renderer.set_color(&COLOR_FG1);
            renderer.draw_text(
                BFS,
                (GP * 2.0, GP * 2.0 + TAB_HEIGHT),
                textbox_size,
                (-1, -1),
                1,
                &tooltip.text,
            );

            let mut hints = tooltip.interaction.clone();
            hints.sort();
            const OUTSIDE_BIAS: f32 = 1.3;
            const IP: f32 = TOOLTIP_HEIGHT * 0.11;
            const IIP: f32 = IP * (2.0 - OUTSIDE_BIAS);
            const OIP: f32 = IP * OUTSIDE_BIAS;
            const IS: f32 = TOOLTIP_HEIGHT - IP * 4.0;
            let mut pos = Vec2D::new(
                tooltip_size.x + GP - IS - IIP - OIP,
                GP + TAB_HEIGHT + IIP + OIP,
            );
            for hint in hints.iter().rev() {
                if let Some(icons) = self_state.hint_icons.get(hint) {
                    let width = icons.len() as f32 * (IS + IIP) + IIP;
                    renderer.draw_rounded_rect(
                        pos + (IS + IIP - width, -IIP),
                        (width, TOOLTIP_HEIGHT - OIP * 2.0),
                        CS,
                    );
                    for icon in icons.iter().rev() {
                        renderer.draw_icon(*icon, pos, IS);
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
                    renderer.set_color(&COLOR_BG2);
                } else {
                    renderer.set_color(&COLOR_BG1);
                }
                renderer.draw_rect(pos, TAB_SIZE);
                renderer.set_color(&COLOR_FG1);
                renderer.draw_text(FS, pos, TAB_SIZE, (0, 0), 1, &tab.get_name());
                pos.x += TAB_SIZE.x + TAB_PADDING;
                index += 1;
            }
        });
    }
}
