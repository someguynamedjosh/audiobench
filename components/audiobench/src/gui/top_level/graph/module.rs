use crate::{
    engine::parts as ep,
    gui::{
        constants::*,
        module_widgets::ModuleWidget,
        top_level::graph::{GraphHighlightMode, ModuleGraph, OutputViewRenderer, WireTracker},
        {InteractionHint, Tooltip},
    },
    scui_config::{DropTarget, MaybeMouseBehavior, Renderer},
};
use scui::{MouseBehavior, MouseMods, OnClickBehavior, Vec2D, Widget, WidgetImpl};
use shared_util::prelude::*;

struct OutputJack {
    label: String,
    tooltip: Tooltip,
    icon: usize,
    small_icon: Option<usize>,
    pos: Vec2D,
}

impl OutputJack {
    fn create(
        label: String,
        tooltip: String,
        typ: ep::JackType,
        mut icon: usize,
        custom_icon: Option<usize>,
        pos: Vec2D,
    ) -> Self {
        let small_icon = if let Some(custom) = custom_icon {
            let small_icon = icon;
            icon = custom;
            Some(small_icon)
        } else {
            None
        };
        Self {
            label,
            tooltip: Tooltip {
                text: tooltip,
                interaction: vec![
                    InteractionHint::LeftClickAndDrag,
                    InteractionHint::ProducesOutput(typ),
                ],
            },
            icon,
            small_icon,
            pos,
        }
    }

    fn mouse_in_bounds(&self, mouse_pos: Vec2D) -> bool {
        (mouse_pos - self.pos).inside((JACK_SIZE, JACK_SIZE).into())
    }

    fn draw(&self, g: &mut Renderer, show_label: bool, mute: bool) {
        g.push_state();
        g.translate(self.pos);

        const JS: f32 = JACK_SIZE;
        const CS: f32 = CORNER_SIZE;
        if mute {
            g.set_color(&COLOR_FG0);
        } else {
            g.set_color(&COLOR_FG1);
        }
        g.draw_rounded_rect(0, JS, CS);
        g.draw_rect((JS - CS, 0.0), (CS, JS));

        const JIP: f32 = JACK_ICON_PADDING;
        if let Some(small_icon) = self.small_icon {
            const JSIS: f32 = JACK_SMALL_ICON_SIZE;
            const MINI_COORD: Vec2D = Vec2D::new(-JSIS / 2.0, JS - JSIS - JIP);
            g.draw_rounded_rect(MINI_COORD - JIP, JSIS + JIP * 2.0, CS);
            g.draw_icon(small_icon, MINI_COORD, JSIS);
        }
        g.draw_icon(self.icon, JIP, JS - JIP * 2.0);

        if show_label && !mute {
            g.draw_text(
                FONT_SIZE,
                (JS + 4.0, 0.0),
                (100.0, JS),
                (-1, 0),
                1,
                &self.label,
            );
        }

        g.pop_state();
    }
}

scui::widget! {
    pub Module
    State {
        module: Rcrc<ep::Module>,
        size: Vec2D,
        label: String,
        outputs: Vec<OutputJack>,
        output_view_data: Vec<Vec<f32>>,
        widgets: Vec<Box<dyn ModuleWidget>>
    }
    Parents {
        graph: Rc<ModuleGraph>
    }
}

impl Module {
    fn jack_y(index: usize) -> f32 {
        coord(index as i32) + JACK_SIZE / 2.0
    }

    pub fn output_position(module: &ep::Module, output_index: usize) -> Vec2D {
        let module_pos = (module.pos.0 as f32, module.pos.1 as f32);
        let module_size = module.template.borrow().size;
        let module_width = fatgrid(module_size.0) + MODULE_IO_WIDTH * 2.0 + JACK_SIZE;
        (
            module_pos.0 + module_width,
            module_pos.1 + Self::jack_y(output_index),
        )
            .into()
    }

    pub fn new(parent: &impl ModuleParent, module: Rcrc<ep::Module>) -> Rc<Self> {
        const MIW: f32 = MODULE_IO_WIDTH;
        let module_ref = module.borrow_mut();
        let template_ref = module_ref.template.borrow();
        let grid_size = template_ref.size;
        let label = template_ref.label.clone();
        let module_controls = &module_ref.controls;

        let size = Vec2D::new(
            fatgrid(grid_size.0) + MIW * 2.0 + JACK_SIZE,
            fatgrid(grid_size.1),
        );
        let mut outputs = Vec::new();
        for (index, output) in template_ref.outputs.iter().enumerate() {
            outputs.push(OutputJack::create(
                output.borrow_label().to_owned(),
                output.borrow_tooltip().to_owned(),
                output.get_type(),
                output.get_icon_index(),
                output.get_custom_icon_index(),
                (size.x - JACK_SIZE, coord(index as i32)).into(),
            ));
        }

        let state = ModuleState {
            module: Rc::clone(&module),
            size,
            label,
            outputs,
            output_view_data: Vec::new(),
            widgets: Vec::new(),
        };

        let this = Rc::new(Self::create(parent, state));
        let widgets = template_ref
            .widget_outlines
            .iter()
            .map(|wo| wo.instantiate(&this, module_controls))
            .collect();
        this.state.borrow_mut().widgets = widgets;

        drop(template_ref);
        drop(module_ref);

        this
    }

