use shared_util::prelude::*;
use std::cell::RefCell;
use std::ops::{Add, Deref, DerefMut, Div, Mul, Sub};
use std::rc::Rc;
use std::time::{Duration, Instant};

pub use scui_macros::*;

/// How many pixels the mouse must move across while being held down for dragging to start.
pub const DRAG_DEADZONE: f32 = 4.0;
/// The maximum amount of time between two clicks for the event to be considered a double-click.
pub const DOUBLE_CLICK_TIME: Duration = Duration::from_millis(400);
/// If the user moves the mouse at least this much between two clicks, it will not be considered
/// a double-click.
pub const DOUBLE_CLICK_RANGE: f32 = 4.0;

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Vec2D {
    pub x: f32,
    pub y: f32,
}

impl Vec2D {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub const fn new_int(x: i32, y: i32) -> Self {
        Self {
            x: x as f32,
            y: y as f32,
        }
    }

    pub const fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }

    /// Returns length of this vector using pythagorean distance.
    pub fn length(self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    pub fn inside(self, other: Self) -> bool {
        self.x >= 0.0 && self.y >= 0.0 && self.x <= other.x && self.y <= other.y
    }
}

impl<T: Into<Vec2D>> Add<T> for Vec2D {
    type Output = Self;

    fn add(self: Self, other: T) -> Self {
        let other = other.into();
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl<T: Into<Vec2D>> Sub<T> for Vec2D {
    type Output = Self;

    fn sub(self: Self, other: T) -> Self {
        let other = other.into();
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl<T: Into<Vec2D>> Mul<T> for Vec2D {
    type Output = Self;

    fn mul(self: Self, other: T) -> Self {
        let other = other.into();
        Self {
            x: self.x * other.x,
            y: self.y * other.y,
        }
    }
}

impl<T: Into<Vec2D>> Div<T> for Vec2D {
    type Output = Self;

    fn div(self: Self, other: T) -> Self {
        let other = other.into();
        Self {
            x: self.x / other.x,
            y: self.y / other.y,
        }
    }
}

impl From<f32> for Vec2D {
    fn from(other: f32) -> Vec2D {
        Self { x: other, y: other }
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

impl From<i32> for Vec2D {
    fn from(other: i32) -> Vec2D {
        Self {
            x: other as f32,
            y: other as f32,
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
    fn on_drag(&mut self, _delta: Vec2D, _mods: &MouseMods) {}
    fn on_drop(self: Box<Self>) {}
    fn on_click(self: Box<Self>) {}
    fn on_double_click(self: Box<Self>) {}
}

pub struct OnClickBehavior<F: FnOnce()> {
    action: F,
}

impl<F: FnOnce() + 'static> OnClickBehavior<F> {
    pub fn wrap(action: F) -> MaybeMouseBehavior {
        Some(Box::new(Self { action }))
    }
}

impl<F: FnOnce() + 'static> From<F> for OnClickBehavior<F> {
    fn from(other: F) -> Self {
        Self { action: other }
    }
}

impl<F: FnOnce()> MouseBehavior for OnClickBehavior<F> {
    fn on_click(self: Box<Self>) {
        (self.action)();
    }
}

pub type MaybeMouseBehavior = Option<Box<dyn MouseBehavior>>;

pub struct MouseMods {
    pub right_click: bool,
    pub snap: bool,
    pub precise: bool,
}

pub trait Widget<R: Renderer> {
    fn get_pos(&self) -> Vec2D;
    fn get_size(&self) -> Vec2D;
    fn get_mouse_behavior(&self, pos: Vec2D, mods: &MouseMods) -> Option<Box<dyn MouseBehavior>>;
    fn on_scroll(&self, pos: Vec2D, delta: f32) -> bool;
    fn on_hover(&self, pos: Vec2D) -> bool;
    fn draw(&self, renderer: &mut R);
    fn on_removed(&self);
}

/// This is the trait that should be implemented by people creating widgets. It is a way to provide
/// default implementations while still letting the programmer override them.
pub trait WidgetImpl<R: Renderer> {
    fn get_size(self: &Rc<Self>) -> Vec2D;

    fn get_mouse_behavior(
        self: &Rc<Self>,
        _pos: Vec2D,
        _mods: &MouseMods,
    ) -> Option<Box<dyn MouseBehavior>> {
        None
    }

    fn on_hover(self: &Rc<Self>, _pos: Vec2D) -> bool {
        false
    }

    fn on_scroll(self: &Rc<Self>, _pos: Vec2D, _delta: f32) -> bool {
        false
    }

    fn draw(self: &Rc<Self>, _renderer: &mut R) {}
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
    fn translate(&mut self, offset: Vec2D);
}

pub struct PlaceholderRenderer;
impl Renderer for PlaceholderRenderer {
    fn push_state(&mut self) {}
    fn pop_state(&mut self) {}
    fn translate(&mut self, _offset: Vec2D) {}
}

pub struct PlaceholderGuiState;

// Stuff that the GUI keeps track of for normal operations, e.g. when the last mouse press was.
struct InternalGuiState {
    mouse_behavior: MaybeMouseBehavior,
    click_pos: Vec2D,
    mouse_pos: Vec2D,
    mouse_down: bool,
    dragged: bool,
    last_click: Instant,
    last_click_pos: Vec2D,
    focused_text_field: Option<Rcrc<TextField>>,
}

impl InternalGuiState {
    // Defocuses the current text field, if one is focused.
    fn defocus_text_field(&mut self) {
        if let Some(field) = self.focused_text_field.take() {
            let mut field = field.borrow_mut();
            field.focused = false;
            (field.on_defocus)(&field.text);
        }
    }
}

pub struct GuiInterface<State> {
    pub state: RefCell<State>,
    internal_state: RefCell<InternalGuiState>,
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
                last_click_pos: Vec2D::zero(),
                focused_text_field: None,
            }),
        }
    }

    pub fn focus_text_field(&self, field: &Rcrc<TextField>) {
        let mut internal = self.internal_state.borrow_mut();
        internal.defocus_text_field();
        field.borrow_mut().focused = true;
        internal.focused_text_field = Some(Rc::clone(field));
    }
}

impl<State> GuiInterfaceProvider<State> for Rc<GuiInterface<State>> {
    fn provide_gui_interface(&self) -> Rc<GuiInterface<State>> {
        Rc::clone(self)
    }
}

pub struct TextField {
    pub text: String,
    focused: bool,
    on_defocus: Box<dyn Fn(&str)>,
}

impl TextField {
    pub fn new<S: Into<String>>(initial_contents: S, on_defocus: Box<dyn Fn(&str)>) -> Self {
        Self {
            text: initial_contents.into(),
            focused: false,
            on_defocus,
        }
    }

    pub fn is_focused(&self) -> bool {
        self.focused
    }
}

/// Acts like a more transparent version of Option<>. It automatically derefs to the templated type,
/// panicking if it is None. You should use it with the same semantics that you would use for a
/// plain variable of type C. Example:
/// ```
/// let field: ChildHolder<i32>;
/// field = 123.into();
/// println!("{}", field);
/// let value = 321 + field;
/// ```
pub struct ChildHolder<C>(Option<C>);

impl<C> Deref for ChildHolder<C> {
    type Target = C;

    fn deref(&self) -> &Self::Target {
        // Print nicer message if we are debug build.
        debug_assert!(
            self.0.is_some(),
            "ChildHolder must be assigned a value before use."
        );
        self.0.as_ref().unwrap()
    }
}

impl<C> DerefMut for ChildHolder<C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // Print nicer message if we are debug build.
        debug_assert!(
            self.0.is_some(),
            "ChildHolder must be assigned a value before use."
        );
        self.0.as_mut().unwrap()
    }
}

