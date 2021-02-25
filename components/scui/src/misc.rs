use crate::Vec2D;
use std::ops::{Deref, DerefMut};

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

pub struct TextField {
    pub text: String,
    pub(crate) focused: bool,
    pub(crate) on_defocus: Box<dyn Fn(&str)>,
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
