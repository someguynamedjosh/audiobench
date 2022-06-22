use std::cmp::Ordering;

use observatory::{observable, ObservablePtr};
use scui::{MouseMods, Vec2D, Widget, WidgetImpl};
use shared_util::prelude::*;

use crate::{
    engine::{controls::Control, parts::JackType, Status, UiThreadEngine},
    gui::{constants::*, top_level::*},
    registry::{save_data::Patch, Registry},
    scui_config::{DropTarget, MaybeMouseBehavior, Renderer},
};

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
pub struct StatusMessage {
    pub text: String,
    pub color: (u8, u8, u8),
}

impl StatusMessage {
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

#[derive(Clone)]
pub enum TabArchetype {
    PatchBrowser,
    NoteGraph,
    ModuleBrowser(Rc<graph::ModuleGraph>),
    LibraryInfo,
    MessageLog,
}

impl TabArchetype {
    /// Returns true if the two archetypes represent the same GUI layout,
    /// presenting the same information and tools.
    pub fn equivalent(&self, other: &Self) -> bool {
        use TabArchetype::*;
        match self {
            PatchBrowser => {
                if let PatchBrowser = other {
                    true
                } else {
                    false
                }
            }
            NoteGraph => {
                if let NoteGraph = other {
                    true
                } else {
                    false
                }
            }
            ModuleBrowser(a) => {
                if let ModuleBrowser(b) = other {
                    Rc::ptr_eq(a, b)
                } else {
                    false
                }
            }
            LibraryInfo => {
                if let LibraryInfo = other {
                    true
                } else {
                    false
                }
            }
            MessageLog => {
                if let MessageLog = other {
                    true
                } else {
                    false
                }
            }
        }
    }

    pub fn instantiate(self, parent: &impl PatchBrowserParent) -> Rc<dyn GuiTab> {
        match self {
            Self::PatchBrowser => Rc::new(PatchBrowser::new(parent)) as _,
            Self::NoteGraph => Rc::new(NoteGraph::new(parent)) as _,
            Self::ModuleBrowser(add_to) => Rc::new(ModuleBrowser::new(parent, add_to)) as _,
            Self::LibraryInfo => Rc::new(LibraryInfo::new(parent)) as _,
            Self::MessageLog => Rc::new(MessageLog::new(parent)) as _,
        }
    }
}

pub struct GuiState {
    pub registry: Rcrc<Registry>,
    pub engine: Rcrc<UiThreadEngine>,
    pub current_patch_index: Option<usize>,
    pub patch_list: ObservablePtr<Vec<Rcrc<Patch>>>,
    messages: Vec<StatusMessage>,
    last_message: bool,
    tooltip: Tooltip,
    // Yeah I know  this is doing Rc<Rc<Widget>> but I don't know what else to do at the moment.
    tabs: Vec<Rc<dyn GuiTab>>,
    current_tab_index: usize,
}

impl GuiState {
    pub fn new(registry: Rcrc<Registry>, engine: Rcrc<UiThreadEngine>) -> Self {
        let patch_list = registry.borrow().borrow_patches().clone();
        let patch_list: Vec<_> = patch_list
            .into_iter()
            .filter(|patch| patch.borrow().exists_on_disk() || !patch.borrow().is_writable())
            .collect();
        let current_patch = Rc::clone(&engine.borrow().borrow_current_patch().borrow_untracked());
        let mut current_patch_index = None;
        let patch = current_patch.borrow();
        for (index, other) in patch_list.iter().enumerate() {
            let other = other.borrow();
            if other.borrow_name() == patch.borrow_name() {
                if other.serialize() == patch.serialize() {
                    current_patch_index = Some(index);
                    break;
                }
            }
        }
        drop(patch);
        Self {
            registry,
            engine,
            current_patch_index,
            patch_list: observable(patch_list),
            messages: Vec::new(),
            last_message: false,
            tooltip: Default::default(),
            tabs: Vec::new(),
            current_tab_index: 0,
        }
    }

