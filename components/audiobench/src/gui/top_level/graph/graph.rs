use crate::{
    engine::{
        controls::{AutomationSource, Control},
        parts as ep,
    },
    gui::{
        constants::*,
        graphics::GrahpicsWrapper,
        top_level::{graph::Module, ModuleBrowser},
        InteractionHint, Tooltip,
    },
    registry::module_template::ModuleTemplate,
    scui_config::{DropTarget, MaybeMouseBehavior, Renderer},
};
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
        highlight_mode: GraphHighlightMode,
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
    None,
    ReceivesType(ep::JackType),
    ProducesType(ep::JackType),
}

impl GraphHighlightMode {
    pub fn is_some(self) -> bool {
        match self {
            Self::None => false,
            _ => true,
        }
    }

    pub fn should_highlight(self, control: &Rcrc<impl Control + ?Sized>) -> bool {
        match self {
            Self::None => false,
            Self::ReceivesType(typ) => control
                .borrow()
                .acceptable_automation()
                .into_iter()
                .any(|i| i == typ),
            Self::ProducesType(_) => false,
        }
    }

    pub fn should_dim(&self, control: &Rcrc<impl Control + ?Sized>) -> bool {
        if let Self::ReceivesType(..) = self {
            !self.should_highlight(control)
        } else {
            false
        }
    }
}

