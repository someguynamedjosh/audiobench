use super::WireTracker;
use crate::engine::parts as ep;
use crate::gui::action::{
    ConnectInput, ConnectOutput, DragModule, DropTarget, GuiRequest, MouseAction,
};
use crate::gui::constants::*;
use crate::gui::graphics::{GrahpicsWrapper, HAlign, VAlign};
use crate::gui::module_widgets::ModuleWidget;
use crate::gui::{InteractionHint, MouseMods, Tooltip};
use crate::registry::Registry;
use shared_util::prelude::*;
use std::f32::consts::PI;

struct InputJack {
    label: String,
    tooltip: Tooltip,
    icon: usize,
    small_icon: Option<usize>,
    pos: (f32, f32),
}

impl InputJack {
    fn create(
        label: String,
        tooltip: String,
        mut icon: usize,
        custom_icon: Option<usize>,
        x: f32,
        y: f32,
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
            pos: (x, y),
        }
    }

    fn mouse_in_bounds(&self, mouse_pos: (f32, f32)) -> bool {
        let mouse_pos = (
            mouse_pos.0 - self.pos.0 + JACK_SIZE,
            mouse_pos.1 - self.pos.1,
        );
        mouse_pos.inside((JACK_SIZE * 2.0, JACK_SIZE))
    }

    fn draw(
        &self,
        g: &mut GrahpicsWrapper,
        default: Option<&ep::DefaultInput>,
        show_label: bool,
        mute: bool,
    ) {
        g.push_state();
        g.apply_offset(self.pos.0, self.pos.1);
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
            g.fill_pie(-JS, 0.0, JS, 0.0, 0.0, PI * 2.0);
            g.fill_rect(-JS / 2.0, 0.0, JS / 2.0, JS);
            g.draw_icon(default.icon, X + JIP, Y + JIP, JS - JIP * 2.0);
        }

        g.fill_rounded_rect(0.0, 0.0, JS, JS, CS);
        g.fill_rect(0.0, 0.0, CS, JS);

        if let Some(small_icon) = self.small_icon {
            const JSIS: f32 = JACK_SMALL_ICON_SIZE;
            const MINI_X: f32 = JS - JSIS / 2.0;
            const MINI_Y: f32 = JS - JSIS - JIP;
            g.fill_rounded_rect(
                MINI_X - JIP,
                MINI_Y - JIP,
                JSIS + JIP * 2.0,
                JSIS + JIP * 2.0,
                CS,
            );
            g.draw_icon(small_icon, MINI_X, MINI_Y, JSIS);
        }
        g.draw_icon(self.icon, JIP, JIP, JS - JIP * 2.0);

        if show_label && !mute {
            const H: HAlign = HAlign::Right;
            const B: VAlign = VAlign::Bottom;
            const C: VAlign = VAlign::Center;
            const T: VAlign = VAlign::Top;
            if let Some(default) = &default {
                const X: f32 = -100.0 - GRID_P - JS;
                g.write_text(FONT_SIZE, X, -JS / 2.0, 100.0, JS, H, B, 1, &self.label);
                let text = format!("({})", default.name);
                g.write_text(FONT_SIZE, X, JS / 2.0, 100.0, JS, H, T, 1, &text);
            } else {
                g.write_text(FONT_SIZE, -104.0, 0.0, 100.0, JS, H, C, 1, &self.label);
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
    pos: (f32, f32),
}

impl OutputJack {
    fn create(
        label: String,
        tooltip: String,
        mut icon: usize,
        custom_icon: Option<usize>,
        x: f32,
        y: f32,
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
            pos: (x, y),
        }
    }

    fn mouse_in_bounds(&self, mouse_pos: (f32, f32)) -> bool {
        let mouse_pos = (mouse_pos.0 - self.pos.0, mouse_pos.1 - self.pos.1);
        mouse_pos.inside((JACK_SIZE, JACK_SIZE))
    }

    fn draw(&self, g: &mut GrahpicsWrapper, show_label: bool, mute: bool) {
        g.push_state();
        g.apply_offset(self.pos.0, self.pos.1);

        const JS: f32 = JACK_SIZE;
        const CS: f32 = CORNER_SIZE;
        if mute {
            g.set_color(&COLOR_FG0);
        } else {
            g.set_color(&COLOR_FG1);
        }
        g.fill_rounded_rect(0.0, 0.0, JS, JS, CS);
        g.fill_rect(JS - CS, 0.0, CS, JS);

        const JIP: f32 = JACK_ICON_PADDING;
        if let Some(small_icon) = self.small_icon {
            const JSIS: f32 = JACK_SMALL_ICON_SIZE;
            const MINI_X: f32 = -JSIS / 2.0;
            const MINI_Y: f32 = JS - JSIS - JIP;
            g.fill_rounded_rect(
                MINI_X - JIP,
                MINI_Y - JIP,
                JSIS + JIP * 2.0,
                JSIS + JIP * 2.0,
                CS,
            );
            g.draw_icon(small_icon, MINI_X, MINI_Y, JSIS);
        }
        g.draw_icon(self.icon, JIP, JIP, JS - JIP * 2.0);

        if show_label && !mute {
            const H: HAlign = HAlign::Left;
            const V: VAlign = VAlign::Center;
            g.write_text(FONT_SIZE, JS + 4.0, 0.0, 100.0, JS, H, V, 1, &self.label);
        }

        g.pop_state();
    }
}