    pub fn after_new_patch(&mut self, new_patch: &Rcrc<Patch>) {
        let next_entry_index = self.patch_list.borrow_untracked().len();
        if new_patch.borrow().exists_on_disk() {
            self.patch_list.borrow_mut().push(Rc::clone(new_patch));
            self.current_patch_index = Some(next_entry_index);
        } else {
            self.current_patch_index = None;
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

    pub fn switch_to(&mut self, archetype: TabArchetype) -> bool {
        for (index, candidate) in self.tabs.iter().enumerate() {
            if candidate.get_archetype().equivalent(&archetype) {
                self.current_tab_index = index;
                return true;
            }
        }
        return false;
    }

    pub fn add_tab(&mut self, tab: Rc<dyn GuiTab>) {
        self.current_tab_index = self.tabs.len();
        self.tabs.push(tab);
    }

    pub fn focus_tab_by_index(&mut self, index: usize) {
        assert!(index < self.tabs.len());
        self.current_tab_index = index;
    }

    pub fn close_tab(&mut self, tab: Rc<dyn GuiTab>) {
        let archetype = tab.get_archetype();
        for (index, candidate) in self.tabs.iter().enumerate() {
            if candidate.get_archetype().equivalent(&archetype) {
                self.tabs.remove(index);
                if index <= self.current_tab_index && self.current_tab_index > 0 {
                    self.current_tab_index -= 1;
                }
                return;
            }
        }
    }

    pub fn all_tabs(&self) -> impl Iterator<Item = &Rc<dyn GuiTab>> {
        self.tabs.iter()
    }

    pub fn num_tabs(&self) -> usize {
        self.tabs.len()
    }

    pub fn get_current_tab_index(&self) -> usize {
        self.current_tab_index
    }

    pub fn add_message(&mut self, message: StatusMessage) {
        self.messages.push(message);
        self.last_message = true;
    }

    pub fn add_success_message(&mut self, message: String) {
        self.add_message(StatusMessage::success(message))
    }

    pub fn add_error_message(&mut self, message: String) {
        self.add_message(StatusMessage::error(message))
    }

    pub fn clear_last_message(&mut self) {
        self.last_message = false;
    }

    pub fn borrow_last_message(&self) -> Option<&StatusMessage> {
        if self.last_message {
            self.messages.last()
        } else {
            None
        }
    }

    pub fn borrow_all_messages(&self) -> &[StatusMessage] {
        &self.messages[..]
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
        let tab = TabArchetype::PatchBrowser.instantiate(&this);
        this.parents.gui.state.borrow_mut().add_tab(tab);
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
        self.with_gui_state_mut(|state| state.clear_last_message());
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
        self.with_gui_state_mut(|state| {
            let new_errors = state.engine.borrow_mut().take_posted_errors();
            for error in new_errors {
                state.add_error_message(error);
            }
        });

        renderer.set_color(&COLOR_BG0);
        renderer.draw_rect(0, (ROOT_WIDTH, ROOT_HEIGHT));
        self.get_current_tab().draw(renderer);
        self.draw_children(renderer);

        let processing_thread_status =
            self.with_gui_state(|state| state.engine.borrow().get_processing_thread_status());
        let message = if processing_thread_status == Status::Busy {
            renderer.set_color(&COLOR_WARNING);
            "Working..."
        } else if processing_thread_status == Status::Error {
            renderer.set_color(&COLOR_ERROR);
            "Unrecoverable Error :("
        } else {
            return;
        };
        const F: f32 = BIG_FONT_SIZE;
        let size = Vec2D::new(F * message.len() as f32 * 0.5 + F * 2.0, F + GRID_P * 2.0);
        let pos = Vec2D::new(ROOT_WIDTH, ROOT_HEIGHT) - size - GRID_P;
        renderer.draw_rounded_rect(pos, size, CORNER_SIZE);
        renderer.set_color(&COLOR_FG1);
        renderer.draw_text(F, pos, size, (0, 0), 1, message);
    }
}

pub trait GuiTab: Widget<Renderer, DropTarget> {
    fn get_name(self: &Self) -> String {
        "Unnamed".to_owned()
    }

    fn is_pinned(self: &Self) -> bool {
        false
    }

    fn get_archetype(&self) -> TabArchetype;
}

pub type Gui = scui::Gui<GuiState, DropTarget, Rc<Root>>;

pub fn new_gui(registry: Rcrc<Registry>, engine: Rcrc<UiThreadEngine>) -> Gui {
    Gui::new(GuiState::new(registry, engine), |gui| Root::new(gui))
}