impl ModuleGraph {
    pub fn new(parent: &impl ModuleGraphParent, graph: Rcrc<ep::ModuleGraph>) -> Rc<Self> {
        let state = ModuleGraphState {
            offset: (0.0, 0.0).into(),
            zoom: 1.0,
            graph: Rc::clone(&graph),
            highlight_mode: GraphHighlightMode::None,
            current_draw_layer: 0,
            wire_preview_endpoint: None,
            hovered_module: None,
        };
        let this = Rc::new(Self::create(parent, state));
        graph.borrow_mut().current_widget = Some(Rc::clone(&this));
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
        state.offset = -center + size / (state.zoom * 2.0);
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
            state.engine.borrow_mut().regenerate_code();
        });
    }

    pub fn get_widget_for_module(self: &Rc<Self>, module: &Rcrc<ep::Module>) -> Option<Rc<Module>> {
        let children = self.children.borrow();
        let index = children
            .modules
            .iter()
            .find(|e| e.represents_module(module));
        index.map(|p| Rc::clone(p))
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
        self.with_gui_state_mut(|state| {
            state.engine.borrow_mut().regenerate_code();
        });
    }

    /// Translates screen-space coordinates to graph-space.
    fn translate_screen_pos(self: &Rc<Self>, pos: Vec2D) -> Vec2D {
        let state = self.state.borrow();
        let offset = state.offset;
        let zoom = state.zoom;
        pos / zoom - offset
    }

    pub fn pan(self: &Rc<Self>, delta: Vec2D) {
        let mut state = self.state.borrow_mut();
        let zoom = state.zoom;
        state.offset += delta / zoom;
    }

    pub fn open_menu(self: &Rc<Self>, menu: Box<dyn Widget<Renderer, DropTarget>>) {
        self.children.borrow_mut().detail_menu = Some(menu);
    }

    pub fn get_current_draw_layer(self: &Rc<Self>) -> usize {
        self.state.borrow().current_draw_layer
    }

    pub fn get_highlight_mode(self: &Rc<Self>) -> GraphHighlightMode {
        self.state.borrow().highlight_mode
    }

    pub fn get_zoom(self: &Rc<Self>) -> f32 {
        self.state.borrow().zoom
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

    fn clear_wire_preview(self: &Rc<Self>) {
        let mut state = self.state.borrow_mut();
        state.wire_preview_endpoint = None;
        state.highlight_mode = GraphHighlightMode::None;
    }

    pub fn connect_from_source_behavior(
        self: &Rc<Self>,
        module: Rcrc<ep::Module>,
        output_index: usize,
    ) -> Box<ConnectFromSource> {
        let mod_ref = module.borrow();
        let mut state = self.state.borrow_mut();
        state.wire_preview_endpoint = Some(Module::output_position(&*mod_ref, output_index));
        let template = mod_ref.template.borrow();
        state.highlight_mode =
            GraphHighlightMode::ReceivesType(template.outputs[output_index].get_type());
        let output_type = template.outputs[output_index].get_type();
        let graph = Rc::clone(self);
        drop(template);
        drop(mod_ref);
        Box::new(ConnectFromSource {
            graph,
            source: AutomationSource {
                module,
                output_index,
                output_type,
            },
        })
    }

    pub fn connect_to_control_behavior(
        self: &Rc<Self>,
        control: Rcrc<dyn Control>,
        visual_pos: Vec2D,
    ) -> Box<ConnectToControl> {
        let mut state = self.state.borrow_mut();
        state.wire_preview_endpoint = Some(visual_pos);
        let acceptable = control.borrow().acceptable_automation();
        assert!(acceptable.len() > 0);
        debug_assert_eq!(acceptable.len(), 1);
        state.highlight_mode = GraphHighlightMode::ProducesType(acceptable[0]);
        let graph = Rc::clone(self);
        Box::new(ConnectToControl { graph, control })
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

pub struct ConnectFromSource {
    graph: Rc<ModuleGraph>,
    source: AutomationSource,
}

impl MouseBehavior<DropTarget> for ConnectFromSource {
    fn on_click(self: Box<Self>) {
        self.on_drop(None);
    }

    fn on_drop(self: Box<Self>, drop_target: Option<DropTarget>) {
        if let Some(DropTarget::Control(control)) = drop_target {
            let types = control.borrow().acceptable_automation();
            if types.into_iter().any(|t| t == self.source.output_type) {
                control.borrow_mut().connect_automation(self.source);
            }
            self.graph.with_gui_state_mut(|state| {
                state.engine.borrow_mut().regenerate_code();
            })
        }
        self.graph.clear_wire_preview();
    }
}

pub struct ConnectToControl {
    graph: Rc<ModuleGraph>,
    control: Rcrc<dyn Control>,
}

impl MouseBehavior<DropTarget> for ConnectToControl {
    fn on_click(self: Box<Self>) {
        self.on_drop(None);
    }

    fn on_drop(self: Box<Self>, drop_target: Option<DropTarget>) {
        if let Some(DropTarget::Output(module, output_index)) = drop_target {
            let output_type = module.borrow().template.borrow().outputs[output_index].get_type();
            let source = AutomationSource {
                module,
                output_index,
                output_type,
            };
            let types = self.control.borrow().acceptable_automation();
            if types.into_iter().any(|t| t == source.output_type) {
                self.control.borrow_mut().connect_automation(source);
            }
            self.graph.with_gui_state_mut(|state| {
                state.engine.borrow_mut().regenerate_code();
            })
        }
        self.graph.clear_wire_preview();
    }
}

impl WidgetImpl<Renderer, DropTarget> for ModuleGraph {
    fn get_pos_impl(self: &Rc<Self>) -> Vec2D {
        0.into()
    }

    fn get_size_impl(self: &Rc<Self>) -> Vec2D {
        (TAB_BODY_WIDTH, TAB_BODY_HEIGHT).into()
    }

    fn get_mouse_behavior_impl(
        self: &Rc<Self>,
        pos: Vec2D,
        mods: &MouseMods,
    ) -> MaybeMouseBehavior {
        let pos = self.translate_screen_pos(pos);
        let children = self.children.borrow();
        if let Some(widget) = &children.detail_menu {
            let local_pos = pos - widget.get_pos();
            if local_pos.inside(widget.get_size()) {
                return widget.get_mouse_behavior(pos, mods);
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
        let pos = self.translate_screen_pos(pos);
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
        self.with_gui_state_mut(|state| {
            state.set_tooltip(Tooltip {
                text: "Double-click to add a new module.".to_owned(),
                interaction: vec![
                    InteractionHint::Scroll,
                    InteractionHint::DoubleClick,
                    InteractionHint::LeftClickAndDrag,
                ],
            })
        });
        Some(())
    }

    fn on_scroll_impl(self: &Rc<Self>, pos: Vec2D, delta: f32) -> Option<()> {
        let center = self.get_size() * 0.5;
        let old_pos = self.translate_screen_pos(center);
        let mut state = self.state.borrow_mut();
        state.zoom *= (1.0 + delta * 0.8);
        let z2 = state.zoom;
        // Black magic algebra voodoo
        state.offset = center / z2 - old_pos;
        Some(())
    }

    fn get_drop_target_impl(self: &Rc<Self>, pos: Vec2D) -> Option<DropTarget> {
        let pos = self.translate_screen_pos(pos);
        self.get_drop_target_children(pos)
    }

    fn draw_impl(self: &Rc<Self>, g: &mut GrahpicsWrapper) {
        let mouse_pos = self.parents.gui.get_mouse_pos() - Vec2D::new(0.0, HEADER_HEIGHT);
        let mouse_pos = self.translate_screen_pos(mouse_pos);
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

    fn on_removed_impl(self: &Rc<Self>) {
        self.state.borrow_mut().graph.borrow_mut().current_widget = None;
    }
}
