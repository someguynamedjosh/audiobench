use crate::graphics::{constants::*, GrahpicsWrapper};
use crate::util::RangeMap;
use std::cell::RefCell;
use std::f32::consts::PI;
use std::rc::Rc;

// This trait is convenient to implement for widgets, but inconvenient to call.
pub trait WidgetImpl {
    fn get_pos(&self) -> (i32, i32);
    fn borrow_children(&self) -> &[Rc<RefCell<dyn Widget>>] {
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
    pub pos: (i32, i32),
    pub bounds: (f32, f32),
    pub value: f32,
    pub label: String,
    pub automation: Vec<(f32, f32)>,
}

impl Default for Knob {
    fn default() -> Knob {
        Knob {
            pos: (0, 0),
            bounds: (-1.0, 1.0),
            value: 0.0,
            label: "UNLABELED".to_owned(),
            automation: Vec::new(),
        }
    }
}

impl WidgetImpl for Knob {
    fn get_pos(&self) -> (i32, i32) {
        self.pos
    }

    fn draw(&self, g: &mut GrahpicsWrapper) {
        fn value_to_angle(slf: &Knob, value: f32) -> f32 {
            value.from_range_to_range(slf.bounds.0, slf.bounds.1, PI, 0.0)
        }

        g.set_color(&COLOR_BG);
        g.fill_pie(0, 0, GRID_2, 0, 0.0, PI);
        g.set_color(&COLOR_KNOB);
        let zero_angle = value_to_angle(self, 0.0);
        let value_angle = value_to_angle(self, self.value);
        g.fill_pie(0, 0, GRID_2, 0, zero_angle, value_angle);
        g.set_color(&COLOR_TEXT);
        g.write_label(0, GRID_1 + GRID_P, GRID_2, &self.label);

        if self.automation.len() == 0 {
            return;
        }

        let num_lanes = self.automation.len() as i32;
        let lane_size = KNOB_AUTOMATION_SPACE / num_lanes;
        let lane_size = lane_size.min(KNOB_MAX_LANE_SIZE);
        for (index, (min, max)) in self.automation.iter().enumerate() {
            if index == 1 {
                g.set_color(&COLOR_AUTOMATION_FOCUSED);
            } else {
                g.set_color(&COLOR_AUTOMATION);
            }
            let index = index as i32;
            let outer_diameter = GRID_2 - (KNOB_OUTSIDE_SPACE * 2) - lane_size * index * 2;
            let inner_diameter = outer_diameter - (lane_size - KNOB_LANE_GAP) * 2;
            let inset = (GRID_2 - outer_diameter) / 2;
            let min_angle = value_to_angle(self, *min);
            let max_angle = value_to_angle(self, *max);
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

#[derive(Clone)]
pub struct Module {
    pub pos: (i32, i32),
    pub size: (i32, i32),
    pub children: Vec<Rc<RefCell<dyn Widget>>>,
    pub num_inputs: usize,
    pub num_outputs: usize,
    pub label: String,
}

impl Module {
    pub fn adopt_child(&mut self, child: impl Widget + 'static) {
        self.children.push(Rc::from(RefCell::new(child)))
    }
}

impl Default for Module {
    fn default() -> Module {
        Module {
            pos: (0, 0),
            size: (FATGRID_2, FATGRID_2),
            children: Vec::new(),
            num_inputs: 0,
            num_outputs: 0,
            label: "UNLABELED".to_owned(),
        }
    }
}

impl WidgetImpl for Module {
    fn get_pos(&self) -> (i32, i32) {
        self.pos
    }

    fn borrow_children(&self) -> &[Rc<RefCell<dyn Widget>>] {
        &self.children[..]
    }

    fn draw(&self, g: &mut GrahpicsWrapper) {
        const IOTS: i32 = IO_TAB_SIZE;
        const MCS: i32 = MODULE_CORNER_SIZE;

        g.set_color(&COLOR_BG);
        g.clear();
        g.set_color(&COLOR_SURFACE);
        g.fill_rounded_rect(-IOTS, 0, self.size.0 + IOTS * 2, self.size.1, MCS);

        g.set_color(&COLOR_TEXT);
        for index in 0..self.num_inputs as i32 {
            let y = coord(index);
            g.fill_rounded_rect(-IOTS, y, IOTS, IOTS, MCS);
            g.fill_rect(-IOTS, y, MCS, IOTS);
        }
        for index in 0..self.num_outputs as i32 {
            let y = coord(index);
            g.fill_rounded_rect(self.size.0, y, IOTS, IOTS, MCS);
            g.fill_rect(self.size.0 + (IOTS - MCS), y, MCS, IOTS);
        }
    }
}

pub struct ModuleGraph {
    pub pos: (i32, i32),
    pub offset: (i32, i32),
    pub size: (i32, i32),
    children: Vec<Rc<RefCell<dyn Widget>>>,
}

impl ModuleGraph {
    pub fn adopt_child(&mut self, child: impl Widget + 'static) {
        self.children.push(Rc::from(RefCell::new(child)))
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

    fn borrow_children(&self) -> &[Rc<RefCell<dyn Widget>>] {
        &self.children[..]
    }

    fn draw(&self, g: &mut GrahpicsWrapper) {
        g.apply_offset(self.offset.0, self.offset.1);
    }
}
