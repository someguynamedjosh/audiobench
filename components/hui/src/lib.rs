use std::cell::RefCell;
use std::marker::PhantomData;
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
    fn provide(&self) -> W;
}

pub trait GuiInterfaceProvider<State> {
    fn provide_gui_interface(&self) -> Rc<GuiInterface<State>>;
}

pub trait Renderer {
    fn push_state(&mut self);
    fn pop_state(&mut self);
    fn translate(&mut self, offset: Pos2D);
}

pub struct PlaceholderRenderer;
impl Renderer for PlaceholderRenderer {
    fn push_state(&mut self) {}
    fn pop_state(&mut self) {}
    fn translate(&mut self, offset: Pos2D) {}
}

pub struct PlaceholderGuiState;

pub struct GuiInterface<State> {
    state: State,
}

impl<State> GuiInterface<State> {
    fn new(state: State) -> Self {
        Self { state }
    }
}

impl<State> GuiInterfaceProvider<State> for Rc<GuiInterface<State>> {
    fn provide_gui_interface(&self) -> Rc<GuiInterface<State>> {
        Rc::clone(self)
    }
}

pub struct Gui<State, R: Renderer, RootWidget: Widget<R>> {
    interface: Rc<GuiInterface<State>>,
    root: RefCell<RootWidget>,
    _r: PhantomData<R>,
}

impl<State, R: Renderer, RW: Widget<R>> Gui<State, R, RW> {
    pub fn new(
        state: State,
        root_builder: impl FnOnce(&Rc<GuiInterface<State>>) -> RW,
    ) -> Rc<Self> {
        let interface = Rc::new(GuiInterface::new(state));
        let root = root_builder(&interface);
        Rc::new(Self {
            interface,
            root: RefCell::new(root),
            _r: PhantomData,
        })
    }
}

impl<State, R: Renderer, RW: Widget<R>> Drop for Gui<State, R, RW> {
    fn drop(&mut self) {
        self.root.borrow_mut().on_removed();
    }
}

hui_macros::widget! {
    pub Test
}

impl Test {
    fn new(parent: &impl TestParent, pos: Pos2D) -> Rc<Self> {
        Rc::new(Self::create(parent, TestState { pos }))
    }

    fn get_mouse_behavior(self: &Rc<Self>, pos: Pos2D) -> MaybeMouseBehavior {
        // let child = Self::new(self, Pos2D::new(0.0, 0.0));
        None
    }

    fn draw(self: &Rc<Self>, renderer: &mut PlaceholderRenderer) {
        self.draw_children(renderer);
    }
}

fn test() {
    let gui = Gui::new(
        PlaceholderGuiState,
        |gui| Test::new(gui, Pos2D::new(0.0, 0.0))
    );
}
