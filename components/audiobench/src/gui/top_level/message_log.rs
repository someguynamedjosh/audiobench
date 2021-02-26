use crate::{
    config::{ENGINE_INFO, ENGINE_UPDATE_URL, ENGINE_VERSION},
    engine::controls::AnyControl,
    gui::{
        constants::*, graphics::GrahpicsWrapper, GuiTab, InteractionHint, TabArchetype, Tooltip,
    },
    registry::{module_template::ModuleTemplate, Registry},
    scui_config::{DropTarget, MaybeMouseBehavior, Renderer},
};
use scui::{MouseMods, OnClickBehavior, Vec2D, WidgetImpl};
use shared_util::prelude::*;

scui::widget! {
    pub MessageLog
    State {
        scroll_offset: f32,
    }
}

impl MessageLog {
    pub fn new(parent: &impl MessageLogParent) -> Rc<Self> {
        let state = MessageLogState { scroll_offset: 0.0 };
        Rc::new(Self::create(parent, state))
    }
}

const LINE_HEIGHT: f32 = FONT_SIZE + 1.0;

impl WidgetImpl<Renderer, DropTarget> for MessageLog {
    fn get_pos_impl(self: &Rc<Self>) -> Vec2D {
        (0.0, HEADER_HEIGHT).into()
    }

    fn get_size_impl(self: &Rc<Self>) -> Vec2D {
        TAB_BODY_SIZE
    }

    fn on_scroll_impl(self: &Rc<Self>, _pos: Vec2D, delta: f32) -> Option<()> {
        let mut state = self.state.borrow_mut();
        state.scroll_offset -= delta * 100.0;
        state.scroll_offset = state.scroll_offset.max(0.0);
        Some(())
    }

    fn draw_impl(self: &Rc<Self>, renderer: &mut Renderer) {
        let gui_state = self.parents.gui.state.borrow();
        let mut y = GRID_P - self.state.borrow().scroll_offset;
        for message in gui_state.borrow_all_messages().iter().rev() {
            let num_lines = message.text.split("\n").count();
            let height = GRID_P * 2.0 + num_lines as f32 * LINE_HEIGHT;
            renderer.set_color(&message.color);
            let top_right = (GRID_P, y);
            let size = (TAB_BODY_WIDTH - GRID_P * 2.0, height);
            renderer.draw_rounded_rect(top_right, size, CORNER_SIZE);
            renderer.set_color(&COLOR_BG1);
            renderer.set_alpha(0.8);
            renderer.draw_rounded_rect(top_right, size, CORNER_SIZE);
            renderer.set_alpha(1.0);
            y += GRID_P;
            renderer.set_color(&COLOR_FG1);
            for line in message.text.split("\n") {
                renderer.draw_text(
                    FONT_SIZE,
                    (GRID_P * 2.0, y),
                    (TAB_BODY_WIDTH - GRID_P * 4.0, height),
                    (-1, -1),
                    1,
                    line,
                );
                y += LINE_HEIGHT;
            }
            y += GRID_P * 2.0;
        }
        // TODO: This is hacky and disgusting.
        if y < 0.0 {
            let mut state = self.state.borrow_mut();
            state.scroll_offset = (state.scroll_offset - 600.0).max(0.0);
            drop(state);
            WidgetImpl::draw_impl(self, renderer);
        }
    }
}

impl GuiTab for Rc<MessageLog> {
    fn get_name(self: &Self) -> String {
        format!("Message Log")
    }

    fn get_archetype(&self) -> TabArchetype {
        TabArchetype::MessageLog
    }
}
