use crate::gui::constants::*;
use crate::gui::widgets;
use crate::util::*;

#[derive(Default)]
pub struct Control {
    pub range: (f32, f32),
    pub value: f32,
    pub automation: Vec<(f32, f32)>,
}

#[derive(Clone)]
pub struct Module {
    gui_outline: Rcrc<GuiOutline>,
    controls: Vec<Rcrc<Control>>,
    pub num_inputs: usize,
    pub num_outputs: usize,
}

impl Module {
    pub fn example() -> Self {
        let gui_outline = rcrc(GuiOutline {
            widget_outlines: vec![
                WidgetOutline::Knob {
                    control_index: 0,
                    grid_pos: (0, 0),
                    label: "Pan".to_owned(),
                },
                WidgetOutline::Knob {
                    control_index: 1,
                    grid_pos: (2, 0),
                    label: "Amplitude".to_owned(),
                },
            ],
        });
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
            gui_outline,
            controls,
            num_inputs: 2,
            num_outputs: 1,
        }
    }

    fn instantiate_widget(&self, outline: &WidgetOutline) -> Rcrc<dyn widgets::Widget> {
        fn convert_grid_pos(grid_pos: &(i32, i32)) -> (i32, i32) {
            (MODULE_IO_WIDTH + coord(grid_pos.0), coord(grid_pos.1))
        }
        match outline {
            WidgetOutline::Knob {
                control_index,
                grid_pos,
                label,
            } => rcrc(widgets::Knob::create(
                Rc::clone(&self.controls[*control_index]),
                convert_grid_pos(grid_pos),
                label.clone(),
            )),
        }
    }

    pub fn example_gui(self) -> widgets::Module {
        let outline = self.gui_outline.borrow();
        let control_widgets = outline
            .widget_outlines
            .iter()
            .map(|wo| self.instantiate_widget(wo))
            .collect();
        drop(outline);

        widgets::Module::create(
            rcrc(self),
            (0, 0),
            (4, 2),
            "TEST".to_owned(),
            control_widgets,
        )
    }
}

pub struct GuiOutline {
    pub widget_outlines: Vec<WidgetOutline>,
}

pub enum WidgetOutline {
    Knob {
        control_index: usize,
        grid_pos: (i32, i32),
        label: String,
    },
}
