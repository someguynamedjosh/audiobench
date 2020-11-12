use super::Module;
use crate::engine::parts as ep;
use crate::gui::constants::*;
use crate::gui::graphics::GrahpicsWrapper;
use crate::gui::{Gui, InteractionHint, Tooltip};
use crate::registry::Registry;
use crate::scui_config::{DropTarget, Renderer};
use scui::{MaybeMouseBehavior, MouseMods, Vec2D, Widget, WidgetImpl};
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
    }
    Children {
        modules: Vec<Rc<Module>>,
        popup_menu: Option<Box<dyn Widget<Renderer, DropTarget>>>
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
        children.popup_menu = None;
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

    pub fn add_module(self: &Rc<Self>, mut module: ep::Module) {
        let state = self.state.borrow();
        let pos = state.offset * -1.0;
        module.pos = (pos.x, pos.y);
        let module = rcrc(module);
        state.graph.borrow_mut().add_module(Rc::clone(&module));
        let mut children = self.children.borrow_mut();
        children.modules.push(Module::new(self, module));
    }

    pub fn remove_module(self: &Rc<Self>, module: &Rcrc<ep::Module>) {
        let mut state = self.state.borrow_mut();
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

    pub fn get_drop_target_at(self: &Rc<Self>, mouse_pos: Vec2D) -> DropTarget {
        // let mouse_pos = self.translate_mouse_pos(mouse_pos);
        // for module in &self.modules {
        //     let target = module.get_drop_target_at(mouse_pos);
        //     if !target.is_none() {
        //         return target;
        //     }
        // }
        DropTarget::None
    }

    pub fn get_tooltip_at(self: &Rc<Self>, mouse_pos: Vec2D) -> Option<Tooltip> {
        Some(Tooltip {
            text: "".to_owned(),
            interaction: InteractionHint::Scroll.into(),
        })
    }

    pub fn open_menu(self: &Rc<Self>, menu: Box<dyn Widget<Renderer, DropTarget>>) {
        self.children.borrow_mut().popup_menu = Some(menu);
    }

    pub fn get_current_draw_layer(self: &Rc<Self>) -> usize {
        self.state.borrow().current_draw_layer
    }

    pub fn get_highlight_mode(self: &Rc<Self>) -> Option<GraphHighlightMode> {
        self.state.borrow().highlight_mode
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
        ris!(self.get_mouse_behavior_children(pos, mods));
        // let pos = self.translate_mouse_pos(pos);
        // if let Some(widget) = &self.detail_menu_widget {
        //     let local_pos = pos.sub(widget.get_pos());
        //     if local_pos.inside(widget.get_bounds()) {
        //         return widget
        //             .respond_to_mouse_press(local_pos, mods)
        //             .scaled(Rc::clone(&self.zoom));
        //     } else {
        //         self.detail_menu_widget = None;
        //     }
        // }
        // for module in self.modules.iter().rev() {
        //     let action = module.respond_to_mouse_press(pos, mods);
        //     if !action.is_none() {
        //         return action.scaled(Rc::clone(&self.zoom));
        //     }
        // }
        // MouseAction::PanOffset(Rc::clone(&self.offset)).scaled(Rc::clone(&self.zoom))
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
        if let Some(popup) = &children.popup_menu {
            popup.draw(g);
        }
        let state = self.state.borrow();
        if let Some(end) = &state.wire_preview_endpoint {
            g.set_color(&COLOR_FG1);
            g.draw_line(*end, mouse_pos, 2.0);
        }
    }
}
