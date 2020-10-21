use std::cell::RefCell;
use std::marker::PhantomData;
use std::ops::{Add, Sub};
use std::rc::Rc;
use std::time::{Duration, Instant};

pub use scui_macros::*;

/// How many pixels the mouse must move across while being held down for dragging to start.
pub const DRAG_DEADZONE: f32 = 4.0;
/// The maximum amount of time between two clicks for the event to be considered a double-click.
pub const DOUBLE_CLICK_TIME: Duration = Duration::from_millis(500);

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Vec2D {
    pub x: f32,
    pub y: f32,
}

impl Vec2D {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn new_int(x: i32, y: i32) -> Self {
        Self {
            x: x as f32,
            y: y as f32,
        }
    }

    pub fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }

    /// Returns length of this vector using pythagorean distance.
    pub fn length(self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    pub fn inside(self, other: Self) -> bool {
        self.x <= other.x && self.y <= other.y
    }
}

impl Add for Vec2D {
    type Output = Self;

    fn add(self: Self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl Sub for Vec2D {
    type Output = Self;

    fn sub(self: Self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl From<(f32, f32)> for Vec2D {
    fn from(other: (f32, f32)) -> Vec2D {
        Self {
            x: other.0,
            y: other.1,
        }
    }
}

impl From<(i32, i32)> for Vec2D {
    fn from(other: (i32, i32)) -> Vec2D {
        Self {
            x: other.0 as f32,
            y: other.1 as f32,
        }
    }
}

pub trait MouseBehavior {
    fn on_drag(&mut self, delta: Vec2D, mods: &MouseMods);
    fn on_drop(self: Box<Self>);
    fn on_click(self: Box<Self>);
    fn on_double_click(self: Box<Self>);
}

pub type MaybeMouseBehavior = Option<Box<dyn MouseBehavior>>;

pub struct MouseMods {
    pub right_click: bool,
    pub snap: bool,
    pub precise: bool,
}

pub trait Widget<'r, R: Renderer<'r>> {
    fn get_pos(&self) -> Vec2D;
    fn get_size(&self) -> Vec2D;
    fn get_mouse_behavior(&self, pos: Vec2D, mods: &MouseMods) -> Option<Box<dyn MouseBehavior>>;
    fn draw(&self, renderer: &mut R);
    fn on_scroll(&self, delta: f32);
    fn on_removed(&self);
}

/// This is the trait that should be implemented by people creating widgets. It is a way to provide
/// default implementations while still letting the programmer override them.
pub trait WidgetImpl<'r, R: Renderer<'r>> {
    fn get_size(self: &Rc<Self>) -> Vec2D {
        Vec2D::zero()
    }

    fn get_mouse_behavior(
        self: &Rc<Self>,
        _pos: Vec2D,
        _mods: &MouseMods,
    ) -> Option<Box<dyn MouseBehavior>> {
        None
    }

    fn draw(self: &Rc<Self>, _renderer: &mut R) {}

    fn on_scroll(self: &Rc<Self>, _delta: f32) {}
}

pub trait WidgetProvider<'r, R: Renderer<'r>, W: Widget<'r, R>> {
    fn provide(&self) -> W;
}

pub trait GuiInterfaceProvider<State> {
    fn provide_gui_interface(&self) -> Rc<GuiInterface<State>>;
}

pub trait Renderer<'r> {
    fn push_state(&mut self);
    fn pop_state(&mut self);
    fn translate(&mut self, offset: Vec2D);
}

pub struct PlaceholderRenderer;
impl<'r> Renderer<'r> for PlaceholderRenderer {
    fn push_state(&mut self) {}
    fn pop_state(&mut self) {}
    fn translate(&mut self, _offset: Vec2D) {}
}

pub struct PlaceholderGuiState;

pub struct GuiInterface<State> {
    pub state: RefCell<State>,
    internal_state: RefCell<InternalGuiState>,
}

// Stuff that the GUI keeps track of for normal operations, e.g. when the last mouse press was.
struct InternalGuiState {
    mouse_behavior: MaybeMouseBehavior,
    click_pos: Vec2D,
    mouse_pos: Vec2D,
    mouse_down: bool,
    dragged: bool,
    last_click: Instant,
    // focused_text_field: Option<Rcrc<gui::ui_widgets::TextField>>,
}

impl<State> GuiInterface<State> {
    fn new(state: State) -> Self {
        Self {
            state: RefCell::new(state),
            internal_state: RefCell::new(InternalGuiState {
                mouse_behavior: None,
                click_pos: Vec2D::zero(),
                mouse_pos: Vec2D::zero(),
                mouse_down: false,
                dragged: false,
                last_click: Instant::now(),
            }),
        }
    }
}

impl<State> GuiInterfaceProvider<State> for Rc<GuiInterface<State>> {
    fn provide_gui_interface(&self) -> Rc<GuiInterface<State>> {
        Rc::clone(self)
    }
}

pub struct Gui<State, RootWidget> {
    interface: Rc<GuiInterface<State>>,
    root: RootWidget,
}

impl<State, RW> Gui<State, RW> {
    pub fn new<'r, R, B>(state: State, root_builder: B) -> Self
    where
        R: Renderer<'r>,
        RW: Widget<'r, R>,
        B: FnOnce(&Rc<GuiInterface<State>>) -> RW,
    {
        let interface = Rc::new(GuiInterface::new(state));
        let root = root_builder(&interface);
        Self { interface, root }
    }

    pub fn on_mouse_down<'r, R>(&self, mods: &MouseMods)
    where
        R: Renderer<'r>,
        RW: Widget<'r, R>,
    {
        let mut internal = self.interface.internal_state.borrow_mut();
        internal.mouse_down = true;
        let mouse_pos = internal.mouse_pos;
        internal.click_pos = mouse_pos;
        internal.mouse_behavior = self.root.get_mouse_behavior(mouse_pos, mods);
    }

    pub fn on_mouse_move<'r, R>(&self, new_pos: Vec2D, mods: &MouseMods)
    where
        R: Renderer<'r>,
        RW: Widget<'r, R>,
    {
        let mut internal = self.interface.internal_state.borrow_mut();
        internal.mouse_pos = new_pos;
        if internal.mouse_down {
            let delta = new_pos - internal.click_pos;
            if !internal.dragged {
                if delta.length() > DRAG_DEADZONE {
                    internal.dragged = true;
                }
            }
            if internal.dragged {
                if let Some(behavior) = &mut internal.mouse_behavior {
                    behavior.on_drag(delta, mods);
                    internal.click_pos = new_pos;
                }
            }
        }
    }

    pub fn on_mouse_up<'r, R>(&self)
    where
        R: Renderer<'r>,
        RW: Widget<'r, R>,
    {
        let mut internal = self.interface.internal_state.borrow_mut();
        if let Some(behavior) = internal.mouse_behavior.take() {
            if internal.dragged {
                // let drop_target = self.root.get_drop_target(internal.mouse_pos);
                // behavior.on_drop(drop_target);
                behavior.on_drop();
            } else {
                if internal.last_click.elapsed() < DOUBLE_CLICK_TIME {
                    behavior.on_double_click()
                } else {
                    internal.last_click = Instant::now();
                    behavior.on_click()
                }
            }
        }
    }

    pub fn on_scroll<'r, R>(&self, delta: f32)
    where
        R: Renderer<'r>,
        RW: Widget<'r, R>,
    {
        self.root.on_scroll(delta);
    }
    // pub fn on_key_press()

    pub fn on_removed<'r, R>(&self)
    where
        R: Renderer<'r>,
        RW: Widget<'r, R>,
    {
        self.root.on_removed();
    }

    pub fn draw<'r, R>(&self, renderer: &mut R)
    where
        R: Renderer<'r>,
        RW: Widget<'r, R>,
    {
        self.root.draw(renderer);
    }
}
