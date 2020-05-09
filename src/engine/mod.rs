use crate::gui::constants::*;
use crate::gui::widgets;
use crate::util::*;

#[derive(Default)]
pub struct Control {
    pub range: (f32, f32),
    pub value: f32,
    pub automation: Vec<(f32, f32)>,
}

pub struct Module {
    controls: Vec<Rcrc<Control>>,
    pub num_inputs: usize,
    pub num_outputs: usize,
}

impl Module {
    pub fn example() -> Self {
        let controls = vec![
            rcrc(Control {
                range: (-1.0, 1.0),
                value: 0.5,
                automation: vec![],
            }),
            rcrc(Control {
                range: (0.0, 10.0),
                value: 2.0,
                automation: vec![(5.0, 9.0), (5.0, 6.0)],
            }),
        ];
        Self {
            controls,
            num_inputs: 2,
            num_outputs: 1,
        }
    }

    pub fn example_gui(self) -> widgets::Module {
        let mut control_widgets = Vec::<Rcrc<dyn widgets::Widget>>::new();

        control_widgets.push(rcrc(widgets::Knob::create(
            Rc::clone(&self.controls[0]),
            (coord(0) + MODULE_IO_WIDTH, coord(0)),
            "Volume".to_owned(),
        )));
        control_widgets.push(rcrc(widgets::Knob::create(
            Rc::clone(&self.controls[1]),
            (coord(2) + MODULE_IO_WIDTH, coord(0)),
            "Amplitude".to_owned(),
        )));

        widgets::Module::create(
            rcrc(self),
            (0, 0),
            (4, 2),
            "TEST".to_owned(),
            control_widgets,
        )
    }
}
