use crate::{MaybeMouseBehavior, MouseMods, Renderer, Vec2D};
use shared_util::prelude::*;

/// This trait represents a widget. This is the trait you should use if you want to interact with
/// another widget. You should not implement this trait yourself, instead implement WidgetImpl on
/// a struct created with the widget! macro.
pub trait Widget<R: Renderer, DT> {
    fn get_pos(&self) -> Vec2D;
    fn get_size(&self) -> Vec2D;
    fn get_mouse_behavior(&self, pos: Vec2D, mods: &MouseMods) -> MaybeMouseBehavior<DT>;
    fn get_drop_target(&self, pos: Vec2D) -> Option<DT>;
    fn on_scroll(&self, pos: Vec2D, delta: f32) -> Option<()>;
    fn on_hover(&self, pos: Vec2D) -> Option<()>;
    fn draw(&self, renderer: &mut R);
    fn on_removed(&self);
}

// Unfortunately we can't make a blanket impl to implement Widget for T: WidgetImpl because that
// would prevent us from implementing for Box<>, which causes other problems. Instead, that code
// is manually repeated for every created widget by the widget! macro.

/// This allows storing widgets dynamically in boxes.
impl<R: Renderer, DT> Widget<R, DT> for Box<dyn Widget<R, DT>> {
    fn get_pos(&self) -> Vec2D {
        (**self).get_pos()
    }
    fn get_size(&self) -> Vec2D {
        (**self).get_size()
    }
    fn get_mouse_behavior(&self, pos: Vec2D, mods: &MouseMods) -> MaybeMouseBehavior<DT> {
        (**self).get_mouse_behavior(pos, mods)
    }
    fn get_drop_target(&self, pos: Vec2D) -> Option<DT> {
        (**self).get_drop_target(pos)
    }
    fn on_scroll(&self, pos: Vec2D, delta: f32) -> Option<()> {
        (**self).on_scroll(pos, delta)
    }
    fn on_hover(&self, pos: Vec2D) -> Option<()> {
        (**self).on_hover(pos)
    }
    fn draw(&self, renderer: &mut R) {
        (**self).draw(renderer)
    }
    fn on_removed(&self) {
        (**self).on_removed()
    }
}

/// This trait provides default code for WidgetImpl functions that will be used
/// if the user does not override those functions. This is necessary because the
/// code is "different" for every generated widget because it uses private
/// methods of the widget. If you are creating your own widgets you should not
/// be using this trait.
pub trait WidgetImplDefaults<R: Renderer, DT> {
    fn get_mouse_behavior_default(
        self: &Rc<Self>,
        pos: Vec2D,
        mods: &MouseMods,
    ) -> MaybeMouseBehavior<DT>;
    fn get_drop_target_default(self: &Rc<Self>, pos: Vec2D) -> Option<DT>;
    fn on_scroll_default(self: &Rc<Self>, pos: Vec2D, delta: f32) -> Option<()>;
    fn on_hover_default(self: &Rc<Self>, pos: Vec2D) -> Option<()>;
    fn draw_default(self: &Rc<Self>, renderer: &mut R);
}

/// This is the trait you should implement if you are creating a widget. Most functions can be
/// omitted to use a sensible default in its place. (E.G. on_scroll_impl's default will 
/// automatically call self.on_scroll_children() for you.) If you provide a custom implementation of
/// a function, make sure to call the *_children() variant at some point during the function or
/// manually perform the job that the *_children() function would perform.
pub trait WidgetImpl<R: Renderer, DT>: WidgetImplDefaults<R, DT> {
    fn get_pos_impl(self: &Rc<Self>) -> Vec2D;
    fn get_size_impl(self: &Rc<Self>) -> Vec2D;

    fn get_mouse_behavior_impl(
        self: &Rc<Self>,
        pos: Vec2D,
        mods: &MouseMods,
    ) -> MaybeMouseBehavior<DT> {
        self.get_mouse_behavior_default(pos, mods)
    }

    fn get_drop_target_impl(self: &Rc<Self>, pos: Vec2D) -> Option<DT> {
        self.get_drop_target_default(pos)
    }

    fn on_scroll_impl(self: &Rc<Self>, pos: Vec2D, delta: f32) -> Option<()> {
        self.on_scroll_default(pos, delta)
    }

    fn on_hover_impl(self: &Rc<Self>, pos: Vec2D) -> Option<()> {
        self.on_hover_default(pos)
    }

    fn draw_impl(self: &Rc<Self>, renderer: &mut R) {
        self.draw_default(renderer)
    }
}

pub trait WidgetProvider<R: Renderer, D, W: Widget<R, D>> {
    fn provide(&self) -> W;
}
