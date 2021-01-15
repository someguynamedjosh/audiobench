use crate::engine::parts as ep;
use crate::gui::constants::*;
use crate::gui::graphics::GrahpicsWrapper;
use crate::gui::top_level::graph::ModuleGraph;
use crate::gui::{Gui, GuiTab, InteractionHint, Tooltip};
use crate::registry::Registry;
use crate::scui_config::{DropTarget, MaybeMouseBehavior, Renderer};
use scui::{ChildHolder, MouseMods, Vec2D, Widget, WidgetImpl};
use shared_util::prelude::*;

scui::widget! {
    pub NoteGraph
    State { }
    Children {
        graph: ChildHolder<Rc<ModuleGraph>>,
    }
}

impl NoteGraph {
    pub fn new(parent: &impl NoteGraphParent) -> Rc<Self> {
        let mut this = Rc::new(Self::create(parent, NoteGraphState {}));
        let inter = parent.provide_gui_interface();
        let state = inter.state.borrow();
        let engine = state.engine.borrow();
        let graph = Rc::clone(engine.borrow_module_graph_ref());

        let this = Rc::new(Self::create(parent, NoteGraphState {}));
        let mut children = this.children.borrow_mut();
        children.graph = ModuleGraph::new(parent, graph).into();
        drop(children);

        this
    }
}

impl WidgetImpl<Renderer, DropTarget> for NoteGraph {
    fn get_pos_impl(self: &Rc<Self>) -> Vec2D {
        0.into()
    }

    fn get_size_impl(self: &Rc<Self>) -> Vec2D {
        (TAB_BODY_WIDTH, TAB_BODY_HEIGHT).into()
    }

    fn get_mouse_behavior_impl(
        self: &Rc<Self>,
        mouse_pos: Vec2D,
        mods: &MouseMods,
    ) -> MaybeMouseBehavior {
        self.get_mouse_behavior_children(mouse_pos, mods)
    }

    fn on_scroll_impl(self: &Rc<Self>, mouse_pos: Vec2D, delta: f32) -> Option<()> {
        self.on_scroll_children(mouse_pos, delta)
    }

    fn on_hover_impl(self: &Rc<Self>, mouse_pos: Vec2D) -> Option<()> {
        self.on_hover_children(mouse_pos)
    }

    fn draw_impl(self: &Rc<Self>, g: &mut Renderer) {
        self.draw_children(g);
    }
}

impl GuiTab for Rc<NoteGraph> {}
