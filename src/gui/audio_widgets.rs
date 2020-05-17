use crate::engine::parts as ep;
use crate::gui::action::{DropTarget, MouseAction};
use crate::gui::constants::*;
use crate::gui::graphics::{GrahpicsWrapper, HAlign, VAlign};
use crate::gui::module_widgets::{self, KnobEditor, ModuleWidget};
use crate::gui::{Gui, MouseMods};
use crate::util::*;
use std::f32::consts::PI;

struct InputJack {
    label: String,
    icon: usize,
    small_icon: Option<usize>,
    pos: (i32, i32),
}

impl InputJack {
    fn create(label: String, mut icon: usize, custom_icon: Option<usize>, x: i32, y: i32) -> Self {
        let small_icon = if let Some(custom) = custom_icon {
            let small_icon = icon;
            icon = custom;
            Some(small_icon)
        } else {
            None
        };
        Self {
            label,
            icon,
            small_icon,
            pos: (x, y),
        }
    }

    fn mouse_in_bounds(&self, mouse_pos: (i32, i32)) -> bool {
        let mouse_pos = (
            mouse_pos.0 - self.pos.0 + JACK_SIZE,
            mouse_pos.1 - self.pos.1,
        );
        mouse_pos.inside((JACK_SIZE * 2, JACK_SIZE))
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
        const JS: i32 = JACK_SIZE;
        const CS: i32 = CORNER_SIZE;
        const JIP: i32 = JACK_ICON_PADDING;

        if mute {
            g.set_color(&COLOR_MUTED_TEXT);
        } else {
            g.set_color(&COLOR_TEXT);
        }
        if let Some(default) = &default {
            const X: i32 = -JS;
            const Y: i32 = (JS - JS) / 2;
            g.fill_pie(-JS, 0, JS, 0, 0.0, PI * 2.0);
            g.fill_rect(-JS / 2, 0, JS / 2, JS);
            g.draw_icon(default.icon, X + JIP, Y + JIP, JS - JIP * 2);
        }

        g.fill_rounded_rect(0, 0, JS, JS, CS);
        g.fill_rect(0, 0, CS, JS);

        if let Some(small_icon) = self.small_icon {
            const JSIS: i32 = JACK_SMALL_ICON_SIZE;
            const MINI_X: i32 = JS - JSIS / 2;
            const MINI_Y: i32 = JS - JSIS - JIP;
            g.fill_rounded_rect(
                MINI_X - JIP,
                MINI_Y - JIP,
                JSIS + JIP * 2,
                JSIS + JIP * 2,
                CS,
            );
            g.draw_icon(small_icon, MINI_X, MINI_Y, JSIS);
        }
        g.draw_icon(self.icon, JIP, JIP, JS - JIP * 2);

        if show_label && !mute {
            const H: HAlign = HAlign::Right;
            const B: VAlign = VAlign::Bottom;
            const C: VAlign = VAlign::Center;
            const T: VAlign = VAlign::Top;
            if let Some(default) = &default {
                const X: i32 = -104 - JS;
                g.write_text(12, X, -JS / 2, 100, JS, H, B, 1, &self.label);
                let text = format!("({})", default.name);
                g.write_text(12, X, JS / 2, 100, JS, H, T, 1, &text);
            } else {
                g.write_text(12, -104, 0, 100, JS, H, C, 1, &self.label);
            }
        }

        g.pop_state();
    }
}

struct OutputJack {
    label: String,
    icon: usize,
    small_icon: Option<usize>,
    pos: (i32, i32),
}

impl OutputJack {
    fn create(label: String, mut icon: usize, custom_icon: Option<usize>, x: i32, y: i32) -> Self {
        let small_icon = if let Some(custom) = custom_icon {
            let small_icon = icon;
            icon = custom;
            Some(small_icon)
        } else {
            None
        };
        Self {
            label,
            icon,
            small_icon,
            pos: (x, y),
        }
    }

    fn mouse_in_bounds(&self, mouse_pos: (i32, i32)) -> bool {
        let mouse_pos = (mouse_pos.0 - self.pos.0, mouse_pos.1 - self.pos.1);
        mouse_pos.inside((JACK_SIZE, JACK_SIZE))
    }

    fn draw(&self, g: &mut GrahpicsWrapper, show_label: bool, mute: bool) {
        g.push_state();
        g.apply_offset(self.pos.0, self.pos.1);

        const JS: i32 = JACK_SIZE;
        const CS: i32 = CORNER_SIZE;
        if mute {
            g.set_color(&COLOR_MUTED_TEXT);
        } else {
            g.set_color(&COLOR_TEXT);
        }
        g.fill_rounded_rect(0, 0, JS, JS, CS);
        g.fill_rect(JS - CS, 0, CS, JS);

        const JIP: i32 = JACK_ICON_PADDING;
        if let Some(small_icon) = self.small_icon {
            const JSIS: i32 = JACK_SMALL_ICON_SIZE;
            const MINI_X: i32 = -JSIS / 2;
            const MINI_Y: i32 = JS - JSIS - JIP;
            g.fill_rounded_rect(
                MINI_X - JIP,
                MINI_Y - JIP,
                JSIS + JIP * 2,
                JSIS + JIP * 2,
                CS,
            );
            g.draw_icon(small_icon, MINI_X, MINI_Y, JSIS);
        }
        g.draw_icon(self.icon, JIP, JIP, JS - JIP * 2);

        if show_label && !mute {
            const H: HAlign = HAlign::Left;
            const V: VAlign = VAlign::Center;
            g.write_text(12, JS + 4, 0, 100, JS, H, V, 1, &self.label);
        }

        g.pop_state();
    }
}