impl<C> Default for ChildHolder<C> {
    fn default() -> Self {
        Self(None)
    }
}

impl<'a, C> IntoIterator for &'a ChildHolder<C> {
    type Item = <&'a Option<C> as IntoIterator>::Item;
    type IntoIter = <&'a Option<C> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        (&self.0).into_iter()
    }
}

impl<C> From<C> for ChildHolder<C> {
    fn from(other: C) -> Self {
        Self(Some(other))
    }
}

pub struct Gui<State, RootWidget> {
    interface: Rc<GuiInterface<State>>,
    root: RootWidget,
}

impl<State, RW> Gui<State, RW> {
    pub fn new<R, B>(state: State, root_builder: B) -> Self
    where
        R: Renderer,
        RW: Widget<R>,
        B: FnOnce(&Rc<GuiInterface<State>>) -> RW,
    {
        let interface = Rc::new(GuiInterface::new(state));
        let root = root_builder(&interface);
        Self { interface, root }
    }

    pub fn on_mouse_down<R>(&self, mods: &MouseMods)
    where
        R: Renderer,
        RW: Widget<R>,
    {
        let mut internal = self.interface.internal_state.borrow_mut();
        internal.defocus_text_field();
        internal.mouse_down = true;
        internal.dragged = false;
        let mouse_pos = internal.mouse_pos;
        internal.click_pos = mouse_pos;
        internal.mouse_behavior = self.root.get_mouse_behavior(mouse_pos, mods);
    }

