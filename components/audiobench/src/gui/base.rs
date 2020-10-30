use crate::engine::parts as ep;
use crate::gui;
use crate::gui::constants::*;
use crate::gui::graphics::{GrahpicsWrapper, HAlign, VAlign};
use crate::gui::top_level::*;
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
    tabs: Vec<Box<dyn GuiTab>>,
    current_tab_index: usize,
}

impl GuiState {
    pub fn new() -> Self {
        Self {
            status: None,
            tooltip: Default::default(),
            tabs: Vec::new(),
            current_tab_index: 0,
        }
    }

    pub fn add_tab(&mut self, tab: Box<dyn GuiTab>) {
        self.tabs.push(tab);
    }

    pub fn all_tabs(&self) -> impl Iterator<Item = &Box<dyn GuiTab>> {
        self.tabs.iter()
    }

    pub fn get_current_tab_index(&self) -> usize {
        self.current_tab_index
    }

    pub fn focus_tab_by_index(&mut self, index: usize) {
        if index < self.tabs.len() {
            self.current_tab_index = index;
        }
    }
}

scui::widget! {
    pub Root
    Children {
        header: Option<Rc<Header>>,
    }
}

impl Root {
    fn new(parent: &impl RootParent) -> Rc<Self> {
        let state = RootState { pos: Vec2D::zero() };
        let this = Rc::new(Self::create(parent, state));
        let tab = TestTab::new(&this);
        this.with_gui_state_mut(|state| {
            state.add_tab(Box::new(tab));
            state.add_tab(Box::new(TestTab::new(&this)));
        });
        let header = Header::new(&this);
        this.children.borrow_mut().header = Some(header);
        this
    }
}

impl WidgetImpl<Renderer> for Root {
    fn get_size(self: &Rc<Self>) -> Vec2D {
        (ROOT_WIDTH, ROOT_HEIGHT).into()
    }

    fn draw(self: &Rc<Self>, renderer: &mut Renderer) {
        renderer.set_color(&COLOR_BG0);
        renderer.draw_rect(0, (ROOT_WIDTH, ROOT_HEIGHT));
        self.with_gui_state(|state| {
            for tab in &state.tabs {
                tab.draw(renderer);
            }
        });
        self.draw_children(renderer);
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
        let state = TestTabState {
            pos: (0.0, HEADER_HEIGHT).into(),
        };
        let this = Rc::new(Self::create(parent, state));
        this
    }
}

impl WidgetImpl<Renderer> for TestTab {
    fn get_size(self: &Rc<Self>) -> Vec2D {
        (TAB_BODY_WIDTH, TAB_BODY_HEIGHT).into()
    }

    fn draw(self: &Rc<Self>, renderer: &mut Renderer) {
        renderer.set_color(&COLOR_BG1);
        renderer.draw_rect(0, (TAB_BODY_WIDTH, TAB_BODY_HEIGHT));
    }
}

impl GuiTab for Rc<TestTab> {}

pub type Gui = scui::Gui<GuiState, Rc<Root>>;

pub fn new_gui() -> Gui {
    Gui::new(GuiState::new(), |gui| Root::new(gui))
}
