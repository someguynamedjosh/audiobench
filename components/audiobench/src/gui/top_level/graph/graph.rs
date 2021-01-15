use super::Module;
use crate::engine::parts as ep;
use crate::gui::constants::*;
use crate::gui::graphics::GrahpicsWrapper;
use crate::gui::top_level::ModuleBrowser;
use crate::gui::{Gui, InteractionHint, Tooltip};
use crate::registry::module_template::ModuleTemplate;
use crate::registry::Registry;
use crate::scui_config::{DropTarget, MaybeMouseBehavior, Renderer};
use scones::make_constructor;
use scui::{
    GuiInterfaceProvider, MouseBehavior, MouseMods, OnClickBehavior, Vec2D, Widget, WidgetImpl,
};
use shared_util::prelude::*;

scui::widget! {
    pub ModuleGraph
    State {
        offset: Vec2D,
        zoom: f32,
        graph: Rcrc<ep::ModuleGraph>,
        highlight_mode: Option<GraphHighlightMode>,
        current_draw_layer: usize,
        wire_preview_endpoint: Option<Vec2D>,
        hovered_module: Option<Rc<Module>>,
    }
    Children {
        modules: Vec<Rc<Module>>,
        detail_menu: Option<Box<dyn Widget<Renderer, DropTarget>>>
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GraphHighlightMode {
    ReceivesType(ep::JackType),
    ProducesType(ep::JackType),
}

impl ModuleGraph {
    pub fn new(parent: &impl ModuleGraphParent, graph: Rcrc<ep::ModuleGraph>) -> Rc<Self> {
        let state = ModuleGraphState {
            offset: (0.0, 0.0).into(),
            zoom: 1.0,
            graph: Rc::clone(&graph),
            highlight_mode: None,
            current_draw_layer: 0,
            wire_preview_endpoint: None,
            hovered_module: None,
        };
        let this = Rc::new(Self::create(parent, state));
        let mut children = this.children.borrow_mut();
        children.modules = graph
            .borrow()
            .borrow_modules()
            .iter()
            .map(|module_rc| Module::new(&this, Rc::clone(module_rc)))
            .collect();
        drop(children);
        this.recenter();
        this
    }

    pub fn rebuild(self: &Rc<Self>) {
        let mut children = self.children.borrow_mut();
        children.modules.clear();
        children.detail_menu = None;
        let state = self.state.borrow();
        let mut top_left = Vec2D::from(std::f32::MAX);
        let mut bottom_right = Vec2D::from(std::f32::MIN);
        for module_rc in state.graph.borrow().borrow_modules() {
            let module_widget = Module::new(self, Rc::clone(module_rc));
            let pos = module_widget.get_pos();
            let size = module_widget.get_size();
            top_left = top_left.min(pos);
            bottom_right = bottom_right.max(pos + size);
            children.modules.push(module_widget);
        }
        drop(children);
        drop(state);
        self.recenter();
    }

    fn recenter(self: &Rc<Self>) {
        let mut top_left = Vec2D::from(std::f32::MAX);
        let mut bottom_right = Vec2D::from(std::f32::MIN);
        let children = self.children.borrow();
        if children.modules.len() == 0 {
            top_left = 0.0.into();
            bottom_right = 0.0.into();
        }
        for module in &children.modules {
            let tl = module.get_pos();
            let br = tl + module.get_size();
            top_left = top_left.min(tl);
            bottom_right = bottom_right.max(br);
        }
        let size = self.get_size();
        let center = (bottom_right - top_left) / 2.0 + top_left;
        let mut state = self.state.borrow_mut();
        state.offset = center - size / (state.zoom * 2.0);
    }

    /// This also adds the module to the actual graph this widget represents.
    pub fn add_module(self: &Rc<Self>, template: Rcrc<ModuleTemplate>) {
        let mut module = ep::Module::create(template);
        let state = self.state.borrow();
        let pos = state.offset * -1.0;
        module.pos = (pos.x, pos.y);
        let module = rcrc(module);
        state.graph.borrow_mut().add_module(Rc::clone(&module));
        let mut children = self.children.borrow_mut();
        children.modules.push(Module::new(self, module));
        self.with_gui_state_mut(|state| {
            state.engine.borrow_mut().recompile();
        })
    }

    pub fn remove_module(self: &Rc<Self>, module: &Rcrc<ep::Module>) {
        let state = self.state.borrow();
        state.graph.borrow_mut().remove_module(module);
        let mut children = self.children.borrow_mut();
        let index = children
            .modules
            .iter()
            .position(|e| e.represents_module(module))
            .unwrap();
        children.modules.remove(index).on_removed();
    }

    fn translate_mouse_pos(self: &Rc<Self>, mouse_pos: Vec2D) -> Vec2D {
        let state = self.state.borrow();
        let offset = state.offset;
        let zoom = state.zoom;
        mouse_pos / zoom - offset
    }

    pub fn pan(self: &Rc<Self>, delta: Vec2D) {
        self.state.borrow_mut().offset += delta;
    }

    pub fn open_menu(self: &Rc<Self>, menu: Box<dyn Widget<Renderer, DropTarget>>) {
        self.children.borrow_mut().detail_menu = Some(menu);
    }

    pub fn get_current_draw_layer(self: &Rc<Self>) -> usize {
        self.state.borrow().current_draw_layer
    }

    pub fn get_highlight_mode(self: &Rc<Self>) -> Option<GraphHighlightMode> {
        self.state.borrow().highlight_mode
    }

    pub fn set_hovered_module(self: &Rc<Self>, module: Rc<Module>) {
        self.state.borrow_mut().hovered_module = Some(module);
    }

    pub fn clear_hovered_module(self: &Rc<Self>) {
        self.state.borrow_mut().hovered_module = None;
    }

    pub fn is_hovered_module(self: &Rc<Self>, module: &Rc<Module>) -> bool {
        let state = self.state.borrow();
        if let Some(hovered) = &state.hovered_module {
            Rc::ptr_eq(hovered, module)
        } else {
            false
        }
    }
}

#[make_constructor]
struct GraphInteract {
    graph: Rc<ModuleGraph>,
}

impl MouseBehavior<DropTarget> for GraphInteract {
    fn on_drag(&mut self, delta: Vec2D, _mods: &MouseMods) {
        self.graph.pan(delta);
    }

    fn on_double_click(self: Box<Self>) {
        let tab = ModuleBrowser::new(&self.graph, Rc::clone(&self.graph));
        let interface = self.graph.provide_gui_interface();
        let mut state = interface.state.borrow_mut();
        state.add_tab(tab);
    }
}

impl WidgetImpl<Renderer, DropTarget> for ModuleGraph {
    fn get_pos_impl(self: &Rc<Self>) -> Vec2D {
        (0.0, HEADER_HEIGHT).into()
    }

    fn get_size_impl(self: &Rc<Self>) -> Vec2D {
        (TAB_BODY_WIDTH, TAB_BODY_HEIGHT).into()
    }

    fn get_mouse_behavior_impl(
        self: &Rc<Self>,
        pos: Vec2D,
        mods: &MouseMods,
    ) -> MaybeMouseBehavior {
        let pos = self.translate_mouse_pos(pos);
        let children = self.children.borrow();
        if let Some(widget) = &children.detail_menu {
            let local_pos = pos - widget.get_pos();
            if local_pos.inside(widget.get_size()) {
                return widget.get_mouse_behavior(pos, mods);
            // .scaled(Rc::clone(&self.zoom));
            } else {
                let this = Rc::clone(self);
                return OnClickBehavior::wrap(move || {
                    this.children.borrow_mut().detail_menu = None;
                });
            }
        }
        for module in children.modules.iter().rev() {
            ris!(module.get_mouse_behavior(pos, mods))
        }
        Some(Box::new(GraphInteract::new(Rc::clone(self))))
    }

    fn on_hover_impl(self: &Rc<Self>, pos: Vec2D) -> Option<()> {
        self.clear_hovered_module();
        let pos = self.translate_mouse_pos(pos);
        let children = self.children.borrow();
        if let Some(widget) = &children.detail_menu {
            let local_pos = pos - widget.get_pos();
            if local_pos.inside(widget.get_size()) {
                return widget.on_hover(pos);
            }
        }
        for module in children.modules.iter().rev() {
            ris!(module.on_hover(pos))
        }
        None
    }

    fn draw_impl(self: &Rc<Self>, g: &mut GrahpicsWrapper) {
        let mouse_pos = self.parents.gui.get_mouse_pos();
        let state = self.state.borrow();
        let children = self.children.borrow();
        g.scale(state.zoom);
        g.translate(state.offset);
        drop(state);
        for layer in 0..4 {
            let mut state = self.state.borrow_mut();
            state.current_draw_layer = layer;
            drop(state);
            for module in &children.modules {
                module.draw(g);
            }
        }
        if let Some(widget) = &children.detail_menu {
            widget.draw(g);
        }
        let state = self.state.borrow();
        if let Some(end) = &state.wire_preview_endpoint {
            g.set_color(&COLOR_FG1);
            g.draw_line(*end, mouse_pos, 2.0);
        }
    }
}
