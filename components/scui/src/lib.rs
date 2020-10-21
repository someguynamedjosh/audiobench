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
pub struct Pos2D {
    pub x: f32,
    pub y: f32,
}

impl Pos2D {
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
}

impl Add for Pos2D {
    type Output = Self;

    fn add(self: Self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl Sub for Pos2D {
    type Output = Self;

    fn sub(self: Self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
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

impl From<(i32, i32)> for Pos2D {
    fn from(other: (i32, i32)) -> Pos2D {
        Self {
            x: other.0 as f32,
            y: other.1 as f32,
        }
    }
}

pub trait MouseBehavior {
    fn on_drag(&mut self, delta: Pos2D, mods: &MouseMods);
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
    fn get_pos(&self) -> Pos2D;
    fn get_mouse_behavior(&self, pos: Pos2D, mods: &MouseMods) -> Option<Box<dyn MouseBehavior>>;
    fn draw(&self, renderer: &mut R);
    fn on_scroll(&self, delta: f32);
    fn on_removed(&self);
}

/// This is the trait that should be implemented by people creating widgets. It is a way to provide
/// default implementations while still letting the programmer override them.
pub trait WidgetImpl<'r, R: Renderer<'r>> {
    fn get_mouse_behavior(self: &Rc<Self>, _pos: Pos2D, _mods: &MouseMods) -> Option<Box<dyn MouseBehavior>> {
        None
    }
    fn draw(self: &Rc<Self>, _renderer: &mut R) { }
    fn on_scroll(&self, _delta: f32) { }
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
    fn translate(&mut self, offset: Pos2D);
}

pub struct PlaceholderRenderer;
impl<'r> Renderer<'r> for PlaceholderRenderer {
    fn push_state(&mut self) {}
    fn pop_state(&mut self) {}
    fn translate(&mut self, _offset: Pos2D) {}
}

pub struct PlaceholderGuiState;

pub struct GuiInterface<State> {
    pub state: RefCell<State>,
    internal_state: RefCell<InternalGuiState>,
}

// Stuff that the GUI keeps track of for normal operations, e.g. when the last mouse press was.
struct InternalGuiState {
    mouse_behavior: MaybeMouseBehavior,
    click_pos: Pos2D,
    mouse_pos: Pos2D,
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
                click_pos: Pos2D::zero(),
                mouse_pos: Pos2D::zero(),
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

pub struct Gui<'r, State, R: Renderer<'r>, RootWidget: Widget<'r, R>> {
    interface: Rc<GuiInterface<State>>,
    root: RootWidget,
    _r: PhantomData<&'r R>,
}

impl<'r, State, R: Renderer<'r>, RW: Widget<'r, R>> Gui<'r, State, R, RW> {
    pub fn new(state: State, root_builder: impl FnOnce(&Rc<GuiInterface<State>>) -> RW) -> Self {
        let interface = Rc::new(GuiInterface::new(state));
        let root = root_builder(&interface);
        Self {
            interface,
            root,
            _r: PhantomData,
        }
    }

    pub fn on_mouse_down(&self, mods: &MouseMods) {
        let mut internal = self.interface.internal_state.borrow_mut();
        internal.mouse_down = true;
        let mouse_pos = internal.mouse_pos;
        internal.click_pos = mouse_pos;
        internal.mouse_behavior = self.root.get_mouse_behavior(mouse_pos, mods);
    }

    pub fn on_mouse_move(&self, new_pos: Pos2D, mods: &MouseMods) {
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

    pub fn on_mouse_up(&self) {
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

    pub fn on_scroll(&self, delta: f32) {
        self.root.on_scroll(delta);
    }
    
    // pub fn on_key_press()
}

impl<'r, State, R: Renderer<'r>, RW: Widget<'r, R>> Drop for Gui<'r, State, R, RW> {
    fn drop(&mut self) {
        self.root.on_removed();
    }
}