    pub fn take_feedback_data(self: &Rc<Self>, data: Vec<f32>, widget_index: usize) {
        self.state.borrow().widgets[widget_index].take_feedback_data(data);
    }

    pub fn take_output_view_data(self: &Rc<Self>, data: Vec<Vec<f32>>) {
        assert_eq!(
            data.len(),
            self.state
                .borrow()
                .module
                .borrow()
                .template
                .borrow()
                .outputs
                .len()
        );
        self.state.borrow_mut().output_view_data = data;
    }

    fn draw_wires(self: &Rc<Self>, g: &mut Renderer, pos: Vec2D) {
        let mut wire_tracker = WireTracker::new(self.get_size());
        let state = self.state.borrow();
        for widget in &state.widgets {
            let center = widget.get_pos() + widget.get_size() / 2.0;
            let input_style = widget.use_input_style_wires();
            if let Some(control) = widget.represented_control() {
                for source in control.borrow().get_connected_automation() {
                    let source_coord =
                        Module::output_position(&*source.module.borrow(), source.output_index);
                    wire_tracker.add_wire(source_coord, center, input_style);
                }
            }
        }
        wire_tracker.draw_wires(g, pos);
    }

    pub fn get_real_module(self: &Rc<Self>) -> Rcrc<ep::Module> {
        Rc::clone(&self.state.borrow().module)
    }

    pub fn represents_module(self: &Rc<Self>, module: &Rcrc<ep::Module>) -> bool {
        Rc::ptr_eq(&self.state.borrow().module, module)
    }

    pub fn is_hovered(self: &Rc<Self>) -> bool {
        self.parents.graph.is_hovered_module(self)
    }
}

pub struct DragModule {
    module: Rc<Module>,
    real_pos: Vec2D,
}

impl DragModule {
    fn new(module: Rc<Module>) -> Self {
        let state = module.state.borrow();
        let real_module = state.module.borrow();
        let real_pos = Vec2D::from(real_module.pos);
        drop(real_module);
        drop(state);
        Self { module, real_pos }
    }
}

impl MouseBehavior<DropTarget> for DragModule {
    fn on_drag(&mut self, delta: Vec2D, mods: &MouseMods) {
        let zoom = self.module.parents.graph.get_zoom();
        self.real_pos += delta / zoom;
        let state = self.module.state.borrow_mut();
        let mut module = state.module.borrow_mut();
        if mods.snap {
            const INTERVAL: f32 = grid(1) + GRID_P;
            module.pos.0 = (self.real_pos.x / INTERVAL).round() * INTERVAL;
            module.pos.1 = (self.real_pos.y / INTERVAL).round() * INTERVAL;
        } else {
            module.pos.0 = self.real_pos.x;
            module.pos.1 = self.real_pos.y;
        }
        self.module.with_gui_state_mut(|state| {
            state.set_tooltip(Tooltip {
                text: format!(""),
                interaction: vec![
                    InteractionHint::LeftClickAndDrag,
                    InteractionHint::SnappingModifier,
                ],
            });
        });
    }
}

impl WidgetImpl<Renderer, DropTarget> for Module {
    fn get_pos_impl(self: &Rc<Self>) -> Vec2D {
        let pos = self.state.borrow().module.borrow().pos;
        (pos.0 as f32, pos.1 as f32).into()
    }

    fn get_size_impl(self: &Rc<Self>) -> Vec2D {
        self.state.borrow().size
    }

    fn get_mouse_behavior_impl(
        self: &Rc<Self>,
        mouse_pos: Vec2D,
        mods: &MouseMods,
    ) -> MaybeMouseBehavior {
        let state = self.state.borrow();
        for widget in &state.widgets {
            ris!(widget.get_mouse_behavior(mouse_pos, mods));
        }
        for (index, output) in state.outputs.iter().enumerate() {
            if output.mouse_in_bounds(mouse_pos) {
                let g = &self.parents.graph;
                return Some(g.connect_from_source_behavior(Rc::clone(&state.module), index));
            }
        }
        if mods.right_click {
            let graph = Rc::clone(&self.parents.graph);
            let module = Rc::clone(&state.module);
            OnClickBehavior::wrap(move || graph.remove_module(&module))
        } else {
            Some(Box::new(DragModule::new(Rc::clone(self))))
        }
    }

