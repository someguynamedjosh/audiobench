use super::{GraphHighlightMode, ModuleGraph, WireTracker};
use crate::engine::parts as ep;
use crate::gui::constants::*;
use crate::gui::graphics::{HAlign, VAlign};
use crate::gui::module_widgets::ModuleWidget;
use crate::gui::{InteractionHint, Tooltip};
use crate::registry::Registry;
use crate::scui_config::{DropTarget, MaybeMouseBehavior, Renderer};
use scui::{ChildHolder, MouseBehavior, MouseMods, OnClickBehavior, Vec2D, Widget, WidgetImpl};
use shared_util::prelude::*;
use std::cell::Ref;
use std::f32::consts::PI;

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
                interaction: InteractionHint::LeftClickAndDrag.into(),
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
        widgets: Vec<(Box<dyn ModuleWidget>, usize)>
    }
    Parents {
        graph: Rc<ModuleGraph>
    }
}

impl Drop for Module {
    fn drop(&mut self) {
        self.state.borrow_mut().module.borrow_mut().feedback_data = None;
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
        let mut module_ref = module.borrow_mut();
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
            widgets: Vec::new(),
        };

        let this = Rc::new(Self::create(parent, state));
        let widgets = template_ref
            .widget_outlines
            .iter()
            .map(|wo| wo.instantiate(&this, module_controls))
            .collect();
        this.state.borrow_mut().widgets = widgets;

        let feedback_data_len = template_ref.feedback_data_len;
        drop(template_ref);
        // There should only be one instance of the GUI at a time.
        assert!(module_ref.feedback_data.is_none());
        module_ref.feedback_data = Some(rcrc(vec![0.0; feedback_data_len]));
        drop(module_ref);

        this
    }

    fn get_tooltip_at(&self, mouse_pos: Vec2D) -> Option<Tooltip> {
        // let state = self.state.borrow();
        // for (widget, _) in &state.widgets {
        //     let pos = widget.get_pos();
        //     let local_pos = mouse_pos - pos;
        //     if local_pos.inside(widget.get_size()) {
        //         let tooltip = widget.get_tooltip_at(local_pos);
        //         if !tooltip.is_none() {
        //             return tooltip;
        //         }
        //     }
        // }
        // for input in state.inputs.iter() {
        //     if input.mouse_in_bounds(mouse_pos) {
        //         return Some(input.tooltip.clone());
        //     }
        // }
        // for output in state.outputs.iter() {
        //     if output.mouse_in_bounds(mouse_pos) {
        //         return Some(output.tooltip.clone());
        //     }
        // }
        // TODO: Tooltip text?
        Some(Tooltip {
            text: "".to_owned(),
            interaction: InteractionHint::LeftClickAndDrag | InteractionHint::RightClick,
        })
    }

    fn draw_wires(self: &Rc<Self>, g: &mut Renderer, pos: Vec2D) {
        let mut wire_tracker = WireTracker::new(self.get_size());
        let state = self.state.borrow();
        for (widget, _) in &state.widgets {
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

    fn drag(self: &Rc<Self>, delta: Vec2D) {
        let mut state = self.state.borrow_mut();
        let mut module = state.module.borrow_mut();
        module.pos.0 += delta.x;
        module.pos.1 += delta.y;
    }

    pub fn represents_module(self: &Rc<Self>, module: &Rcrc<ep::Module>) -> bool {
        Rc::ptr_eq(&self.state.borrow().module, module)
    }

    pub fn is_hovered(self: &Rc<Self>) -> bool {
        self.parents.graph.is_hovered_module(self)
    }
}

pub struct DragModule(Rc<Module>);

impl MouseBehavior<DropTarget> for DragModule {
    fn on_drag(&mut self, delta: Vec2D, _mods: &MouseMods) {
        self.0.drag(delta);
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
        for (widget, _) in &state.widgets {
            ris!(widget.get_mouse_behavior(mouse_pos, mods));
        }
        for (index, output) in state.outputs.iter().enumerate() {
            if output.mouse_in_bounds(mouse_pos) {
                let g = &self.parents.graph;
                return Some(g.connect_from_source_behavior(Rc::clone(&state.module), index));
            }
        }
        if mods.right_click {
            // MouseAction::RemoveModule(Rc::clone(&state.module))
            unimplemented!()
        } else {
            Some(Box::new(DragModule(Rc::clone(self))))
        }
    }

    fn get_drop_target_impl(self: &Rc<Self>, mouse_pos: Vec2D) -> Option<DropTarget> {
        let state = self.state.borrow();
        for (widget, _) in &state.widgets {
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
        for (widget, _) in &state.widgets {
            ris!(widget.get_drop_target(mouse_pos));
        }
        None
    }

    fn on_hover_impl(self: &Rc<Self>, pos: Vec2D) -> Option<()> {
        self.parents.graph.set_hovered_module(Rc::clone(self));
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
                let mute = highlight.is_some()
                    && highlight != Some(GraphHighlightMode::ProducesType(jack.get_type()));
                output.draw(g, hovering, mute);
            }
            let feedback_data_ref = module_ref.feedback_data.as_ref().unwrap().borrow();
            let feedback_data = &feedback_data_ref[..];
            let mut fdi = 0;
            for (widget, segment_len) in &state.widgets {
                let data = &feedback_data[fdi..fdi + segment_len];
                widget.draw(g);
                fdi += segment_len;
            }

            g.set_color(&COLOR_FG1);
        } else if layer_index == 3 {
            g.set_color(&COLOR_FG1);
            g.set_alpha(0.2);
            g.translate(pos * -1.0);
            self.draw_wires(g, pos);
        }
    }
}
