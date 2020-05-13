use crate::engine::parts as ep;
use crate::gui::constants::*;
use crate::gui::graphics::GrahpicsWrapper;
use crate::gui::{DropTarget, Gui, MouseAction, MouseMods};
use crate::util::*;
use std::f32::consts::PI;

fn bound_check(coord: (i32, i32), bounds: (i32, i32)) -> bool {
    coord.0 >= 0 && coord.1 >= 0 && coord.0 <= bounds.0 && coord.1 <= bounds.1
}

fn tab_y(index: i32) -> i32 {
    coord(index) + MODULE_IO_TAB_SIZE / 2
}

fn input_position(module: &ep::Module, input_index: i32) -> (i32, i32) {
    let module_pos = module.pos;
    (module_pos.0, module_pos.1 + tab_y(input_index))
}

fn output_position(module: &ep::Module, output_index: i32) -> (i32, i32) {
    let module_pos = module.pos;
    let module_size = module.gui_outline.borrow().size;
    let module_width = fatgrid(module_size.0) + MODULE_IO_WIDTH * 2;
    (
        module_pos.0 + module_width,
        module_pos.1 + tab_y(output_index),
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
        if bound_check(mouse_pos, self.size) {
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
                    // Nothing interactable inside the knob.
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

    pub fn get_drop_target_at(&self, mouse_pos: (i32, i32)) -> DropTarget {
        let mouse_pos = (mouse_pos.0 - self.pos.0, mouse_pos.1 - self.pos.1);
        if bound_check(mouse_pos, self.size) {
            DropTarget::None
        } else {
            DropTarget::None
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
        if bound_check(mouse_pos, (GRID_2, GRID_1)) {
            if mods.right_click {
                let pos = (
                    self.pos.0 + parent_pos.0 + GRID_2 / 2,
                    self.pos.1 + parent_pos.1 + GRID_2 / 2,
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
        if bound_check(mouse_pos, (GRID_2, GRID_1)) {
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
        let (cx, cy) = (self.pos.0 + GRID_2 / 2, self.pos.1 + GRID_2 / 2);
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
        g.fill_pie(0, 0, GRID_2, KNOB_INSIDE_SPACE * 2, 0.0, PI);
        g.set_color(&COLOR_KNOB);
        let zero_angle = value_to_angle(control.range, 0.0);
        let value_angle = value_to_angle(control.range, control.value);
        g.fill_pie(0, 0, GRID_2, KNOB_INSIDE_SPACE * 2, zero_angle, value_angle);
        g.set_color(&COLOR_TEXT);
        g.write_label(0, GRID_1 + GRID_P, GRID_2, &self.label);

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
                let outer_diameter = GRID_2 - (KNOB_OUTSIDE_SPACE * 2) - lane_size * index * 2;
                let inner_diameter = outer_diameter - (lane_size - KNOB_LANE_GAP) * 2;
                let inset = (GRID_2 - outer_diameter) / 2;
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

struct IOTab {
    pos: (i32, i32),
    is_output: bool,
}

impl IOTab {
    fn input(x: i32, y: i32) -> Self {
        Self {
            pos: (x, y),
            is_output: false,
        }
    }

    fn output(x: i32, y: i32) -> Self {
        Self {
            pos: (x, y),
            is_output: true,
        }
    }

    pub fn mouse_in_bounds(&self, mouse_pos: (i32, i32)) -> bool {
        let mouse_pos = (mouse_pos.0 - self.pos.0, mouse_pos.1 - self.pos.1);
        bound_check(mouse_pos, (MODULE_IO_TAB_SIZE, MODULE_IO_TAB_SIZE))
    }

    fn draw(&self, g: &mut GrahpicsWrapper) {
        g.push_state();
        g.apply_offset(self.pos.0, self.pos.1);

        const MITS: i32 = MODULE_IO_TAB_SIZE;
        const MCS: i32 = MODULE_CORNER_SIZE;
        g.fill_rounded_rect(0, 0, MITS, MITS, MCS);
        let x = if self.is_output { MITS - MCS } else { 0 };
        g.fill_rect(x, 0, MCS, MITS);

        g.pop_state();
    }
}

pub struct Module {
    module: Rcrc<ep::Module>,
    size: (i32, i32),
    label: String,
    inputs: Vec<IOTab>,
    outputs: Vec<IOTab>,
    controls: Vec<Knob>,
}

impl Module {
    pub fn create(
        module: Rcrc<ep::Module>,
        grid_size: (i32, i32),
        label: String,
        controls: Vec<Knob>,
    ) -> Self {
        const MIW: i32 = MODULE_IO_WIDTH;
        let size = (fatgrid(grid_size.0) + MIW * 2, fatgrid(grid_size.1));
        let module_ref = module.borrow();
        let mut inputs = Vec::new();
        for index in 0..module_ref.inputs.len() as i32 {
            inputs.push(IOTab::input(0, coord(index)));
        }
        let x = size.0 - MODULE_IO_TAB_SIZE;
        let mut outputs = Vec::new();
        for index in 0..module_ref.output_tabs.len() as i32 {
            outputs.push(IOTab::output(x, coord(index)));
        }
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
        if !bound_check(mouse_pos, self.size) {
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
        if !bound_check(mouse_pos, self.size) {
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

    fn draw(&self, g: &mut GrahpicsWrapper) {
        let pos = self.get_pos();
        g.push_state();
        g.apply_offset(pos.0, pos.1);

        const MCS: i32 = MODULE_CORNER_SIZE;
        const MIW: i32 = MODULE_IO_WIDTH;

        g.set_color(&COLOR_IO_AREA);
        g.fill_rounded_rect(0, 0, self.size.0, self.size.1, MCS);
        g.set_color(&COLOR_SURFACE);
        g.fill_rect(MIW, 0, self.size.0 - MIW * 2, self.size.1);

        g.set_color(&COLOR_TEXT);
        for (index, tab) in self.module.borrow().inputs.iter().enumerate() {
            let index = index as i32;
            let y = coord(index) + GRID_1 / 2;
            if let ep::InputConnection::Wire(module, output_index) = tab {
                let output_index = *output_index as i32;
                let module_ref = module.borrow();
                let (ox, oy) = output_position(&*module_ref, output_index);
                let (ox, oy) = (ox - pos.0, oy - pos.1);
                g.stroke_line(0, y, ox, oy, 5.0);
            }
        }

        for input in &self.inputs {
            input.draw(g);
        }
        for output in &self.outputs {
            output.draw(g);
        }
        for control in &self.controls {
            control.draw(g, pos);
        }

        g.pop_state();
    }
}

pub struct ModuleGraph {
    offset: Rcrc<(i32, i32)>,
    modules: Vec<Module>,
    // Box because eventually this is going to be dyn.
    detail_menu_widget: Option<Box<KnobEditor>>,
}

impl ModuleGraph {
    pub fn create(modules: Vec<Module>) -> Self {
        Self {
            offset: rcrc((0, 0)),
            modules,
            detail_menu_widget: None,
        }
    }

    pub fn respond_to_mouse_press(
        &mut self,
        mouse_pos: (i32, i32),
        mods: &MouseMods,
    ) -> MouseAction {
        let offset = self.offset.borrow();
        let mouse_pos = (mouse_pos.0 - offset.0, mouse_pos.1 - offset.1);
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
        let mouse_pos = (mouse_pos.0 - offset.0, mouse_pos.1 - offset.1);
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
        g.apply_offset(offset.0, offset.1);
        for module in &self.modules {
            module.draw(g);
        }
        if let Some(widget) = &self.detail_menu_widget {
            widget.draw(g);
        }
        if gui_state.is_dragging() {
            let cma = gui_state.borrow_current_mouse_action();
            if let MouseAction::ConnectInput(module, index) = cma {
                let module_ref = module.borrow();
                let (sx, sy) = input_position(&*module_ref, *index as i32);
                let (mx, my) = gui_state.get_current_mouse_pos();
                let offset = self.offset.borrow();
                let (mx, my) = (mx - offset.0, my - offset.1);
                g.set_color(&COLOR_DEBUG);
                g.stroke_line(sx, sy, mx, my, 2.0);
            } else if let MouseAction::ConnectOutput(module, index) = cma {
                let module_ref = module.borrow();
                let (sx, sy) = output_position(&*module_ref, *index as i32);
                let (mx, my) = gui_state.get_current_mouse_pos();
                let offset = self.offset.borrow();
                let (mx, my) = (mx - offset.0, my - offset.1);
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