pub struct Module {
    pub(super) module: Rcrc<ep::Module>,
    pub(super) size: (f32, f32),
    label: String,
    inputs: Vec<InputJack>,
    outputs: Vec<OutputJack>,
    widgets: Vec<(Box<dyn ModuleWidget>, usize)>,
}

impl Drop for Module {
    fn drop(&mut self) {
        self.module.borrow_mut().feedback_data = None;
    }
}

impl Module {
    fn jack_y(index: i32) -> f32 {
        coord(index) + JACK_SIZE / 2.0
    }

    pub fn input_position(module: &ep::Module, input_index: i32) -> (f32, f32) {
        let module_pos = (module.pos.0 as f32, module.pos.1 as f32);
        (
            module_pos.0 + JACK_SIZE,
            module_pos.1 + Self::jack_y(input_index),
        )
    }

    pub fn output_position(module: &ep::Module, output_index: i32) -> (f32, f32) {
        let module_pos = (module.pos.0 as f32, module.pos.1 as f32);
        let module_size = module.template.borrow().size;
        let module_width = fatgrid(module_size.0) + MODULE_IO_WIDTH * 2.0 + JACK_SIZE;
        (
            module_pos.0 + module_width,
            module_pos.1 + Self::jack_y(output_index),
        )
    }

