use crate::engine;
use crate::gui::constants::*;
use crate::gui::graphics::GrahpicsWrapper;
use crate::util::*;
use std::f32::consts::PI;

// This trait is convenient to implement for widgets, but inconvenient to call.
pub trait WidgetImpl {
    fn get_pos(&self) -> (i32, i32);
    fn borrow_children(&self) -> &[Rcrc<dyn Widget>] {
        &[]
    }
    fn draw(&self, g: &mut GrahpicsWrapper);
}

// This trait is convenient to call, but inconvenient for widgets to implement.
pub trait Widget: WidgetImpl {
    fn draw(&self, g: &mut GrahpicsWrapper);
}

// All widgets with the easy-to-implement trait will also implement the easy-to-call trait.
impl<T: WidgetImpl> Widget for T {
    fn draw(&self, g: &mut GrahpicsWrapper) {
        g.push_state();
        let pos = WidgetImpl::get_pos(self);
        g.apply_offset(pos.0, pos.1);
        WidgetImpl::draw(self, g);
        for child in self.borrow_children() {
            Widget::draw(&*child.borrow(), g);
        }
        g.pop_state();
    }
}

#[derive(Clone)]
pub struct Knob {
    control: Rcrc<engine::Control>,
    pos: (i32, i32),
    label: String,
}

impl Knob {
    pub fn create(control: Rcrc<engine::Control>, pos: (i32, i32), label: String) -> Knob {
        Knob {
            control,
            pos,
            label,
        }
    }
}

impl WidgetImpl for Knob {
    fn get_pos(&self) -> (i32, i32) {
        self.pos
    }

    fn draw(&self, g: &mut GrahpicsWrapper) {
        let control = &*self.control.borrow();
        fn value_to_angle(range: (f32, f32), value: f32) -> f32 {
            value.from_range_to_range(range.0, range.1, PI, 0.0)
        }

        g.set_color(&COLOR_BG);
        g.fill_pie(0, 0, GRID_2, KNOB_INSIDE_SPACE, 0.0, PI);
        g.set_color(&COLOR_KNOB);
        let zero_angle = value_to_angle(control.range, 0.0);
        let value_angle = value_to_angle(control.range, control.value);
        g.fill_pie(0, 0, GRID_2, KNOB_INSIDE_SPACE, zero_angle, value_angle);
        g.set_color(&COLOR_TEXT);
        g.write_label(0, GRID_1 + GRID_P, GRID_2, &self.label);

        if control.automation.len() == 0 {
            return;
        }

        let num_lanes = control.automation.len() as i32;
        let lane_size = KNOB_AUTOMATION_SPACE / num_lanes;
        let lane_size = lane_size.min(KNOB_MAX_LANE_SIZE);
        for (index, (min, max)) in control.automation.iter().enumerate() {
            if index == 1 {
                g.set_color(&COLOR_AUTOMATION_FOCUSED);
            } else {
                g.set_color(&COLOR_AUTOMATION);
            }
            let index = index as i32;
            let outer_diameter = GRID_2 - (KNOB_OUTSIDE_SPACE * 2) - lane_size * index * 2;
            let inner_diameter = outer_diameter - (lane_size - KNOB_LANE_GAP) * 2;
            let inset = (GRID_2 - outer_diameter) / 2;
            let min_angle = value_to_angle(control.range, *min);
            let max_angle = value_to_angle(control.range, *max);
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

impl WidgetImpl for IOTab {
    fn get_pos(&self) -> (i32, i32) {
        self.pos
    }

    fn draw(&self, g: &mut GrahpicsWrapper) {
        const MITS: i32 = MODULE_IO_TAB_SIZE;
        const MCS: i32 = MODULE_CORNER_SIZE;
        g.fill_rounded_rect(0, 0, MITS, MITS, MCS);
        let x = if self.is_output { MITS - MCS } else { 0 };
        g.fill_rect(x, 0, MCS, MITS);
    }
}

pub struct Module {
    module: Rcrc<engine::Module>,
    pos: (i32, i32),
    size: (i32, i32),
    label: String,
    children: Vec<Rcrc<dyn Widget>>,
}

impl Module {
    pub fn create(
        module: Rcrc<engine::Module>,
        pos: (i32, i32),
        grid_size: (i32, i32),
        label: String,
        mut children: Vec<Rcrc<dyn Widget>>,
    ) -> Self {
        const MIW: i32 = MODULE_IO_WIDTH;
        let size = (fatgrid(grid_size.0) + MIW * 2, fatgrid(grid_size.1));
        let module_ref = module.borrow();
        for index in 0..module_ref.num_inputs as i32 {
            children.push(rcrc(IOTab::input(0, coord(index))));
        }
        let x = size.0 - MODULE_IO_TAB_SIZE;
        for index in 0..module_ref.num_outputs as i32 {
            children.push(rcrc(IOTab::output(x, coord(index))));
        }
        drop(module_ref);
        Self {
            module,
            pos,
            size,
            label,
            children,
        }
    }
}

impl WidgetImpl for Module {
    fn get_pos(&self) -> (i32, i32) {
        self.pos
    }

    fn borrow_children(&self) -> &[Rcrc<dyn Widget>] {
        &self.children[..]
    }

    fn draw(&self, g: &mut GrahpicsWrapper) {
        const MCS: i32 = MODULE_CORNER_SIZE;
        const MIW: i32 = MODULE_IO_WIDTH;

        g.set_color(&COLOR_BG);
        g.clear();
        g.set_color(&COLOR_IO_AREA);
        g.fill_rounded_rect(0, 0, self.size.0, self.size.1, MCS);
        g.set_color(&COLOR_SURFACE);
        g.fill_rect(MIW, 0, self.size.0 - MIW * 2, self.size.1);

        g.set_color(&COLOR_TEXT);
    }
}

pub struct ModuleGraph {
    pub pos: (i32, i32),
    pub offset: (i32, i32),
    pub size: (i32, i32),
    children: Vec<Rcrc<dyn Widget>>,
}

impl ModuleGraph {
    pub fn adopt_child(&mut self, child: impl Widget + 'static) {
        self.children.push(rcrc(child))
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

impl WidgetImpl for ModuleGraph {
    fn get_pos(&self) -> (i32, i32) {
        self.pos
    }

    fn borrow_children(&self) -> &[Rcrc<dyn Widget>] {
        &self.children[..]
    }

    fn draw(&self, g: &mut GrahpicsWrapper) {
        g.apply_offset(self.offset.0, self.offset.1);
    }
}
