use crate::engine::parts as ep;
use crate::gui::constants::*;
use crate::gui::graphics::{GrahpicsWrapper, HAlign, VAlign};
use crate::gui::{DropTarget, Gui, MouseAction, MouseMods};
use crate::util::*;
use std::f32::consts::PI;

fn jack_y(index: i32) -> i32 {
    coord(index) + MODULE_IO_JACK_SIZE / 2
}

fn input_position(module: &ep::Module, input_index: i32) -> (i32, i32) {
    let module_pos = module.pos;
    (module_pos.0, module_pos.1 + jack_y(input_index))
}

fn output_position(module: &ep::Module, output_index: i32) -> (i32, i32) {
    let module_pos = module.pos;
    let module_size = module.template.borrow().size;
    let module_width = fatgrid(module_size.0) + MODULE_IO_WIDTH * 2;
    (
        module_pos.0 + module_width,
        module_pos.1 + jack_y(output_index),
    )
}

#[derive(Clone)]
pub struct KnobEditor {
    control: Rcrc<ep::Control>,
    pos: (i32, i32),
    size: (i32, i32),
    label: String,
}

impl KnobEditor {
    pub fn create(control: Rcrc<ep::Control>, center_pos: (i32, i32), label: String) -> Self {
        let num_channels = control.borrow().automation.len().max(2) as i32;
        let required_radius =
            (KNOB_MENU_LANE_SIZE + KNOB_MENU_LANE_GAP) * num_channels + KNOB_MENU_KNOB_OR + GRID_P;
        let size = (required_radius * 2, required_radius + fatgrid(1));
        Self {
            control,
            pos: (center_pos.0 - size.0 / 2, center_pos.1 - size.1 / 2),
            size,
            label,
        }
    }

    pub fn respond_to_mouse_press(
        &self,
        mouse_pos: (i32, i32),
        mods: &MouseMods,
    ) -> Option<MouseAction> {
        let mouse_pos = (mouse_pos.0 - self.pos.0, mouse_pos.1 - self.pos.1);
        if mouse_pos.inside(self.size) {
            // Yes, the last 0 is intentional. The center of the knob is not vertically centered.
            let (cx, cy) = (mouse_pos.0 - self.size.0 / 2, mouse_pos.1 - self.size.0 / 2);
            // y coordinate is inverted from how it appears on screen.
            let (fcx, fcy) = (cx as f32, -cy as f32);
            let (angle, radius) = (fcy.atan2(fcx), (fcy * fcy + fcx * fcx).sqrt());
            let control = &*self.control.borrow();
            let auto_lanes = control.automation.len();
            // Clicked somewhere in the top "half" where the main knob and automation lanes are.
            if angle >= 0.0 && angle <= PI {
                let radius = radius as i32;
                if radius < KNOB_MENU_KNOB_IR {
                    // Nothing interacjackle inside the knob.
                } else if radius < KNOB_MENU_KNOB_OR {
                    return Some(MouseAction::ManipulateControl(Rc::clone(&self.control)));
                } else {
                    let radius = radius - KNOB_MENU_KNOB_OR;
                    let lane = (radius / (KNOB_MENU_LANE_SIZE + KNOB_MENU_LANE_GAP)) as usize;
                    if lane < auto_lanes {
                        // It's rendered backwards so we need to flip the index to make it visually
                        // match up.
                        let lane = auto_lanes - lane - 1;
                        let range = control.range;
                        let lane_range = control.automation[lane].range;
                        let min_angle = lane_range.0.from_range_to_range(range.0, range.1, PI, 0.0);
                        let max_angle = lane_range.1.from_range_to_range(range.0, range.1, PI, 0.0);
                        // TODO: Handle inverted lanes.
                        return Some(if angle > min_angle {
                            MouseAction::ManipulateLaneStart(Rc::clone(&self.control), lane)
                        } else if angle < max_angle {
                            MouseAction::ManipulateLaneEnd(Rc::clone(&self.control), lane)
                        } else {
                            MouseAction::ManipulateLane(Rc::clone(&self.control), lane)
                        });
                    }
                }
            }
            Some(MouseAction::None)
        } else {
            None
        }
    }

