use crate::{
    gui::constants::*,
    scui_config::{DropTarget, MaybeMouseBehavior, Renderer},
};
use scui::{MouseMods, OnClickBehavior, Vec2D, WidgetImpl};
use shared_util::prelude::*;

scui::widget! {
    pub Header
}

const TAB_SIZE: Vec2D = Vec2D::new(grid(4), grid(1));
const TAB_PADDING: f32 = GRID_P * 0.5;
const TAB_HEIGHT: f32 = grid(1);

impl Header {
    pub fn new(parent: &impl HeaderParent) -> Rc<Self> {
        let state = HeaderState {};
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
