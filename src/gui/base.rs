use crate::gui::graphics::GrahpicsWrapper;
use crate::util::*;

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
        let hit = trace_hit(&self.root_widget, pos);
        self.clicked = Some(hit);
        self.click_position = pos;
        self.dragged = false;
    }

    /// Minimum number of pixels the mouse must move before dragging starts.
    const MIN_DRAG_DELTA: i32 = 4;
    pub fn on_mouse_move(&mut self, new_pos: (i32, i32)) {
        if let Some(clicked) = &self.clicked {
            let delta = (
                new_pos.0 - self.click_position.0,
                new_pos.1 - self.click_position.1,
            );
            if !self.dragged {
                let distance = delta.0.abs() + delta.1.abs();
                if distance > Self::MIN_DRAG_DELTA {
                    self.dragged = true;
                }
            }
            if self.dragged {
                clicked.borrow_mut().on_drag(delta);
                self.click_position = new_pos;
            }
        }
    }

    pub fn on_mouse_up(&mut self) {
        self.clicked = None;
        self.dragged = false;
    }
}

fn trace_hit(widget: &Rcrc<dyn Widget>, pixel: (i32, i32)) -> Rcrc<dyn Widget> {
    let widget_ref = widget.borrow();
    if let Some(child) = widget_ref.trace_hit(pixel) {
        child
    } else {
        drop(widget_ref);
        Rc::clone(widget)
    }
}

pub trait Widget {
    fn get_pos(&self) -> (i32, i32);
    fn get_size(&self) -> (i32, i32);
    fn borrow_children(&self) -> &[Rcrc<dyn Widget>] {
        &[]
    }

    fn on_mouse_down(&mut self) {}
    fn on_drag_start(&mut self) {}
    fn on_drag(&mut self, _delta: (i32, i32)) {}
    fn on_click(&mut self) {}

    fn is_hit(&self, pixel: (i32, i32)) -> bool {
        let size = self.get_size();
        pixel.0 >= 0 && pixel.1 >= 0 && pixel.0 <= size.0 && pixel.1 <= size.1
    }
    fn trace_hit(&self, pixel: (i32, i32)) -> Option<Rcrc<dyn Widget>> {
        for child in self.borrow_children() {
            let child_ref = child.borrow();
            let child_pos = child_ref.get_pos();
            let child_pixel = (pixel.0 - child_pos.0, pixel.1 - child_pos.1);
            if child_ref.is_hit(child_pixel) {
                return Some(trace_hit(child, child_pixel));
            }
        }
        None
    }

    fn draw_impl(&self, g: &mut GrahpicsWrapper);
    fn draw(&self, g: &mut GrahpicsWrapper) {
        g.push_state();
        let pos = Widget::get_pos(self);
        g.apply_offset(pos.0, pos.1);
        self.draw_impl(g);
        for child in self.borrow_children() {
            child.borrow().draw(g);
        }
        g.pop_state();
    }
}