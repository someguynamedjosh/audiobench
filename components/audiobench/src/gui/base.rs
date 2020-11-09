use crate::engine::UiThreadEngine;
use crate::gui::constants::*;
use crate::gui::top_level::*;
use crate::scui_config::Renderer;
use enumflags2::BitFlags;
use scui::{MaybeMouseBehavior, MouseMods, Vec2D, Widget, WidgetImpl};
use shared_util::prelude::*;

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
    pub engine: Rcrc<UiThreadEngine>,
    status: Option<Status>,
    tooltip: Tooltip,
    tabs: Vec<Box<dyn GuiTab>>,
    current_tab_index: usize,
}

impl GuiState {
    pub fn new(engine: Rcrc<UiThreadEngine>) -> Self {
        Self {
            engine,
            status: None,
            tooltip: Default::default(),
            tabs: Vec::new(),
            current_tab_index: 0,
        }
    }

    pub fn set_tooltip(&mut self, tooltip: Tooltip) {
        self.tooltip = tooltip;
    }

    pub fn borrow_tooltip(&self) -> &Tooltip {
        &self.tooltip
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

    pub fn add_success_status(&mut self, message: String) {
        self.status = Some(Status::success(message));
    }

    pub fn add_error_status(&mut self, message: String) {
        self.status = Some(Status::error(message));
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
        let state = RootState {};
        let this = Rc::new(Self::create(parent, state));
        let tab = Box::new(PatchBrowser::new(&this));
        this.with_gui_state_mut(|state| {
            state.add_tab(tab);
        });
        let header = Header::new(&this);
        this.children.borrow_mut().header = Some(header);
        this
    }
}

impl WidgetImpl<Renderer> for Root {
    fn get_pos(self: &Rc<Self>) -> Vec2D {
        0.into()
    }

    fn get_size(self: &Rc<Self>) -> Vec2D {
        (ROOT_WIDTH, ROOT_HEIGHT).into()
    }

    fn get_mouse_behavior(
        self: &Rc<Self>,
        mouse_pos: Vec2D,
        mods: &MouseMods,
    ) -> MaybeMouseBehavior {
        self.with_gui_state(|state| {
            let tab = &state.tabs[state.get_current_tab_index()];
            let child_pos = mouse_pos - tab.get_pos();
            tab.get_mouse_behavior(child_pos, mods)
        })
    }

    fn on_hover(self: &Rc<Self>, mouse_pos: Vec2D) -> bool {
        self.with_gui_state(|state| {
            let tab = &state.tabs[state.get_current_tab_index()];
            let child_pos = mouse_pos - tab.get_pos();
            tab.on_hover(child_pos)
        })
    }

    fn on_scroll(self: &Rc<Self>, mouse_pos: Vec2D, delta: f32) -> bool {
        self.with_gui_state(|state| {
            let tab = &state.tabs[state.get_current_tab_index()];
            let child_pos = mouse_pos - tab.get_pos();
            tab.on_scroll(child_pos, delta)
        })
    }

    fn draw(self: &Rc<Self>, renderer: &mut Renderer) {
        renderer.set_color(&COLOR_BG0);
        renderer.draw_rect(0, (ROOT_WIDTH, ROOT_HEIGHT));
        self.with_gui_state(|state| {
            state.tabs[state.get_current_tab_index()].draw(renderer);
        });
        self.draw_children(renderer);
    }
}

pub trait GuiTab: Widget<Renderer> {
    fn get_name(self: &Self) -> String {
        "Unnamed".to_owned()
    }

    fn is_pinned(self: &Self) -> bool {
        false
    }
}

pub type Gui = scui::Gui<GuiState, Rc<Root>>;

pub fn new_gui(engine: Rcrc<UiThreadEngine>) -> Gui {
    Gui::new(GuiState::new(engine), |gui| Root::new(gui))
}