    fn draw(&self, g: &mut GrahpicsWrapper) {
        g.push_state();

        g.apply_offset(self.pos.0, self.pos.1);
        g.set_color(&COLOR_SURFACE);
        g.fill_rounded_rect(0, 0, self.size.0, self.size.1, MODULE_CORNER_SIZE);

        let control = &*self.control.borrow();
        fn value_to_angle(range: (f32, f32), value: f32) -> f32 {
            value.from_range_to_range(range.0, range.1, PI, 0.0)
        }
        g.apply_offset(self.size.0 / 2, self.size.1 - fatgrid(1));

        const KOR: i32 = KNOB_MENU_KNOB_OR;
        const KIR: i32 = KNOB_MENU_KNOB_IR;
        g.set_color(&COLOR_BG);
        g.fill_pie(-KOR, -KOR, KOR * 2, KIR * 2, PI, 0.0);
        g.set_color(&COLOR_KNOB);
        let zero_angle = value_to_angle(control.range, 0.0);
        let value_angle = value_to_angle(control.range, control.value);
        g.fill_pie(-KOR, -KOR, KOR * 2, KIR * 2, zero_angle, value_angle);

        const GAP: i32 = KNOB_MENU_LANE_GAP;
        const LS: i32 = KNOB_MENU_LANE_SIZE;
        // TODO: Handle inverted lanes.
        for (index, lane) in control.automation.iter().rev().enumerate() {
            let ir = KOR + GAP + (GAP + LS) * index as i32;
            let or = ir + LS;
            g.set_color(&COLOR_BG);
            g.fill_pie(-or, -or, or * 2, ir * 2, PI, 0.0);
            g.set_color(&COLOR_AUTOMATION);
            let min_angle = value_to_angle(control.range, lane.range.0);
            let max_angle = value_to_angle(control.range, lane.range.1);
            g.fill_pie(-or, -or, or * 2, ir * 2, min_angle, max_angle);
        }

        g.set_color(&COLOR_TEXT);
        let value_text = format_decimal(control.value, 3);
        g.write_label(-KIR, -12, KIR * 2, &value_text);
        g.write_label(-KOR, GRID_P, KOR * 2, &self.label);

        g.pop_state();
    }
}

#[derive(Clone)]
pub struct Knob {
    control: Rcrc<ep::Control>,
    pos: (i32, i32),
    label: String,
}

impl Knob {
    pub fn create(control: Rcrc<ep::Control>, pos: (i32, i32), label: String) -> Knob {
        Knob {
            control,
            pos,
            label,
        }
    }

    pub fn respond_to_mouse_press(
        &self,
        mouse_pos: (i32, i32),
        mods: &MouseMods,
        parent_pos: (i32, i32),
    ) -> MouseAction {
        let mouse_pos = (mouse_pos.0 - self.pos.0, mouse_pos.1 - self.pos.1);
        if mouse_pos.inside((grid(2), grid(1))) {
            if mods.right_click {
                let pos = (
                    self.pos.0 + parent_pos.0 + grid(2) / 2,
                    self.pos.1 + parent_pos.1 + grid(2) / 2,
                );
                MouseAction::OpenMenu(Box::new(KnobEditor::create(
                    Rc::clone(&self.control),
                    pos,
                    self.label.clone(),
                )))
            } else {
                MouseAction::ManipulateControl(Rc::clone(&self.control))
            }
        } else {
            MouseAction::None
        }
    }

    pub fn get_drop_target_at(&self, mouse_pos: (i32, i32)) -> DropTarget {
        let mouse_pos = (mouse_pos.0 - self.pos.0, mouse_pos.1 - self.pos.1);
        if mouse_pos.inside((grid(2), grid(1))) {
            DropTarget::Control(Rc::clone(&self.control))
        } else {
            DropTarget::None
        }
    }

