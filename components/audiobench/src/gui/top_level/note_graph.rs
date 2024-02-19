use scui::{ChildHolder, MouseMods, Vec2D, WidgetImpl};
use shared_util::prelude::*;

use crate::{
    gui::{constants::*, top_level::graph::ModuleGraph, GuiTab, TabArchetype},
    scui_config::{DropTarget, MaybeMouseBehavior, Renderer},
};

scui::widget! {
    pub NoteGraph
    State { }
    Children {
        graph: ChildHolder<Rc<ModuleGraph>>,
    }
}

impl NoteGraph {
    pub fn new(parent: &impl NoteGraphParent) -> Rc<Self> {
        let inter = parent.provide_gui_interface();
        let state = inter.state.borrow();
        let engine = state.engine.borrow();
        let graph = Rc::clone(engine.borrow_module_graph_ref());

        let this = Rc::new(Self::create(parent, NoteGraphState {}));
        let mut children = this.children.borrow_mut();
        children.graph = ModuleGraph::new(parent, &*state.registry.borrow(), graph).into();
        drop(children);

        this
    }
}

impl WidgetImpl<Renderer, DropTarget> for NoteGraph {
    fn get_pos_impl(self: &Rc<Self>) -> Vec2D {
        (0.0, HEADER_HEIGHT).into()
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

impl GuiTab for Rc<NoteGraph> {
    fn get_name(self: &Self) -> String {
        format!("Module Graph")
    }

    fn get_archetype(&self) -> TabArchetype {
        TabArchetype::NoteGraph
    }
}