pub struct Module {
    module: Rcrc<ep::Module>,
    size: (i32, i32),
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
    fn jack_y(index: i32) -> i32 {
        coord(index) + JACK_SIZE / 2
    }

    pub fn input_position(module: &ep::Module, input_index: i32) -> (i32, i32) {
        let module_pos = module.pos;
        (
            module_pos.0 + JACK_SIZE,
            module_pos.1 + Self::jack_y(input_index),
        )
    }

    pub fn output_position(module: &ep::Module, output_index: i32) -> (i32, i32) {
        let module_pos = module.pos;
        let module_size = module.template.borrow().size;
        let module_width = fatgrid(module_size.0) + MODULE_IO_WIDTH * 2 + JACK_SIZE;
        (
            module_pos.0 + module_width,
            module_pos.1 + Self::jack_y(output_index),
        )
    }

    pub fn create(module: Rcrc<ep::Module>) -> Self {
        const MIW: i32 = MODULE_IO_WIDTH;
        let mut module_ref = module.borrow_mut();
        let template_ref = module_ref.template.borrow();
        let grid_size = template_ref.size;
        let label = template_ref.label.clone();
        let module_controls = &module_ref.controls;
        let widgets = template_ref
            .widget_outlines
            .iter()
            .map(|wo| module_widgets::widget_from_outline(module_controls, wo))
            .collect();

        let size = (
            fatgrid(grid_size.0) + MIW * 2 + JACK_SIZE,
            fatgrid(grid_size.1),
        );
        let mut inputs = Vec::new();
        for (index, input) in template_ref.inputs.iter().enumerate() {
            inputs.push(InputJack::create(
                input.borrow_label().to_owned(),
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

    fn get_pos(&self) -> (i32, i32) {
        self.module.borrow().pos
    }

    pub fn respond_to_mouse_press(&self, mouse_pos: (i32, i32), mods: &MouseMods) -> MouseAction {
        let pos = self.get_pos();
        let mouse_pos = (mouse_pos.0 - pos.0, mouse_pos.1 - pos.1);
        if !mouse_pos.inside(self.size) {
            return MouseAction::None;
        }
        for (widget, _) in &self.widgets {
            let action = widget.respond_to_mouse_press(mouse_pos, mods, pos);
            if !action.is_none() {
                return action;
            }
        }
        for (index, input) in self.inputs.iter().enumerate() {
            if input.mouse_in_bounds(mouse_pos) {
                return MouseAction::ConnectInput(Rc::clone(&self.module), index);
            }
        }
        for (index, output) in self.outputs.iter().enumerate() {
            if output.mouse_in_bounds(mouse_pos) {
                return MouseAction::ConnectOutput(Rc::clone(&self.module), index);
            }
        }
        MouseAction::MoveModule(Rc::clone(&self.module))
    }

    pub fn get_drop_target_at(&self, mouse_pos: (i32, i32)) -> DropTarget {
        let pos = self.get_pos();
        let mouse_pos = (mouse_pos.0 - pos.0, mouse_pos.1 - pos.1);
        if !mouse_pos.inside(self.size) {
            return DropTarget::None;
        }
        for (widget, _) in &self.widgets {
            let target = widget.get_drop_target_at(mouse_pos);
            if !target.is_none() {
                return target;
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

    fn draw(
        &self,
        g: &mut GrahpicsWrapper,
        mouse_pos: (i32, i32),
        highlight: Option<(bool, ep::JackType)>,
    ) {
        let pos = self.get_pos();
        g.push_state();
        g.apply_offset(pos.0, pos.1);
        let mouse_pos = mouse_pos.sub(pos);

        const CS: i32 = CORNER_SIZE;
        const JS: i32 = JACK_SIZE;
        const MIW: i32 = MODULE_IO_WIDTH;

        g.set_color(&COLOR_IO_AREA);
        g.fill_rounded_rect(JS, 0, self.size.0 - JS, self.size.1, CS);
        g.set_color(&COLOR_SURFACE);
        g.fill_rect(JS + MIW, 0, self.size.0 - MIW * 2 - JS, self.size.1);

        g.set_color(&COLOR_TEXT);
        for (index, jack) in self.module.borrow().inputs.iter().enumerate() {
            let index = index as i32;
            let y = coord(index) + grid(1) / 2;
            if let ep::InputConnection::Wire(module, output_index) = jack {
                let output_index = *output_index as i32;
                let module_ref = module.borrow();
                let (ox, oy) = Self::output_position(&*module_ref, output_index);
                let (ox, oy) = (ox - pos.0, oy - pos.1);
                g.stroke_line(JS, y, ox, oy, 5.0);
            }
        }

        g.set_color(&COLOR_TEXT);
        g.write_text(
            12,
            MODULE_IO_WIDTH,
            -20,
            self.size.0,
            20,
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
            if let ep::InputConnection::Default(default_index) = module_ref.inputs[input_index] {
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
        for (widget, segment_len) in &self.widgets{
            widget.draw(g, highlight, pos, &feedback_data[fdi..fdi + segment_len]);
            fdi += segment_len;
        }

        g.pop_state();
    }
}

pub struct ModuleGraph {
    pub pos: (i32, i32),
    offset: Rcrc<(i32, i32)>,
    graph: Rcrc<ep::ModuleGraph>,
    modules: Vec<Module>,
    // Box because eventually this is going to be dyn.
    detail_menu_widget: Option<Box<KnobEditor>>,
}

impl ModuleGraph {
    pub fn create(graph: Rcrc<ep::ModuleGraph>) -> Self {
        let modules = graph
            .borrow()
            .borrow_modules()
            .iter()
            .map(|module_rc| Module::create(Rc::clone(module_rc)))
            .collect();
        Self {
            pos: (0, 0),
            offset: rcrc((0, 0)),
            graph,
            modules,
            detail_menu_widget: None,
        }
    }

    pub fn add_module(&mut self, mut module: ep::Module) {
        module.pos = *self.offset.borrow();
        module.pos = (-module.pos.0, -module.pos.1);
        let module = rcrc(module);
        self.graph.borrow_mut().add_module(Rc::clone(&module));
        self.modules.push(Module::create(module));
    }

    pub fn respond_to_mouse_press(
        &mut self,
        mouse_pos: (i32, i32),
        mods: &MouseMods,
    ) -> MouseAction {
        let offset = self.offset.borrow();
        let mouse_pos = (
            mouse_pos.0 - offset.0 - self.pos.0,
            mouse_pos.1 - offset.1 - self.pos.1,
        );
        if let Some(widget) = &self.detail_menu_widget {
            if let Some(action) = widget.respond_to_mouse_press(mouse_pos, mods) {
                return action;
            } else {
                self.detail_menu_widget = None;
            }
        }
        for module in self.modules.iter().rev() {
            let action = module.respond_to_mouse_press(mouse_pos, mods);
            if !action.is_none() {
                return action;
            }
        }
        MouseAction::PanOffset(Rc::clone(&self.offset))
    }

    pub fn get_drop_target_at(&self, mouse_pos: (i32, i32)) -> DropTarget {
        let offset = self.offset.borrow();
        let mouse_pos = (
            mouse_pos.0 - offset.0 - self.pos.0,
            mouse_pos.1 - offset.1 - self.pos.1,
        );
        for module in &self.modules {
            let target = module.get_drop_target_at(mouse_pos);
            if !target.is_none() {
                return target;
            }
        }
        DropTarget::None
    }

    pub fn draw(&self, g: &mut GrahpicsWrapper, gui_state: &Gui) {
        let offset = self.offset.borrow();
        g.push_state();
        g.apply_offset(offset.0 + self.pos.0, offset.1 + self.pos.1);
        let (mx, my) = gui_state.get_current_mouse_pos();
        let (mx, my) = (mx - offset.0 - self.pos.0, my - offset.1 - self.pos.1);
        let highlight = if gui_state.is_dragging() {
            let cma = gui_state.borrow_current_mouse_action();
            if let MouseAction::ConnectInput(module, index) = cma {
                let typ = module.borrow().template.borrow().inputs[*index].get_type();
                // Highlight outputs with typ.
                Some((true, typ))
            } else if let MouseAction::ConnectOutput(module, index) = cma {
                let typ = module.borrow().template.borrow().outputs[*index].get_type();
                // Highlight inputs with typ.
                Some((false, typ))
            } else {
                None
            }
        } else {
            None
        };
        for module in &self.modules {
            module.draw(g, (mx, my), highlight);
        }
        if let Some(widget) = &self.detail_menu_widget {
            widget.draw(g);
        }
        if gui_state.is_dragging() {
            let cma = gui_state.borrow_current_mouse_action();
            if let MouseAction::ConnectInput(module, index) = cma {
                let module_ref = module.borrow();
                let (sx, sy) = Module::input_position(&*module_ref, *index as i32);
                g.set_color(&COLOR_DEBUG);
                g.stroke_line(sx, sy, mx, my, 2.0);
            } else if let MouseAction::ConnectOutput(module, index) = cma {
                let module_ref = module.borrow();
                let (sx, sy) = Module::output_position(&*module_ref, *index as i32);
                g.set_color(&COLOR_DEBUG);
                g.stroke_line(sx, sy, mx, my, 2.0);
            }
        }
        g.pop_state();
    }

    pub fn open_menu(&mut self, menu: Box<KnobEditor>) {
        self.detail_menu_widget = Some(menu);
    }
}