    fn draw(&self, g: &mut GrahpicsWrapper, parent_pos: (i32, i32)) {
        g.push_state();

        let control = &*self.control.borrow();
        fn value_to_angle(range: (f32, f32), value: f32) -> f32 {
            value.from_range_to_range(range.0, range.1, PI, 0.0)
        }

        g.set_color(&COLOR_TEXT);
        let (cx, cy) = (self.pos.0 + grid(2) / 2, self.pos.1 + grid(2) / 2);
        for lane in self.control.borrow().automation.iter() {
            let (module, output_index) = &lane.connection;
            let output_index = *output_index as i32;
            let module_ref = module.borrow();
            let (ox, oy) = output_position(&*module_ref, output_index);
            let (ox, oy) = (ox - parent_pos.0, oy - parent_pos.1);
            g.stroke_line(cx, cy, ox, oy, 2.0);
        }

        // Applying the offset later makes connections easier to render.
        g.apply_offset(self.pos.0, self.pos.1);

        g.set_color(&COLOR_BG);
        g.fill_pie(0, 0, grid(2), KNOB_INSIDE_SPACE * 2, 0.0, PI);
        g.set_color(&COLOR_KNOB);
        let zero_angle = value_to_angle(control.range, 0.0);
        let value_angle = value_to_angle(control.range, control.value);
        g.fill_pie(
            0,
            0,
            grid(2),
            KNOB_INSIDE_SPACE * 2,
            zero_angle,
            value_angle,
        );
        g.set_color(&COLOR_TEXT);
        g.write_label(0, grid(1) + GRID_P, grid(2), &self.label);

        if control.automation.len() > 0 {
            let num_lanes = control.automation.len() as i32;
            let lane_size = KNOB_AUTOMATION_SPACE / num_lanes;
            let lane_size = lane_size.min(KNOB_MAX_LANE_SIZE).max(2);
            for (index, lane) in control.automation.iter().enumerate() {
                if index == 1 {
                    g.set_color(&COLOR_AUTOMATION_FOCUSED);
                } else {
                    g.set_color(&COLOR_AUTOMATION);
                }
                let index = index as i32;
                let outer_diameter = grid(2) - (KNOB_OUTSIDE_SPACE * 2) - lane_size * index * 2;
                let inner_diameter = outer_diameter - (lane_size - KNOB_LANE_GAP) * 2;
                let inset = (grid(2) - outer_diameter) / 2;
                let min_angle = value_to_angle(control.range, lane.range.0);
                let max_angle = value_to_angle(control.range, lane.range.1);
                g.fill_pie(
                    inset,
                    inset,
                    outer_diameter,
                    inner_diameter,
                    min_angle,
                    max_angle,
                );
            }
        }

        g.pop_state();
    }
}

struct IOJack {
    label: String,
    icon_index: usize,
    pos: (i32, i32),
    is_output: bool,
}

impl IOJack {
    fn input(label: String, icon_index: usize, x: i32, y: i32) -> Self {
        Self {
            label,
            icon_index,
            pos: (x, y),
            is_output: false,
        }
    }

    fn output(label: String, icon_index: usize, x: i32, y: i32) -> Self {
        Self {
            label,
            icon_index,
            pos: (x, y),
            is_output: true,
        }
    }

    pub fn mouse_in_bounds(&self, mouse_pos: (i32, i32)) -> bool {
        let mouse_pos = (mouse_pos.0 - self.pos.0, mouse_pos.1 - self.pos.1);
        mouse_pos.inside((MODULE_IO_JACK_SIZE, MODULE_IO_JACK_SIZE))
    }

    fn draw(&self, g: &mut GrahpicsWrapper, show_label: bool) {
        g.push_state();
        g.apply_offset(self.pos.0, self.pos.1);

        const MITS: i32 = MODULE_IO_JACK_SIZE;
        const MCS: i32 = MODULE_CORNER_SIZE;
        g.fill_rounded_rect(0, 0, MITS, MITS, MCS);
        let x = if self.is_output { MITS - MCS } else { 0 };
        g.fill_rect(x, 0, MCS, MITS);
        const MITIP: i32 = MODULE_IO_JACK_ICON_PADDING;
        g.draw_icon(self.icon_index, MITIP, MITIP, MITS - MITIP * 2);

        if show_label {
            g.write_text(
                12,
                if self.is_output { MITS + 2 } else { -102 },
                0,
                100,
                MITS,
                if self.is_output {
                    HAlign::Left
                } else {
                    HAlign::Right
                },
                VAlign::Center,
                1,
                &self.label,
            )
        }

        g.pop_state();
    }
}

