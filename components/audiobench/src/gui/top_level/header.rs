use crate::gui::constants::*;
use crate::scui_config::Renderer;
use scui::{Vec2D, WidgetImpl};
use shared_util::prelude::*;

scui::widget! {
    pub Header
}

impl Header {
    pub fn new(parent: &impl HeaderParent) -> Rc<Self> {
        let state = HeaderState { pos: Vec2D::zero() };
        let this = Rc::new(Self::create(parent, state));
        this
    }
}

impl WidgetImpl<Renderer> for Header {
    fn get_size(self: &Rc<Self>) -> Vec2D {
        (ROOT_WIDTH, HEADER_HEIGHT).into()
    }

    fn draw(self: &Rc<Self>, renderer: &mut Renderer) {
        const CS: f32 = CORNER_SIZE;
        const GP: f32 = GRID_P;

        renderer.set_color(&COLOR_BG2);
        renderer.draw_rect(0, (ROOT_WIDTH, HEADER_HEIGHT));

        let tooltip_size: Vec2D = (ROOT_WIDTH - GP * 2.0, TOOLTIP_HEIGHT).into();
        renderer.set_color(&COLOR_BG0);
        renderer.draw_rounded_rect(GP, tooltip_size, CS);
        let textbox_size = tooltip_size - GP * 2.0;
        renderer.set_color(&COLOR_FG1);
        renderer.draw_
    }
}
