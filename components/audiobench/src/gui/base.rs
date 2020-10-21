use crate::engine::parts as ep;
use crate::gui;
use crate::gui::constants::*;
use crate::gui::graphics::{GrahpicsWrapper, HAlign, VAlign};
use crate::registry::save_data::Patch;
use crate::registry::Registry;
use crate::scui_config::Renderer;
use enumflags2::BitFlags;
use scui::{Vec2D, WidgetImpl};
use shared_util::prelude::*;

pub use scui::MouseMods;

#[derive(BitFlags, Copy, Clone)]
#[repr(u8)]
pub enum InteractionHint {
    LeftClick = 0x1,
    RightClick = 0x2,
    Scroll = 0x40,
    LeftClickAndDrag = 0x4,
    DoubleClick = 0x8,
    PrecisionModifier = 0x10,
    SnappingModifier = 0x20,
}

#[derive(Clone)]
pub struct Tooltip {
    pub text: String,
    pub interaction: BitFlags<InteractionHint>,
}

impl Default for Tooltip {
    fn default() -> Tooltip {
        Tooltip {
            text: "".to_owned(),
            interaction: Default::default(),
        }
    }
}

#[derive(Clone)]
pub struct Status {
    pub text: String,
    pub color: (u8, u8, u8),
}

impl Status {
    fn success(text: String) -> Self {
        Self {
            text,
            color: COLOR_SUCCESS,
        }
    }

    fn error(text: String) -> Self {
        Self {
            text,
            color: COLOR_ERROR,
        }
    }
}

pub struct GuiState {
    // pub registry: Rcrc<Registry>,
    pub status: Option<Status>,
    pub tooltip: Tooltip,
}

scui::widget! {
    pub Root
    Children {
        // menu_bar: Option<Rc<gui::ui_widgets::MenuBar>>,
    }
}

impl Root {
    fn new(parent: &impl RootParent) -> Rc<Self> {
        Rc::new(Self::create(parent, RootState { pos: Vec2D::zero() }))
    }
}

impl<'r> WidgetImpl<'r, Renderer<'r>> for Root {
    fn get_size(self: &Rc<Self>) -> Vec2D {
        (ROOT_WIDTH, ROOT_HEIGHT).into()
    }

    fn draw(self: &Rc<Self>, renderer: &mut Renderer<'r>) {
        renderer.set_color(&COLOR_BG0);
        renderer.fill_rect(0.0, 0.0, ROOT_WIDTH, ROOT_HEIGHT);
    }
}

pub type Gui = scui::Gui<GuiState, Rc<Root>>;

pub fn new_gui() -> Gui {
    Gui::new(
        GuiState {
            // registry,
            status: None,
            tooltip: Tooltip::default(),
        },
        |gui| Root::new(gui),
    )
}
