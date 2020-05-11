use crate::engine;
use crate::gui::MouseAction;
use crate::gui::constants::*;
use crate::gui::graphics::GrahpicsWrapper;
use crate::util::*;
use std::f32::consts::PI;

fn bound_check(coord: (i32, i32), bounds: (i32, i32)) -> bool {
    coord.0 >= 0 && coord.1 >= 0 && coord.0 <= bounds.0 && coord.1 <= bounds.1
}

#[derive(Clone)]
pub struct Knob {
    control: Rcrc<engine::Control>,
    pos: (i32, i32),
    label: String,
}

impl Knob {
    pub fn create(
        control: Rcrc<engine::Control>,
        pos: (i32, i32),
        label: String,
    ) -> Knob {
        Knob {
            control,
            pos,
            label,
        }
    }

    pub fn respond_to_mouse_press(&self, mouse_pos: (i32, i32)) -> MouseAction {
        let mouse_pos = (mouse_pos.0 - self.pos.0, mouse_pos.1 - self.pos.1);
        if bound_check(mouse_pos, (GRID_2, GRID_1)) {
            MouseAction::ManipulateControl(Rc::clone(&self.control))
        } else {
            MouseAction::None
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
            let ox = module_ref.pos.0
                + fatgrid(module_ref.gui_outline.borrow().size.0)
                + MODULE_IO_WIDTH * 2
                - parent_pos.0;
            let oy = module_ref.pos.1 + coord(output_index) + GRID_1 / 2
                - parent_pos.1;
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
            let lane_size = lane_size.min(KNOB_MAX_LANE_SIZE);
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

    pub fn respond_to_mouse_press(&self, mouse_pos: (i32, i32)) -> MouseAction {
        let mouse_pos = (mouse_pos.0 - self.pos.0, mouse_pos.1 - self.pos.1);
        if bound_check(mouse_pos, (MODULE_IO_TAB_SIZE, MODULE_IO_TAB_SIZE)) {
            // TODO: wire actions.
            MouseAction::None
        } else {
            MouseAction::None
        }
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
    module: Rcrc<engine::Module>,
    size: (i32, i32),
    label: String,
    tabs: Vec<IOTab>,
    controls: Vec<Knob>,
}

impl Module {
    pub fn create(
        module: Rcrc<engine::Module>,
        grid_size: (i32, i32),
        label: String,
        controls: Vec<Knob>,
    ) -> Self {
        const MIW: i32 = MODULE_IO_WIDTH;
        let size = (fatgrid(grid_size.0) + MIW * 2, fatgrid(grid_size.1));
        let module_ref = module.borrow();
        let mut tabs = Vec::new();
        for index in 0..module_ref.inputs.len() as i32 {
            tabs.push(IOTab::input(0, coord(index)));
        }
        let x = size.0 - MODULE_IO_TAB_SIZE;
        for index in 0..module_ref.outputs.len() as i32 {
            tabs.push(IOTab::output(x, coord(index)));
        }
        drop(module_ref);
        Self {
            module,
            size,
            label,
            tabs,
            controls,
        }
    }

    fn get_pos(&self) -> (i32, i32) {
        self.module.borrow().pos
    }

    pub fn respond_to_mouse_press(&self, mouse_pos: (i32, i32)) -> MouseAction {
        let pos = self.get_pos();
        let mouse_pos = (mouse_pos.0 - pos.0, mouse_pos.1 - pos.1);
        if !bound_check(mouse_pos, self.size) {
            return MouseAction::None;
        }
        for control in &self.controls {
            let action = control.respond_to_mouse_press(mouse_pos);
            if !action.is_none() {
                return action;
            }
        }
        for tab in &self.tabs {
            let action = tab.respond_to_mouse_press(mouse_pos);
            if !action.is_none() {
                return action;
            }
        }
        MouseAction::MoveModule(Rc::clone(&self.module))
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
            if let Some((module, output_index)) = &tab.connection {
                let output_index = *output_index as i32;
                let module_ref = module.borrow();
                let ox = module_ref.pos.0
                    + fatgrid(module_ref.gui_outline.borrow().size.0)
                    + MODULE_IO_WIDTH * 2
                    - pos.0;
                let oy = module_ref.pos.1 + coord(output_index) + GRID_1 / 2 - pos.1;
                g.stroke_line(0, y, ox, oy, 5.0);
            }
        }

        for tab in &self.tabs {
            tab.draw(g);
        }
        for control in &self.controls {
            control.draw(g, pos);
        }

        g.pop_state();
    }
}

pub struct ModuleGraph {
    pub pos: (i32, i32),
    pub offset: (i32, i32),
    pub size: (i32, i32),
    modules: Vec<Module>,
}

impl ModuleGraph {
    pub fn create(modules: Vec<Module>) -> Self {
        Self {
            pos: (0, 0),
            offset: (0, 0),
            size: (0, 0),
            modules,
        }
    }

    pub fn respond_to_mouse_press(&self, mouse_pos: (i32, i32)) -> MouseAction {
        let mouse_pos = (mouse_pos.0 - self.offset.0, mouse_pos.1 - self.offset.1);
        for module in &self.modules {
            let action = module.respond_to_mouse_press(mouse_pos);
            if !action.is_none() {
                return action;
            }
        }
        MouseAction::None
    }

    pub fn draw(&self, g: &mut GrahpicsWrapper) {
        g.push_state();
        g.apply_offset(self.pos.0, self.pos.1);
        g.apply_offset(self.offset.0, self.offset.1);
        for module in &self.modules {
            module.draw(g);
        }
        g.pop_state();
    }
}

impl Default for ModuleGraph {
    fn default() -> ModuleGraph {
        ModuleGraph {
            pos: (0, 0),
            offset: (0, 0),
            size: (9999, 9999),
            modules: Vec::new(),
        }
    }
}
