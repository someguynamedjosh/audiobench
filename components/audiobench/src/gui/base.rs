use crate::engine::UiThreadEngine;
use crate::gui::constants::*;
use crate::gui::top_level::*;
use crate::registry::Registry;
use crate::scui_config::{DropTarget, MaybeMouseBehavior, Renderer};
use enumflags2::BitFlags;
use scui::{MouseMods, Vec2D, Widget, WidgetImpl};
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
    pub registry: Rcrc<Registry>,
    pub engine: Rcrc<UiThreadEngine>,
    status: Option<Status>,
    tooltip: Tooltip,
    // Yeah I know  this is doing Rc<Rc<Widget>> but I don't know what else to do at the moment.
    tabs: Vec<Rc<dyn GuiTab>>,
    current_tab_index: usize,
}

impl GuiState {
    pub fn new(registry: Rcrc<Registry>, engine: Rcrc<UiThreadEngine>) -> Self {
        Self {
            registry,
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

    pub fn add_tab(&mut self, tab: impl GuiTab + 'static) {
        self.tabs.push(Rc::new(tab));
    }

    pub fn all_tabs(&self) -> impl Iterator<Item = &Rc<dyn GuiTab>> {
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
        let tab1 = PatchBrowser::new(&this);
        let tab2 = NoteGraph::new(&this);
        this.with_gui_state_mut(|state| {
            state.add_tab(tab1);
            state.add_tab(tab2);
        });
        let header = Header::new(&this);
        this.children.borrow_mut().header = Some(header);
        this
    }

    fn get_current_tab(self: &Rc<Self>) -> Rc<dyn GuiTab> {
        self.with_gui_state(|state| Rc::clone(&state.tabs[state.get_current_tab_index()]))
    }
}

impl WidgetImpl<Renderer, DropTarget> for Root {
    fn get_pos_impl(self: &Rc<Self>) -> Vec2D {
        0.into()
    }

    fn get_size_impl(self: &Rc<Self>) -> Vec2D {
        (ROOT_WIDTH, ROOT_HEIGHT).into()
    }

    fn get_drop_target_impl(self: &Rc<Self>, pos: Vec2D) -> Option<DropTarget> {
        ris!(self.get_drop_target_children(pos));
        self.get_current_tab().get_drop_target(pos)
    }

    fn get_mouse_behavior_impl(
        self: &Rc<Self>,
        mouse_pos: Vec2D,
        mods: &MouseMods,
    ) -> MaybeMouseBehavior {
        ris!(self.get_mouse_behavior_children(mouse_pos, mods));
        self.get_current_tab().get_mouse_behavior(mouse_pos, mods)
    }

    fn on_scroll_impl(self: &Rc<Self>, mouse_pos: Vec2D, delta: f32) -> Option<()> {
        ris!(self.on_scroll_children(mouse_pos, delta));
        self.get_current_tab().on_scroll(mouse_pos, delta)
    }

    fn on_hover_impl(self: &Rc<Self>, mouse_pos: Vec2D) -> Option<()> {
        self.with_gui_state_mut(|state| {
            state.set_tooltip(Tooltip::default());
        });
        ris!(self.on_hover_children(mouse_pos));
        self.get_current_tab().on_hover(mouse_pos)
    }

    fn draw_impl(self: &Rc<Self>, renderer: &mut Renderer) {
        renderer.set_color(&COLOR_BG0);
        renderer.draw_rect(0, (ROOT_WIDTH, ROOT_HEIGHT));
        self.get_current_tab().draw(renderer);
        self.draw_children(renderer);
    }
}

pub trait GuiTab: Widget<Renderer, DropTarget> {
    fn get_name(self: &Self) -> String {
        "Unnamed".to_owned()
    }

    fn is_pinned(self: &Self) -> bool {
        false
    }
}

pub type Gui = scui::Gui<GuiState, DropTarget, Rc<Root>>;

pub fn new_gui(registry: Rcrc<Registry>, engine: Rcrc<UiThreadEngine>) -> Gui {
    Gui::new(GuiState::new(registry, engine), |gui| Root::new(gui))
}
