use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone, Copy)]
pub struct Pos2D {
    pub x: f32,
    pub y: f32,
}

impl Pos2D {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

impl From<(f32, f32)> for Pos2D {
    fn from(other: (f32, f32)) -> Pos2D {
        Self {
            x: other.0,
            y: other.1,
        }
    }
}

pub trait MouseBehavior {
    fn on_drag(&mut self);
    fn on_drop(self);
    fn on_click(self);
    fn on_double_click(self);
}

pub type MaybeMouseBehavior = Option<Box<dyn MouseBehavior>>;

pub trait Widget<R: Renderer> {
    fn get_pos(&self) -> Pos2D;
    fn get_mouse_behavior(&self, pos: Pos2D) -> Option<Box<dyn MouseBehavior>>;
    fn draw(&self, renderer: &mut R);
    fn on_removed(&self);
}

pub trait WidgetProvider<R: Renderer, W: Widget<R>> {
    fn provide(self: &Self) -> W;
}

pub trait Renderer {
    fn push_state(&mut self);
    fn pop_state(&mut self);
    fn translate(&mut self, offset: Pos2D);
}

pub struct PlaceholderRenderer;
impl Renderer for PlaceholderRenderer {
    fn push_state(&mut self) { }
    fn pop_state(&mut self) { }
    fn translate(&mut self, offset: Pos2D) { }
}

hui_macros::widget! {
    pub BoxWidget
    Children {
        c: Option<Rc<Asdf>>
    }
}

impl BoxWidget {
    fn new(parent: &impl BoxWidgetParent, pos: Pos2D) -> Rc<Self> {
        Rc::new(Self::create(parent, BoxWidgetState { pos }))
    }

    fn get_mouse_behavior(self: &Rc<Self>, pos: Pos2D) -> MaybeMouseBehavior {
        None
    }

    fn draw(self: &Rc<Self>, renderer: &mut PlaceholderRenderer) {

    }
}

hui_macros::widget! {
    pub Asdf
    Parents {
        asdf: Rc<Asdf>,
        parent2: Rc<BoxWidget>,
    }
    Children {
        c: Vec<Rc<Asdf>>,
    }
}

impl Asdf {
    fn new(parent: &impl AsdfParent, pos: Pos2D) -> Rc<Self> {
        Rc::new(Self::create(parent, AsdfState { pos }))
    }

    fn get_mouse_behavior(self: &Rc<Self>, pos: Pos2D) -> MaybeMouseBehavior {
        let child = Self::new(self, Pos2D::new(0.0, 0.0));
        None
    }

    fn draw(self: &Rc<Self>, renderer: &mut PlaceholderRenderer) {
        self.draw_children(renderer);
    }
}