    pub fn on_mouse_move<R>(&self, new_pos: Vec2D, mods: &MouseMods)
    where
        R: Renderer,
        RW: Widget<R>,
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

    pub fn on_mouse_up<R>(&self)
    where
        R: Renderer,
        RW: Widget<R>,
    {
        let mut internal = self.interface.internal_state.borrow_mut();
        if let Some(behavior) = internal.mouse_behavior.take() {
            if internal.dragged {
                // let drop_target = self.root.get_drop_target(internal.mouse_pos);
                // behavior.on_drop(drop_target);
                drop(internal);
                behavior.on_drop();
            } else {
                let time = internal.last_click.elapsed();
                let distance = (internal.last_click_pos - internal.click_pos).length();
                if time <= DOUBLE_CLICK_TIME && distance <= DOUBLE_CLICK_RANGE {
                    drop(internal);
                    behavior.on_double_click()
                } else {
                    internal.last_click = Instant::now();
                    internal.last_click_pos = internal.click_pos;
                    drop(internal);
                    behavior.on_click()
                }
            }
        }
    }

    pub fn on_scroll<R>(&self, delta: f32)
    where
        R: Renderer,
        RW: Widget<R>,
    {
        let pos = self.interface.internal_state.borrow().mouse_pos;
        self.root.on_scroll(pos, delta);
    }

    pub fn on_key_press(&self, key: char) {
        // For some reason JUCE gives \r instead of \n.
        let key = if key == '\r' { '\n' } else { key };
        let mut internal = self.interface.internal_state.borrow_mut();
        if let Some(field) = &mut internal.focused_text_field {
            let mut field = field.borrow_mut();
            match key {
                '\x08' | '\x7F' => {
                    // ASCII delete or backspace.
                    if field.text.len() > 0 {
                        let last = field.text.len() - 1;
                        field.text = field.text[..last].to_owned();
                    }
                }
                '\x1B' | '\x0A' => {
                    // Escape or enter
                    drop(field);
                    internal.defocus_text_field();
                }
                _ => {
                    field.text.push(key);
                }
            }
        }
    }

    pub fn on_removed<R>(&self)
    where
        R: Renderer,
        RW: Widget<R>,
    {
        self.root.on_removed();
    }

    pub fn draw<R>(&self, renderer: &mut R)
    where
        R: Renderer,
        RW: Widget<R>,
    {
        self.root.draw(renderer);
    }
}
