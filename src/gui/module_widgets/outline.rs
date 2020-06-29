use super::*;
use crate::engine::parts as ep;
use crate::engine::registry::Registry;
use crate::gui::constants::*;
use crate::util::*;

pub enum FeedbackDataRequirement {
    None,
    Control { control_index: usize },
    Custom { code_name: String, size: usize },
}

impl FeedbackDataRequirement {
    pub fn size(&self) -> usize {
        match self {
            Self::None => 0,
            Self::Control { .. } => 1,
            Self::Custom { size, .. } => *size,
        }
    }
}

#[derive(Debug)]
pub enum WidgetOutline {
    Knob {
        tooltip: String,
        control_index: usize,
        grid_pos: (i32, i32),
        label: String,
    },
    WaveformGraph {
        grid_pos: (i32, i32),
        grid_size: (i32, i32),
        feedback_name: String,
    },
    EnvelopeGraph {
        grid_pos: (i32, i32),
        grid_size: (i32, i32),
        feedback_name: String,
    },
    IntBox {
        tooltip: String,
        ccontrol_index: usize,
        grid_pos: (i32, i32),
        range: (i32, i32),
        label: String,
    },
    HertzBox {
        tooltip: String,
        ccontrol_index: usize,
        grid_pos: (i32, i32),
        range: (f32, f32),
        label: String,
    },
    OptionBox {
        tooltip: String,
        ccontrol_index: usize,
        grid_pos: (i32, i32),
        grid_size: (i32, i32),
        options: Vec<String>,
        label: String,
    },
    TimingSelector {
        source_control_index: usize,
        type_control_index: usize,
        grid_pos: (i32, i32),
    },
}

impl WidgetOutline {
    pub fn get_feedback_data_requirement(&self) -> FeedbackDataRequirement {
        match self {
            Self::Knob { control_index, .. } => FeedbackDataRequirement::Control {
                control_index: *control_index,
            },
            Self::WaveformGraph { feedback_name, .. } => FeedbackDataRequirement::Custom {
                code_name: feedback_name.clone(),
                size: 60,
            },
            Self::EnvelopeGraph { feedback_name, .. } => FeedbackDataRequirement::Custom {
                code_name: feedback_name.clone(),
                size: 6,
            },
            Self::IntBox { .. } => FeedbackDataRequirement::None,
            Self::HertzBox { .. } => FeedbackDataRequirement::None,
            Self::OptionBox { .. } => FeedbackDataRequirement::None,
            Self::TimingSelector { .. } => FeedbackDataRequirement::None,
        }
    }
}

pub(in crate::gui) fn widget_from_outline(
    registry: &Registry,
    controls: &Vec<Rcrc<ep::Control>>,
    ccontrols: &Vec<Rcrc<ep::ComplexControl>>,
    outline: &WidgetOutline,
    // usize is the amount of feedback data the widget uses.
) -> (Box<dyn ModuleWidget>, usize) {
    fn convert_grid_pos(grid_pos: (i32, i32)) -> (f32, f32) {
        (
            MODULE_IO_WIDTH + JACK_SIZE + coord(grid_pos.0),
            coord(grid_pos.1),
        )
    }
    fn convert_grid_size(grid_size: (i32, i32)) -> (f32, f32) {
        (grid(grid_size.0), grid(grid_size.1))
    }

    let widget: Box<dyn ModuleWidget> = match outline {
        WidgetOutline::Knob {
            tooltip,
            control_index,
            grid_pos,
            label,
        } => Box::new(Knob::create(
            tooltip.clone(),
            Rc::clone(&controls[*control_index]),
            convert_grid_pos(*grid_pos),
            label.clone(),
        )),
        WidgetOutline::WaveformGraph {
            grid_pos,
            grid_size,
            ..
        } => Box::new(WaveformGraph::create(
            convert_grid_pos(*grid_pos),
            convert_grid_size(*grid_size),
        )),
        WidgetOutline::EnvelopeGraph {
            grid_pos,
            grid_size,
            ..
        } => Box::new(EnvelopeGraph::create(
            convert_grid_pos(*grid_pos),
            convert_grid_size(*grid_size),
        )),
        WidgetOutline::IntBox {
            tooltip,
            ccontrol_index,
            grid_pos,
            range,
            label,
            ..
        } => Box::new(IntBox::create(
            tooltip.clone(),
            registry,
            Rc::clone(&ccontrols[*ccontrol_index]),
            convert_grid_pos(*grid_pos),
            *range,
            label.clone(),
        )),
        WidgetOutline::HertzBox {
            tooltip,
            ccontrol_index,
            grid_pos,
            range,
            label,
            ..
        } => Box::new(HertzBox::create(
            tooltip.clone(),
            registry,
            Rc::clone(&ccontrols[*ccontrol_index]),
            convert_grid_pos(*grid_pos),
            *range,
            label.clone(),
        )),
        WidgetOutline::OptionBox {
            tooltip,
            ccontrol_index,
            grid_pos,
            grid_size,
            options,
            label,
            ..
        } => Box::new(OptionBox::create(
            tooltip.clone(),
            Rc::clone(&ccontrols[*ccontrol_index]),
            convert_grid_pos(*grid_pos),
            convert_grid_size(*grid_size),
            options.clone(),
            label.clone(),
        )),
        WidgetOutline::TimingSelector {
            source_control_index,
            type_control_index,
            grid_pos,
        } => Box::new(TimingSelector::create(
            Rc::clone(&ccontrols[*source_control_index]),
            Rc::clone(&ccontrols[*type_control_index]),
            convert_grid_pos(*grid_pos),
            registry,
        )),
    };
    let feedback_data_len = outline.get_feedback_data_requirement().size();
    (widget, feedback_data_len)
}