fn widget_from_outline(controls: &Vec<Rcrc<ep::Control>>, outline: &ep::WidgetOutline) -> Knob {
    fn convert_grid_pos(grid_pos: (i32, i32)) -> (i32, i32) {
        (MODULE_IO_WIDTH + coord(grid_pos.0), coord(grid_pos.1))
    }
    match outline {
        ep::WidgetOutline::Knob {
            control_index,
            grid_pos,
            label,
        } => {
            let pos = convert_grid_pos(*grid_pos);
            Knob::create(Rc::clone(&controls[*control_index]), pos, label.clone())
        }
    }
}

pub struct Module {
    module: Rcrc<ep::Module>,
    size: (i32, i32),
    label: String,
    inputs: Vec<IOJack>,
    outputs: Vec<IOJack>,
    controls: Vec<Knob>,
}

impl Module {
    pub fn create(module: Rcrc<ep::Module>) -> Self {
        const MIW: i32 = MODULE_IO_WIDTH;
        let module_ref = module.borrow();
        let template_ref = module_ref.template.borrow();
        let grid_size = template_ref.size;
        let label = template_ref.label.clone();
        let module_controls = &module_ref.controls;
        let controls = template_ref
            .widget_outlines
            .iter()
            .map(|wo| widget_from_outline(module_controls, wo))
            .collect();

        let size = (fatgrid(grid_size.0) + MIW * 2, fatgrid(grid_size.1));
        let mut inputs = Vec::new();
        for (index, input) in template_ref.inputs.iter().enumerate() {
            inputs.push(IOJack::input(
                input.borrow_label().to_owned(),
                input.get_icon_index(),
                0,
                coord(index as i32),
            ));
        }
        let x = size.0 - MODULE_IO_JACK_SIZE;
        let mut outputs = Vec::new();
        for (index, output) in template_ref.outputs.iter().enumerate() {
            outputs.push(IOJack::output(
                output.borrow_label().to_owned(),
                output.get_icon_index(),
                x,
                coord(index as i32),
            ));
        }
        drop(template_ref);
        drop(module_ref);

        Self {
            module,
            size,
            label,
            inputs,
            outputs,
            controls,
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
        for control in &self.controls {
            let action = control.respond_to_mouse_press(mouse_pos, mods, pos);
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
        for control in &self.controls {
            let target = control.get_drop_target_at(mouse_pos);
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

    fn draw(&self, g: &mut GrahpicsWrapper, mouse_pos: (i32, i32)) {
        let pos = self.get_pos();
        g.push_state();
        g.apply_offset(pos.0, pos.1);
        let mouse_pos = mouse_pos.sub(pos);

        const MCS: i32 = MODULE_CORNER_SIZE;
        const MIW: i32 = MODULE_IO_WIDTH;

        g.set_color(&COLOR_IO_AREA);
        g.fill_rounded_rect(0, 0, self.size.0, self.size.1, MCS);
        g.set_color(&COLOR_SURFACE);
        g.fill_rect(MIW, 0, self.size.0 - MIW * 2, self.size.1);

        g.set_color(&COLOR_TEXT);
        for (index, jack) in self.module.borrow().inputs.iter().enumerate() {
            let index = index as i32;
            let y = coord(index) + grid(1) / 2;
            if let ep::InputConnection::Wire(module, output_index) = jack {
                let output_index = *output_index as i32;
                let module_ref = module.borrow();
                let (ox, oy) = output_position(&*module_ref, output_index);
                let (ox, oy) = (ox - pos.0, oy - pos.1);
                g.stroke_line(0, y, ox, oy, 5.0);
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

        let hovering = mouse_pos.inside(self.size);
        for input in &self.inputs {
            input.draw(g, hovering);
        }
        for output in &self.outputs {
            output.draw(g, hovering);
        }
        for control in &self.controls {
            control.draw(g, pos);
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
        for module in &self.modules {
            module.draw(g, (mx, my));
        }
        if let Some(widget) = &self.detail_menu_widget {
            widget.draw(g);
        }
        if gui_state.is_dragging() {
            let cma = gui_state.borrow_current_mouse_action();
            if let MouseAction::ConnectInput(module, index) = cma {
                let module_ref = module.borrow();
                let (sx, sy) = input_position(&*module_ref, *index as i32);
                g.set_color(&COLOR_DEBUG);
                g.stroke_line(sx, sy, mx, my, 2.0);
            } else if let MouseAction::ConnectOutput(module, index) = cma {
                let module_ref = module.borrow();
                let (sx, sy) = output_position(&*module_ref, *index as i32);
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