    pub fn create(registry: &Registry, module: Rcrc<ep::Module>) -> Self {
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
            .map(|wo| wo.instantiate(registry, module_autocons, module_staticons))
            .collect();

        let size = (
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
                JACK_SIZE,
                coord(index as i32),
            ));
        }
        let mut outputs = Vec::new();
        for (index, output) in template_ref.outputs.iter().enumerate() {
            outputs.push(OutputJack::create(
                output.borrow_label().to_owned(),
                output.borrow_tooltip().to_owned(),
                output.get_icon_index(),
                output.get_custom_icon_index(),
                size.0 - JACK_SIZE,
                coord(index as i32),
            ));
        }

        let feedback_data_len = template_ref.feedback_data_len;
        drop(template_ref);
        // There should only be one instance of the GUI at a time.
        assert!(module_ref.feedback_data.is_none());
        module_ref.feedback_data = Some(rcrc(vec![0.0; feedback_data_len]));
        drop(module_ref);

        Self {
            module,
            size,
            label,
            inputs,
            outputs,
            widgets,
        }
    }

    pub(super) fn get_pos(&self) -> (f32, f32) {
        let pos = self.module.borrow().pos;
        (pos.0 as f32, pos.1 as f32)
    }

    pub fn respond_to_mouse_press(
        &self,
        mouse_pos: (f32, f32),
        mods: &MouseMods,
    ) -> Option<Box<dyn MouseAction>> {
        let pos = self.get_pos();
        let mouse_pos = (mouse_pos.0 - pos.0, mouse_pos.1 - pos.1);
        if !mouse_pos.inside(self.size) {
            return None;
        }
        for (widget, _) in &self.widgets {
            let wpos = widget.get_position();
            let local_pos = mouse_pos.sub(wpos);
            if local_pos.inside(widget.get_bounds()) {
                let action = widget.respond_to_mouse_press(local_pos, mods, pos);
                if !action.is_none() {
                    return action;
                }
            }
        }
        for (index, input) in self.inputs.iter().enumerate() {
            if input.mouse_in_bounds(mouse_pos) {
                return Some(Box::new(ConnectInput::new(Rc::clone(&self.module), index)));
            }
        }
        for (index, output) in self.outputs.iter().enumerate() {
            if output.mouse_in_bounds(mouse_pos) {
                return Some(Box::new(ConnectOutput::new(Rc::clone(&self.module), index)));
            }
        }
        if mods.right_click {
            GuiRequest::RemoveModule(Rc::clone(&self.module)).into()
        } else {
            Some(Box::new(DragModule::new(Rc::clone(&self.module))))
        }
    }

    pub fn get_drop_target_at(&self, mouse_pos: (f32, f32)) -> DropTarget {
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

    pub(super) fn get_tooltip_at(&self, mouse_pos: (f32, f32)) -> Option<Tooltip> {
        let pos = self.get_pos();
        let mouse_pos = (mouse_pos.0 - pos.0, mouse_pos.1 - pos.1);
        if !mouse_pos.inside(self.size) {
            return None;
        }
        for (widget, _) in &self.widgets {
            let pos = widget.get_position();
            let local_pos = (mouse_pos.0 - pos.0, mouse_pos.1 - pos.1);
            if local_pos.inside(widget.get_bounds()) {
                let tooltip = widget.get_tooltip_at(local_pos);
                if !tooltip.is_none() {
                    return tooltip;
                }
            }
        }
        for input in self.inputs.iter() {
            if input.mouse_in_bounds(mouse_pos) {
                return Some(input.tooltip.clone());
            }
        }
        for output in self.outputs.iter() {
            if output.mouse_in_bounds(mouse_pos) {
                return Some(output.tooltip.clone());
            }
        }
        // TODO: Tooltip text?
        Some(Tooltip {
            text: "".to_owned(),
            interaction: InteractionHint::LeftClickAndDrag | InteractionHint::RightClick,
        })
    }

    fn draw_wires(&self, g: &mut GrahpicsWrapper, pos: (f32, f32)) {
        let mut wire_tracker = WireTracker::new(self.size);
        for (widget, _) in &self.widgets {
            widget.add_wires(&mut wire_tracker);
        }
        wire_tracker.draw_wires(g, pos);
        for (index, jack) in self.module.borrow().inputs.iter().enumerate() {
            let y = coord(index as i32) + grid(1) / 2.0;
            if let ep::InputConnection::Wire(module, output_index) = jack {
                let output_index = *output_index as i32;
                let module_ref = module.borrow();
                let (ox, oy) = Self::output_position(&*module_ref, output_index);
                super::draw_io_wire(g, pos.0 + JACK_SIZE, pos.1 + y, ox, oy);
            }
        }
    }

    pub(super) fn draw(
        &self,
        g: &mut GrahpicsWrapper,
        mouse_pos: (f32, f32),
        highlight: Option<(bool, ep::JackType)>,
        layer_index: usize,
    ) {
        let pos = self.get_pos();

        if layer_index == 0 {
            g.draw_inset_box_shadow(
                pos.0 + JACK_SIZE,
                pos.1,
                self.size.0 - JACK_SIZE,
                self.size.1,
                MODULE_SHADOW_RADIUS,
                CORNER_SIZE,
            );
        } else if layer_index == 1 {
            g.set_color(&COLOR_FG1);
            self.draw_wires(g, pos);
        } else if layer_index == 2 {
            g.push_state();
            g.apply_offset(pos.0, pos.1);
            let mouse_pos = mouse_pos.sub(pos);

            const CS: f32 = CORNER_SIZE;
            const JS: f32 = JACK_SIZE;
            const MIW: f32 = MODULE_IO_WIDTH;

            g.set_color(&COLOR_BG1);
            g.fill_rounded_rect(JS, 0.0, self.size.0 - JS, self.size.1, CS);
            g.set_color(&COLOR_BG2);
            g.fill_rect(JS + MIW, 0.0, self.size.0 - MIW * 2.0 - JS, self.size.1);

            g.set_color(&COLOR_FG1);
            g.write_text(
                FONT_SIZE,
                MODULE_IO_WIDTH,
                -20.0,
                self.size.0,
                20.0,
                HAlign::Left,
                VAlign::Bottom,
                1,
                &self.label,
            );

            let module_ref = self.module.borrow();
            let template_ref = module_ref.template.borrow();
            let hovering = mouse_pos.inside(self.size);
            for input_index in 0..self.inputs.len() {
                let input = &self.inputs[input_index];
                let jack = &template_ref.inputs[input_index];
                let mute = if let Some((outs, typ)) = highlight {
                    outs || typ != jack.get_type()
                } else {
                    false
                };
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
            for output_index in 0..self.outputs.len() {
                let output = &self.outputs[output_index];
                let jack = &template_ref.outputs[output_index];
                let mute = if let Some((outs, typ)) = highlight {
                    !outs || typ != jack.get_type()
                } else {
                    false
                };
                output.draw(g, hovering, mute);
            }
            let feedback_data_ref = module_ref.feedback_data.as_ref().unwrap().borrow();
            let feedback_data = &feedback_data_ref[..];
            let mut fdi = 0;
            let highlight = highlight == Some((false, ep::JackType::Audio));
            for (widget, segment_len) in &self.widgets {
                widget.draw(g, highlight, pos, &feedback_data[fdi..fdi + segment_len]);
                fdi += segment_len;
            }

            g.pop_state();
            g.set_color(&COLOR_FG1);
        } else if layer_index == 3 {
            g.set_color(&COLOR_FG1);
            g.set_alpha(0.2);
            self.draw_wires(g, pos);
        }
    }
}
