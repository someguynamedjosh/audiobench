use crate::util::*;
use crate::gui::graphics::GrahpicsWrapper;
use crate::gui::widgets::{self, Widget};

pub struct Gui {
    root_widget: Rcrc<dyn Widget>,
    clicked: Option<Rcrc<dyn Widget>>,
    click_position: (i32, i32),
    dragged: bool,
}

impl Gui {
    pub fn new(root: impl Widget + 'static) -> Self {
        Self {
            root_widget: rcrc(root),
            clicked: None,
            click_position: (0, 0),
            dragged: false,
        }
    }

    pub fn draw(&self, g: &mut GrahpicsWrapper) {
        self.root_widget.borrow().draw(g);
    }

    pub fn on_mouse_down(&mut self, pos: (i32, i32)) {
        let hit = widgets::trace_hit(&self.root_widget, pos);
        self.clicked = Some(hit);
        self.click_position = pos;
        self.dragged = false;
    }

    // TODO: Require minimum amount of movement before recognizing dragging.
    pub fn on_mouse_move(&mut self, new_pos: (i32, i32)) {
        if let Some(clicked) = &self.clicked {
            let delta = (
                new_pos.0 - self.click_position.0,
                new_pos.1 - self.click_position.1,
            );
            clicked.borrow_mut().on_drag(delta);
            self.click_position = new_pos;
            self.dragged = true;
        }
    }

    pub fn on_mouse_up(&mut self) {
        self.clicked = None;
    }
}
