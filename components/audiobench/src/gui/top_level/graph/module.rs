use super::{GraphHighlightMode, ModuleGraph, WireTracker};
use crate::engine::parts as ep;
use crate::gui::constants::*;
use crate::gui::graphics::{HAlign, VAlign};
use crate::gui::module_widgets::ModuleWidget;
use crate::gui::{InteractionHint, Tooltip};
use crate::registry::Registry;
use crate::scui_config::{DropTarget, Renderer};
use owning_ref::OwningRef;
use scui::{
    ChildHolder, MaybeMouseBehavior, MouseMods, OnClickBehavior, Vec2D, Widget, WidgetImpl,
};
use shared_util::prelude::*;
use std::cell::Ref;
use std::f32::consts::PI;

struct InputJack {
    label: String,
    tooltip: Tooltip,
    icon: usize,
    small_icon: Option<usize>,
    pos: Vec2D,
}

impl InputJack {
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
                interaction: InteractionHint::LeftClickAndDrag | InteractionHint::LeftClick,
            },
            icon,
            small_icon,
            pos,
        }
    }

    fn mouse_in_bounds(&self, mouse_pos: Vec2D) -> bool {
        (mouse_pos - self.pos + (JACK_SIZE, 0.0)).inside((JACK_SIZE * 2.0, JACK_SIZE).into())
    }

    fn draw(
        &self,
        g: &mut Renderer,
        default: Option<&ep::DefaultInput>,
        show_label: bool,
        mute: bool,
    ) {
        g.push_state();
        g.translate(self.pos);
        const JS: f32 = JACK_SIZE;
        const CS: f32 = CORNER_SIZE;
        const JIP: f32 = JACK_ICON_PADDING;

        if mute {
            g.set_color(&COLOR_FG0);
        } else {
            g.set_color(&COLOR_FG1);
        }
        if let Some(default) = &default {
            const X: f32 = -JS;
            const Y: f32 = (JS - JS) / 2.0;
            g.draw_pie((-JS, 0.0), JS, 0.0, 0.0, PI * 2.0);
            g.draw_rect((-JS / 2.0, 0.0), (JS / 2.0, JS));
            g.draw_icon(default.icon, Vec2D::new(X, Y) + JIP, JS - JIP * 2.0);
        }

        g.draw_rounded_rect(0, JS, CS);
        g.draw_rect(0, (CS, JS));

        if let Some(small_icon) = self.small_icon {
            const JSIS: f32 = JACK_SMALL_ICON_SIZE;
            const MINI_COORD: Vec2D = Vec2D::new(JS - JSIS / 2.0, JS - JSIS - JIP);
            g.draw_rounded_rect(MINI_COORD - JIP, JSIS + JIP * 2.0, CS);
            g.draw_icon(small_icon, MINI_COORD, JSIS);
        }
        g.draw_icon(self.icon, JIP, JS - JIP * 2.0);

        if show_label && !mute {
            if let Some(default) = &default {
                const X: f32 = -100.0 - GRID_P - JS;
                g.draw_text(
                    FONT_SIZE,
                    (X, -JS / 2.0),
                    (100.0, JS),
                    (1, 1),
                    1,
                    &self.label,
                );
                let text = format!("({})", default.name);
                g.draw_text(
                    FONT_SIZE,
                    (X, JS / 2.0),
                    (100.0, JS),
                    (1, 1),
                    1,
                    &self.label,
                );
            } else {
                g.draw_text(FONT_SIZE, (-104, 0), (100.0, JS), (1, 0), 1, &self.label);
            }
        }

        g.pop_state();
    }
}

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
        inputs: Vec<InputJack>,
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
    fn jack_y(index: i32) -> f32 {
        coord(index) + JACK_SIZE / 2.0
    }

    pub fn input_position(module: &ep::Module, input_index: i32) -> Vec2D {
        let module_pos = (module.pos.0 as f32, module.pos.1 as f32);
        (
            module_pos.0 + JACK_SIZE,
            module_pos.1 + Self::jack_y(input_index),
        )
            .into()
    }

    pub fn output_position(module: &ep::Module, output_index: i32) -> Vec2D {
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
        let module_autocons = &module_ref.autocons;
        let module_staticons = &module_ref.staticons;
        let widgets = template_ref
            .widget_outlines
            .iter()
            .map(|wo| wo.instantiate(parent, module_autocons, module_staticons))
            .collect();

        let size = Vec2D::new(
            fatgrid(grid_size.0) + MIW * 2.0 + JACK_SIZE,
            fatgrid(grid_size.1),
        );
        let mut inputs = Vec::new();
        for (index, input) in template_ref.inputs.iter().enumerate() {
            inputs.push(InputJack::create(
                input.borrow_label().to_owned(),
                input.borrow_tooltip().to_owned(),
                input.get_icon_index(),
                input.get_custom_icon_index(),
                (JACK_SIZE, coord(index as i32)).into(),
            ));
        }
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

        let feedback_data_len = template_ref.feedback_data_len;
        drop(template_ref);
        // There should only be one instance of the GUI at a time.
        assert!(module_ref.feedback_data.is_none());
        module_ref.feedback_data = Some(rcrc(vec![0.0; feedback_data_len]));
        drop(module_ref);

        let state = ModuleState {
            module,
            size,
            label,
            inputs,
            outputs,
            widgets,
        };

        Rc::new(Self::create(parent, state))
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
            widget.add_wires(&mut wire_tracker);
        }
        wire_tracker.draw_wires(g, pos);
        for (index, jack) in state.module.borrow().inputs.iter().enumerate() {
            let y = coord(index as i32) + grid(1) / 2.0;
            if let ep::InputConnection::Wire(module, output_index) = jack {
                let output_index = *output_index as i32;
                let module_ref = module.borrow();
                let op = Self::output_position(&*module_ref, output_index);
                super::draw_io_wire(g, pos + (JACK_SIZE, y), pos);
            }
        }
    }

    pub fn represents_module(&self, module: &Rcrc<ep::Module>) -> bool {
        Rc::ptr_eq(&self.state.borrow().module, module)
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
            let wpos = widget.get_pos();
            let local_pos = mouse_pos - wpos;
            if local_pos.inside(widget.get_size()) {
                let action = widget.get_mouse_behavior(local_pos, mods);
                if !action.is_none() {
                    return action;
                }
            }
        }
        for (index, input) in state.inputs.iter().enumerate() {
            if input.mouse_in_bounds(mouse_pos) {
                // return MouseAction::ConnectInput(Rc::clone(&state.module), index);
            }
        }
        for (index, output) in state.outputs.iter().enumerate() {
            if output.mouse_in_bounds(mouse_pos) {
                // return MouseAction::ConnectOutput(Rc::clone(&state.module), index);
            }
        }
        if mods.right_click {
            // MouseAction::RemoveModule(Rc::clone(&state.module))
        } else {
            // MouseAction::MoveModule(Rc::clone(&state.module), state.module.borrow().pos)
        }
        None
    }

    /*
    fn get_drop_target_at(&self, mouse_pos: Vec2D) -> DropTarget {
        let pos = self.get_pos();
        let mouse_pos = (mouse_pos.0 - pos.0, mouse_pos.1 - pos.1);
        if !mouse_pos.inside(self.size) {
            return DropTarget::None;
        }
        for (widget, _) in &self.widgets {
            let pos = widget.get_position();
            let local_pos = (mouse_pos.0 - pos.0, mouse_pos.1 - pos.1);
            if local_pos.inside(widget.get_bounds()) {
                let target = widget.get_drop_target_at(local_pos);
                if !target.is_none() {
                    return target;
                }
            }
        }
        for (index, input) in self.inputs.iter().enumerate() {
            if input.mouse_in_bounds(mouse_pos) {
                return DropTarget::Input(Rc::clone(&self.module), index);
            }
        }
        for (index, output) in self.outputs.iter().enumerate() {
            if output.mouse_in_bounds(mouse_pos) {
                return DropTarget::Output(Rc::clone(&self.module), index);
            }
        }
        DropTarget::None
    }
    */

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
            let mouse_pos = self.parents.gui.get_mouse_pos() - pos;

            const CS: f32 = CORNER_SIZE;
            const JS: f32 = JACK_SIZE;
            const MIW: f32 = MODULE_IO_WIDTH;

            g.set_color(&COLOR_BG1);
            g.draw_rounded_rect((JS, 0.0), size - (JS, 0.0), CS);
            g.set_color(&COLOR_BG2);
            g.draw_rect((JS + MIW, 0.0), size - (MIW * 2.0 - JS, 0.0));

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
            let hovering = mouse_pos.inside(size);
            for input_index in 0..state.inputs.len() {
                let input = &state.inputs[input_index];
                let jack = &template_ref.inputs[input_index];
                let mute = highlight.is_some()
                    && highlight != Some(GraphHighlightMode::ReceivesType(jack.get_type()));
                if let ep::InputConnection::Default(default_index) = module_ref.inputs[input_index]
                {
                    input.draw(
                        g,
                        Some(&jack.borrow_default_options()[default_index]),
                        hovering,
                        mute,
                    );
                } else {
                    input.draw(g, None, hovering, mute);
                }
            }
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

            g.pop_state();
            g.set_color(&COLOR_FG1);
        } else if layer_index == 3 {
            g.set_color(&COLOR_FG1);
            g.set_alpha(0.2);
            g.translate(pos * -1.0);
            self.draw_wires(g, pos);
        }
    }
}
