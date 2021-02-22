use std::cmp::Ordering;

use crate::{
    engine::{controls::Control, parts::JackType, UiThreadEngine},
    gui::{constants::*, top_level::*},
    registry::Registry,
    scui_config::{DropTarget, MaybeMouseBehavior, Renderer},
};
use scui::{MouseMods, Vec2D, Widget, WidgetImpl};
use shared_util::prelude::*;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum InteractionHint {
    Scroll,
    LeftClick,
    LeftClickAndDrag,
    RightClick,
    DoubleClick,
    PrecisionModifier,
    SnappingModifier,
    TakesInput(JackType),
    ProducesOutput(JackType),
}

impl InteractionHint {
    fn jack_type_ordinal(ty: &JackType) -> u8 {
        use JackType::*;
        match ty {
            Audio => 0,
            Pitch => 1,
            Trigger => 1,
            Waveform => 1,
        }
    }

    fn ordinal(&self) -> u8 {
        use InteractionHint::*;
        match self {
            Scroll => 0,
            LeftClick => 1,
            LeftClickAndDrag => 2,
            DoubleClick => 3,
            RightClick => 4,
            PrecisionModifier => 5,
            SnappingModifier => 6,
            TakesInput(ty) => 10 + Self::jack_type_ordinal(ty),
            ProducesOutput(ty) => 20 + Self::jack_type_ordinal(ty),
        }
    }
}

impl PartialOrd for InteractionHint {
    fn partial_cmp(&self, other: &InteractionHint) -> Option<Ordering> {
        self.ordinal().partial_cmp(&other.ordinal())
    }
}

impl Ord for InteractionHint {
    fn cmp(&self, other: &InteractionHint) -> Ordering {
        self.ordinal().cmp(&other.ordinal())
    }
}

#[derive(Clone)]
pub struct Tooltip {
    pub text: String,
    pub interaction: Vec<InteractionHint>,
}

impl Default for Tooltip {
    fn default() -> Tooltip {
        Tooltip {
            text: "".to_owned(),
            interaction: Default::default(),
        }
    }
}

impl Tooltip {
    pub fn add_control_automation(&mut self, control: &Rcrc<impl Control + ?Sized>) {
        for acceptable_type in control.borrow().acceptable_automation() {
            self.interaction
                .push(InteractionHint::TakesInput(acceptable_type));
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

    pub fn add_automation_to_tooltip(&mut self, from_control: &Rcrc<impl Control + ?Sized>) {
        self.tooltip.add_control_automation(from_control);
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

    pub fn clear_status(&mut self) {
        self.status = None;
    }

    pub fn borrow_status(&self) -> &Option<Status> {
        &self.status
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
        let tab3 = LibraryInfo::new(&this);
        this.with_gui_state_mut(|state| {
            state.add_tab(tab1);
            state.add_tab(tab2);
            state.add_tab(tab3);
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
        self.with_gui_state_mut(|state| state.clear_status());
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

        let julia_busy = self.with_gui_state(|state| state.engine.borrow().is_julia_thread_busy());
        if julia_busy {
            renderer.set_color(&COLOR_WARNING);
            const F: f32 = BIG_FONT_SIZE;
            let size = Vec2D::new(F * 7.0, F + GRID_P * 2.0);
            let pos = Vec2D::new(ROOT_WIDTH, ROOT_HEIGHT) - size - GRID_P;
            renderer.draw_rounded_rect(pos, size, CORNER_SIZE);
            renderer.set_color(&COLOR_FG1);
            renderer.draw_text(F, pos, size, (0, 0), 1, "Working...");
        }
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