    fn get_drop_target_impl(self: &Rc<Self>, mouse_pos: Vec2D) -> Option<DropTarget> {
        let state = self.state.borrow();
        for widget in &state.widgets {
            if !(mouse_pos - widget.get_pos()).inside(widget.get_size()) {
                continue;
            }
            if let Some(control) = widget.represented_control() {
                return Some(DropTarget::Control(control));
            }
        }
        for (index, output) in state.outputs.iter().enumerate() {
            if output.mouse_in_bounds(mouse_pos) {
                return Some(DropTarget::Output(Rc::clone(&state.module), index));
            }
        }
        for widget in &state.widgets {
            ris!(widget.get_drop_target(mouse_pos));
        }
        None
    }

    fn on_hover_impl(self: &Rc<Self>, pos: Vec2D) -> Option<()> {
        self.parents.graph.set_hovered_module(Rc::clone(self));
        let state = self.state.borrow();
        for widget in &state.widgets {
            if let Some(i) = widget.on_hover(pos) {
                if let Some(control) = widget.represented_control() {
                    self.with_gui_state_mut(|state| {
                        state.add_automation_to_tooltip(&control);
                    });
                }
                return Some(i);
            }
        }

        let mut tooltip = Tooltip {
            text: "".to_owned(),
            interaction: vec![
                InteractionHint::LeftClickAndDrag,
                InteractionHint::RightClick,
            ],
        };
        for output in state.outputs.iter() {
            if output.mouse_in_bounds(pos) {
                tooltip = output.tooltip.clone();
            }
        }
        drop(state);
        self.with_gui_state_mut(|state| {
            state.set_tooltip(tooltip);
        });
        Some(())
    }

    fn draw_impl(self: &Rc<Self>, g: &mut Renderer) {
        let pos = self.get_pos();
        let size = self.get_size();
        let state = self.state.borrow();
        let layer_index = self.parents.graph.get_current_draw_layer();
        let highlight = self.parents.graph.get_highlight_mode();

        if layer_index == 0 {
            g.draw_inset_box_shadow(
                (JACK_SIZE, 0.0),
                size - (JACK_SIZE, 0.0),
                MODULE_SHADOW_RADIUS,
                CORNER_SIZE,
            );
        } else if layer_index == 1 {
            g.translate(pos * -1.0);
            g.set_color(&COLOR_FG1);
            self.draw_wires(g, pos);
        } else if layer_index == 2 {
            const CS: f32 = CORNER_SIZE;
            const JS: f32 = JACK_SIZE;
            const MIW: f32 = MODULE_IO_WIDTH;

            g.set_color(&COLOR_BG1);
            g.draw_rounded_rect((JS, 0.0), size - (JS, 0.0), CS);
            g.set_color(&COLOR_BG2);
            g.draw_rect((JS + MIW, 0.0), size - (MIW * 2.0 + JS, 0.0));

            g.set_color(&COLOR_FG1);
            g.draw_text(
                FONT_SIZE,
                (MODULE_IO_WIDTH, -20.0),
                (size.x, 20.0),
                (-1, 1),
                1,
                &state.label,
            );

            let module_ref = state.module.borrow();
            let template_ref = module_ref.template.borrow();
            let hovering = self.is_hovered();
            for output_index in 0..state.outputs.len() {
                let output = &state.outputs[output_index];
                let jack = &template_ref.outputs[output_index];
                let dim = if let GraphHighlightMode::ProducesType(typ) = highlight {
                    typ != jack.get_type()
                } else {
                    false
                };
                output.draw(g, hovering, dim);
            }
            for widget in &state.widgets {
                widget.draw(g);
            }

            g.set_color(&COLOR_FG1);
        } else if layer_index == 3 {
            g.set_color(&COLOR_FG1);
            g.set_alpha(0.2);
            g.translate(pos * -1.0);
            self.draw_wires(g, pos);
        } else if layer_index == 4 && self.is_hovered() {
            let module = state.module.borrow();
            let template = module.template.borrow();
            let x = size.x;
            let mut y = GRID_P + JACK_SIZE / 2.0;
            let mut index = 0;
            for output in &template.outputs {
                let data = if state.output_view_data.len() > index {
                    &state.output_view_data[index][..]
                } else {
                    &[][..]
                };
                g.draw_output_view(
                    data,
                    output.get_type(),
                    Vec2D::new(x, y),
                    Vec2D::new(x + grid(1), y),
                );
                y += GRID_P + JACK_SIZE;
                index += 1;
            }
        }
    }
}
