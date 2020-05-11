use crate::engine;
use crate::gui::constants::*;
use crate::gui::graphics::GrahpicsWrapper;
use crate::gui::Widget;
use crate::util::*;
use std::f32::consts::PI;

#[derive(Clone)]
pub struct Knob {
    module: Rcrc<engine::Module>,
    control: Rcrc<engine::Control>,
    pos: (i32, i32),
    label: String,
}

impl Knob {
    pub fn create(
        module: Rcrc<engine::Module>,
        control: Rcrc<engine::Control>,
        pos: (i32, i32),
        label: String,
    ) -> Knob {
        Knob {
            module,
            control,
            pos,
            label,
        }
    }
}

impl Widget for Knob {
    fn get_pos(&self) -> (i32, i32) {
        self.pos
    }

    fn get_size(&self) -> (i32, i32) {
        // Don't include the label in the boundaries.
        (GRID_2, GRID_1)
    }

    fn on_drag(&mut self, delta: (i32, i32)) {
        let delta = delta.0 - delta.1;
        // How many pixels the user must drag across to cover the entire range of the knob.
        const DRAG_PIXELS: f32 = 200.0;
        let delta = delta as f32 / DRAG_PIXELS;

        let mut control_ref = self.control.borrow_mut();
        let range = control_ref.range;
        let delta = delta * (range.1 - range.0) as f32;
        control_ref.value = (control_ref.value + delta).clam(range.0, range.1);
    }

    fn draw_impl(&self, g: &mut GrahpicsWrapper) {
        let control = &*self.control.borrow();
        fn value_to_angle(range: (f32, f32), value: f32) -> f32 {
            value.from_range_to_range(range.0, range.1, PI, 0.0)
        }

        g.set_color(&COLOR_TEXT);
        let self_module_pos = self.module.borrow().pos;
        for lane in self.control.borrow().automation.iter() {
            let (module, output_index) = &lane.connection;
            let output_index = *output_index as i32;
            let module_ref = module.borrow();
            let ox = module_ref.pos.0
                + fatgrid(module_ref.gui_outline.borrow().size.0)
                + MODULE_IO_WIDTH * 2
                - self.pos.0
                - self_module_pos.0;
            let oy = module_ref.pos.1 + coord(output_index) + GRID_1 / 2
                - self.pos.1
                - self_module_pos.1;
            g.stroke_line(GRID_2 / 2, GRID_2 / 2, ox, oy, 2.0);
        }

        g.set_color(&COLOR_BG);
        g.fill_pie(0, 0, GRID_2, KNOB_INSIDE_SPACE * 2, 0.0, PI);
        g.set_color(&COLOR_KNOB);
        let zero_angle = value_to_angle(control.range, 0.0);
        let value_angle = value_to_angle(control.range, control.value);
        g.fill_pie(0, 0, GRID_2, KNOB_INSIDE_SPACE * 2, zero_angle, value_angle);
        g.set_color(&COLOR_TEXT);
        g.write_label(0, GRID_1 + GRID_P, GRID_2, &self.label);

        if control.automation.len() == 0 {
            return;
        }

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
}

impl Widget for IOTab {
    fn get_pos(&self) -> (i32, i32) {
        self.pos
    }

    fn get_size(&self) -> (i32, i32) {
        const MITS: i32 = MODULE_IO_TAB_SIZE;
        (MITS, MITS)
    }

    fn draw_impl(&self, g: &mut GrahpicsWrapper) {
        const MITS: i32 = MODULE_IO_TAB_SIZE;
        const MCS: i32 = MODULE_CORNER_SIZE;
        g.fill_rounded_rect(0, 0, MITS, MITS, MCS);
        let x = if self.is_output { MITS - MCS } else { 0 };
        g.fill_rect(x, 0, MCS, MITS);
    }
}

pub struct Module {
    module: Rcrc<engine::Module>,
    size: (i32, i32),
    label: String,
    children: Vec<Rcrc<dyn Widget>>,
}

impl Module {
    pub fn create(
        module: Rcrc<engine::Module>,
        grid_size: (i32, i32),
        label: String,
        mut children: Vec<Rcrc<dyn Widget>>,
    ) -> Self {
        const MIW: i32 = MODULE_IO_WIDTH;
        let size = (fatgrid(grid_size.0) + MIW * 2, fatgrid(grid_size.1));
        let module_ref = module.borrow();
        for index in 0..module_ref.inputs.len() as i32 {
            children.push(rcrc(IOTab::input(0, coord(index))));
        }
        let x = size.0 - MODULE_IO_TAB_SIZE;
        for index in 0..module_ref.outputs.len() as i32 {
            children.push(rcrc(IOTab::output(x, coord(index))));
        }
        drop(module_ref);
        Self {
            module,
            size,
            label,
            children,
        }
    }
}

impl Widget for Module {
    fn get_pos(&self) -> (i32, i32) {
        self.module.borrow().pos
    }

    fn get_size(&self) -> (i32, i32) {
        self.size
    }

    fn borrow_children(&self) -> &[Rcrc<dyn Widget>] {
        &self.children[..]
    }

    fn on_drag(&mut self, delta: (i32, i32)) {
        let mut module_ref = self.module.borrow_mut();
        module_ref.pos.0 += delta.0;
        module_ref.pos.1 += delta.1;
    }

    fn draw_impl(&self, g: &mut GrahpicsWrapper) {
        const MCS: i32 = MODULE_CORNER_SIZE;
        const MIW: i32 = MODULE_IO_WIDTH;

        g.set_color(&COLOR_IO_AREA);
        g.fill_rounded_rect(0, 0, self.size.0, self.size.1, MCS);
        g.set_color(&COLOR_SURFACE);
        g.fill_rect(MIW, 0, self.size.0 - MIW * 2, self.size.1);

        let pos = self.get_pos();
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
    }
}

pub struct ModuleGraph {
    pub pos: (i32, i32),
    pub offset: (i32, i32),
    pub size: (i32, i32),
    children: Vec<Rcrc<dyn Widget>>,
}

impl ModuleGraph {
    pub fn create(children: Vec<Rcrc<dyn Widget>>) -> Self {
        Self {
            pos: (0, 0),
            offset: (0, 0),
            size: (0, 0),
            children,
        }
    }
}

impl Default for ModuleGraph {
    fn default() -> ModuleGraph {
        ModuleGraph {
            pos: (0, 0),
            offset: (0, 0),
            size: (9999, 9999),
            children: Vec::new(),
        }
    }
}

impl Widget for ModuleGraph {
    fn get_pos(&self) -> (i32, i32) {
        self.pos
    }

    fn get_size(&self) -> (i32, i32) {
        self.size
    }

    fn borrow_children(&self) -> &[Rcrc<dyn Widget>] {
        &self.children[..]
    }

    fn draw_impl(&self, g: &mut GrahpicsWrapper) {
        g.apply_offset(self.offset.0, self.offset.1);
    }
}
