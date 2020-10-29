use crate::engine::parts as ep;
use crate::gui;
use crate::gui::constants::*;
use crate::gui::graphics::{GrahpicsWrapper, HAlign, VAlign};
use crate::registry::save_data::Patch;
use crate::registry::Registry;
use crate::scui_config::Renderer;
use enumflags2::BitFlags;
use scui::{Vec2D, Widget, WidgetImpl};
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
    pub tabs: Vec<Box<dyn GuiTab>>,
}

scui::widget! {
    pub Root
    Children {
        // menu_bar: Option<Rc<gui::ui_widgets::MenuBar>>,
    }
}

impl Root {
    fn new(parent: &impl RootParent) -> Rc<Self> {
        let state = RootState { pos: Vec2D::zero() };
        let this = Rc::new(Self::create(parent, state));
        let tab = TestTab::new(&this);
        this.with_gui_state_mut(|state| {
            state.tabs.push(Box::new(tab));
        });
        this
    }
}

impl WidgetImpl<Renderer> for Root {
    fn get_size(self: &Rc<Self>) -> Vec2D {
        (ROOT_WIDTH, ROOT_HEIGHT).into()
    }

    fn draw(self: &Rc<Self>, renderer: &mut Renderer) {
        renderer.set_color(&COLOR_BG0);
        renderer.fill_rect(0.0, 0.0, ROOT_WIDTH, ROOT_HEIGHT);
        renderer.push_state();
        renderer.apply_offset(0.0, 100.0);
        self.with_gui_state(|state| {
            for tab in &state.tabs {
                tab.draw(renderer);
            }
        });
        renderer.pop_state();
    }
}

scui::widget! {
    pub TestTab
}

pub trait GuiTab: Widget<Renderer> {
    fn get_name(self: &Self) -> String {
        "Unnamed".to_owned()
    }

    fn is_pinned(self: &Self) -> bool {
        false
    }
}

impl TestTab {
    fn new(parent: &impl TestTabParent) -> Rc<Self> {
        let state = TestTabState { pos: Vec2D::zero() };
        let this = Rc::new(Self::create(parent, state));
        this
    }
}

impl WidgetImpl<Renderer> for TestTab {
    fn draw(self: &Rc<Self>, renderer: &mut Renderer) {
        renderer.set_color(&COLOR_BG1);
        renderer.fill_rect(0.0, 0.0, 100.0, 100.0);
    }
}

impl GuiTab for Rc<TestTab> {}

pub type Gui = scui::Gui<GuiState, Rc<Root>>;

pub fn new_gui() -> Gui {
    Gui::new(
        GuiState {
            // registry,
            status: None,
            tooltip: Tooltip::default(),
            tabs: Vec::new(),
        },
        |gui| Root::new(gui),
    )
}
