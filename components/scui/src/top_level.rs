use crate::{
    MaybeMouseBehavior, MouseMods, Renderer, TextField, Vec2D, Widget, DOUBLE_CLICK_RANGE,
    DOUBLE_CLICK_TIME, DRAG_DEADZONE,
};
use shared_util::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;

pub trait GuiInterfaceProvider<State, DT> {
    fn provide_gui_interface(&self) -> Rc<GuiInterface<State, DT>>;
}

pub struct PlaceholderGuiState;

// Stuff that the GUI keeps track of for normal operations, e.g. when the last mouse press was.
struct InternalGuiState<DT> {
    mouse_behavior: MaybeMouseBehavior<DT>,
    click_pos: Vec2D,
    mouse_pos: Vec2D,
    mouse_down: bool,
    dragged: bool,
    last_click: Instant,
    last_click_pos: Vec2D,
    focused_text_field: Option<Rcrc<TextField>>,
}

impl<DT> InternalGuiState<DT> {
    // Defocuses the current text field, if one is focused.
    fn defocus_text_field(&mut self) {
        if let Some(field) = self.focused_text_field.take() {
            let mut field = field.borrow_mut();
            field.focused = false;
            (field.on_defocus)(&field.text);
        }
    }
}

pub struct GuiInterface<State, DT> {
    pub state: RefCell<State>,
    internal_state: RefCell<InternalGuiState<DT>>,
}

impl<State, DT> GuiInterface<State, DT> {
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

    pub fn get_mouse_pos(&self) -> Vec2D {
        self.internal_state.borrow().mouse_pos
    }
}

impl<State, DT> GuiInterfaceProvider<State, DT> for Rc<GuiInterface<State, DT>> {
    fn provide_gui_interface(&self) -> Rc<GuiInterface<State, DT>> {
        Rc::clone(self)
    }
}

pub struct Gui<State, DT, RootWidget> {
    interface: Rc<GuiInterface<State, DT>>,
    root: RootWidget,
}

impl<State, DT, RW> Gui<State, DT, RW> {
    pub fn new<R, D, B>(state: State, root_builder: B) -> Self
    where
        R: Renderer,
        RW: Widget<R, D>,
        B: FnOnce(&Rc<GuiInterface<State, DT>>) -> RW,
    {
        let interface = Rc::new(GuiInterface::new(state));
        let root = root_builder(&interface);
        Self { interface, root }
    }

    pub fn on_mouse_down<R>(&self, mods: &MouseMods)
    where
        R: Renderer,
        RW: Widget<R, DT>,
    {
        let mut internal = self.interface.internal_state.borrow_mut();
        internal.defocus_text_field();
        internal.mouse_down = true;
        internal.dragged = false;
        let mouse_pos = internal.mouse_pos;
        internal.click_pos = mouse_pos;
        internal.mouse_behavior = self.root.get_mouse_behavior(mouse_pos, mods);
    }

    pub fn on_mouse_move<R, D>(&self, new_pos: Vec2D, mods: &MouseMods)
    where
        R: Renderer,
        RW: Widget<R, D>,
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
        RW: Widget<R, DT>,
    {
        let mut internal = self.interface.internal_state.borrow_mut();
        if let Some(behavior) = internal.mouse_behavior.take() {
            if internal.dragged {
                let drop_target = self.root.get_drop_target(internal.mouse_pos);
                drop(internal);
                behavior.on_drop(drop_target);
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

    pub fn on_scroll<R, D>(&self, delta: f32)
    where
        R: Renderer,
        RW: Widget<R, D>,
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

    pub fn on_removed<R, D>(&self)
    where
        R: Renderer,
        RW: Widget<R, D>,
    {
        self.root.on_removed();
    }

    pub fn draw<R, D>(&self, renderer: &mut R)
    where
        R: Renderer,
        RW: Widget<R, D>,
    {
        self.root.draw(renderer);
    }
}
