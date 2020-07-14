use super::yaml::YamlNode;
use super::Registry;
use crate::engine::parts as ep;
use crate::gui::constants::*;
use crate::gui::module_widgets::*;
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
    DurationBox {
        tooltip: String,
        ccontrol_index: usize,
        type_control_index: usize,
        grid_pos: (i32, i32),
        label: String,
    },
    TriggerSequence {
        tooltip: String,
        sequence_control_index: usize,
        grid_pos: (i32, i32),
        grid_size: (i32, i32),
        feedback_name: String,
    },
    TriggerSequenceLength {
        tooltip: String,
        sequence_control_index: usize,
        grid_pos: (i32, i32),
        label: String,
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
                size: 44, // Graph normally takes up 42 pixels on default zoom.
            },
            Self::EnvelopeGraph { feedback_name, .. } => FeedbackDataRequirement::Custom {
                code_name: feedback_name.clone(),
                size: 6,
            },
            Self::IntBox { .. } => FeedbackDataRequirement::None,
            Self::HertzBox { .. } => FeedbackDataRequirement::None,
            Self::OptionBox { .. } => FeedbackDataRequirement::None,
            Self::TimingSelector { .. } => FeedbackDataRequirement::None,
            Self::DurationBox { .. } => FeedbackDataRequirement::None,
            Self::TriggerSequence { feedback_name, .. } => FeedbackDataRequirement::Custom {
                code_name: feedback_name.clone(),
                size: 1, // Current position in sequence
            },
            Self::TriggerSequenceLength { .. } => FeedbackDataRequirement::None,
        }
    }
}

pub(super) fn outline_from_yaml(
    yaml: &YamlNode,
    controls: &Vec<Rcrc<ep::Control>>,
    complex_controls: &mut Vec<Rcrc<ep::ComplexControl>>,
) -> Result<WidgetOutline, String> {
    let x = yaml.unique_child("x")?.i32()?;
    let y = yaml.unique_child("y")?.i32()?;
    let grid_pos = (x, y);
    let tooltip_node = yaml.unique_child("tooltip");
    let find_control_index = |name: &str| {
        controls
            .iter()
            .position(|item| &item.borrow().code_name == name)
            .ok_or_else(|| {
                format!(
                    "ERROR: Invalid widget {}, caused by:\nERROR: No control named {}.",
                    &yaml.full_name, name
                )
            })
    };
    let find_complex_control_index = |name: &str| {
        complex_controls
            .iter()
            .position(|item| &item.borrow().code_name == name)
            .ok_or_else(|| {
                format!(
                    "ERROR: Invalid widget {}, caused by:\nERROR: No complex control named {}.",
                    &yaml.full_name, name
                )
            })
    };
    let mut set_default = Vec::new();
    let outline = match &yaml.name[..] {
        "knob" => {
            let control_name = &yaml.unique_child("control")?.value;
            let control_index = find_control_index(control_name)?;
            let label = yaml.unique_child("label")?.value.clone();
            WidgetOutline::Knob {
                tooltip: tooltip_node?.value.clone(),
                control_index,
                grid_pos,
                label,
            }
        }
        "envelope_graph" => {
            let grid_size = (
                yaml.unique_child("w")?.i32()?,
                yaml.unique_child("h")?.i32()?,
            );
            let feedback_name = yaml.unique_child("feedback_name")?.value.clone();
            WidgetOutline::EnvelopeGraph {
                grid_pos,
                grid_size,
                feedback_name,
            }
        }
        "waveform_graph" => {
            let grid_size = (
                yaml.unique_child("w")?.i32()?,
                yaml.unique_child("h")?.i32()?,
            );
            let feedback_name = yaml.unique_child("feedback_name")?.value.clone();
            WidgetOutline::WaveformGraph {
                grid_pos,
                grid_size,
                feedback_name,
            }
        }
        "int_box" => {
            let ccontrol_name = &yaml.unique_child("control")?.value;
            let ccontrol_index = find_complex_control_index(ccontrol_name)?;
            let min = yaml.unique_child("min")?.i32()?;
            let max = yaml.unique_child("max")?.i32()?;
            let default = if let Ok(child) = yaml.unique_child("default") {
                child.i32()?
            } else {
                min
            };
            let label = yaml.unique_child("label")?.value.clone();
            set_default.push((ccontrol_index, format!("{}", default)));
            WidgetOutline::IntBox {
                tooltip: tooltip_node?.value.clone(),
                ccontrol_index,
                grid_pos,
                range: (min, max),
                label,
            }
        }
        "hertz_box" => {
            let ccontrol_name = &yaml.unique_child("control")?.value;
            let ccontrol_index = find_complex_control_index(ccontrol_name)?;
            let min = yaml.unique_child("min")?.f32()?;
            let max = yaml.unique_child("max")?.f32()?;
            let default = if let Ok(child) = yaml.unique_child("default") {
                child.f32()?
            } else {
                min
            };
            let label = yaml.unique_child("label")?.value.clone();
            set_default.push((ccontrol_index, format!("{:.1}", default)));
            WidgetOutline::HertzBox {
                tooltip: tooltip_node?.value.clone(),
                ccontrol_index,
                grid_pos,
                range: (min, max),
                label,
            }
        }
        "option_box" => {
            let grid_size = (
                yaml.unique_child("w")?.i32()?,
                yaml.unique_child("h")?.i32()?,
            );
            let ccontrol_name = &yaml.unique_child("control")?.value;
            let ccontrol_index = find_complex_control_index(ccontrol_name)?;
            let default = if let Ok(child) = yaml.unique_child("default") {
                child.i32()?
            } else {
                0
            };
            let mut options = Vec::new();
            for child in &yaml.unique_child("options")?.children {
                options.push(child.name.clone());
            }
            if options.len() < 2 {
                return Err(format!(
                    concat!(
                        "ERROR: Invalid widget {}, caused by:\n",
                        "ERROR: Option box must have at least 2 options."
                    ),
                    &yaml.full_name
                ));
            }
            let label = yaml.unique_child("label")?.value.clone();
            set_default.push((ccontrol_index, format!("{}", default)));
            WidgetOutline::OptionBox {
                tooltip: tooltip_node?.value.clone(),
                ccontrol_index,
                grid_pos,
                grid_size,
                options,
                label,
            }
        }
        "timing_selector" => {
            let source_control_name = &yaml.unique_child("source_control")?.value;
            let source_control_index = find_complex_control_index(source_control_name)?;
            set_default.push((
                source_control_index,
                if yaml.unique_child("default_song_source").is_ok() {
                    "TRUE"
                } else {
                    "FALSE"
                }
                .to_owned(),
            ));
            let type_control_name = &yaml.unique_child("type_control")?.value;
            let type_control_index = find_complex_control_index(type_control_name)?;
            set_default.push((
                type_control_index,
                if yaml.unique_child("default_beats_type").is_ok() {
                    "TRUE"
                } else {
                    "FALSE"
                }
                .to_owned(),
            ));
            WidgetOutline::TimingSelector {
                source_control_index,
                type_control_index,
                grid_pos,
            }
        }
        "duration_box" => {
            let ccontrol_name = &yaml.unique_child("control")?.value;
            let ccontrol_index = find_complex_control_index(ccontrol_name)?;
            let default = if let Ok(child) = yaml.unique_child("default") {
                child.value.clone()
            } else {
                "1.00".to_owned()
            };
            set_default.push((ccontrol_index, default));
            let type_control_name = &yaml.unique_child("type_control")?.value;
            let type_control_index = find_complex_control_index(type_control_name)?;
            let label = yaml.unique_child("label")?.value.clone();
            WidgetOutline::DurationBox {
                tooltip: tooltip_node?.value.clone(),
                ccontrol_index,
                type_control_index,
                grid_pos,
                label,
            }
        }
        "trigger_sequence" => {
            let tooltip = yaml.unique_child("tooltip")?.value.clone();
            let sequence_control_name = &yaml.unique_child("sequence_control")?.value;
            let sequence_control_index = find_complex_control_index(sequence_control_name)?;
            let grid_size = (
                yaml.unique_child("w")?.i32()?,
                yaml.unique_child("h")?.i32()?,
            );
            let feedback_name = yaml.unique_child("feedback_name")?.value.clone();
            WidgetOutline::TriggerSequence {
                tooltip,
                sequence_control_index,
                grid_pos,
                grid_size,
                feedback_name,
            }
        }
        "trigger_sequence_length" => {
            let sequence_control_name = &yaml.unique_child("sequence_control")?.value;
            let sequence_control_index = find_complex_control_index(sequence_control_name)?;
            let label = yaml.unique_child("label")?.value.clone();
            set_default.push((
                sequence_control_index,
                "[TRUE ,FALSE,FALSE,FALSE,]".to_owned(),
            ));
            WidgetOutline::TriggerSequenceLength {
                tooltip: tooltip_node?.value.clone(),
                sequence_control_index,
                grid_pos,
                label,
            }
        }
        _ => {
            return Err(format!(
                "ERROR: Invalid widget {}, caused by:\nERROR: {} is not a valid widget type.",
                &yaml.full_name, &yaml.name
            ))
        }
    };
    for (index, value) in set_default {
        if complex_controls[index].borrow().value != "" {
            return Err(format!(
                "ERROR: Multiple widgets controlling the same complex control {}.",
                complex_controls[index].borrow().code_name
            ));
        }
        complex_controls[index].borrow_mut().default = value.clone();
        complex_controls[index].borrow_mut().value = value;
    }
    Ok(outline)
}

pub fn widget_from_outline(
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
        WidgetOutline::DurationBox {
            tooltip,
            ccontrol_index,
            type_control_index,
            grid_pos,
            label,
            ..
        } => Box::new(DurationBox::create(
            tooltip.clone(),
            Rc::clone(&ccontrols[*ccontrol_index]),
            Rc::clone(&ccontrols[*type_control_index]),
            convert_grid_pos(*grid_pos),
            label.clone(),
        )),
        WidgetOutline::TriggerSequence {
            tooltip,
            sequence_control_index,
            grid_pos,
            grid_size,
            ..
        } => Box::new(TriggerSequence::create(
            tooltip.clone(),
            Rc::clone(&ccontrols[*sequence_control_index]),
            convert_grid_pos(*grid_pos),
            convert_grid_size(*grid_size),
        )),
        WidgetOutline::TriggerSequenceLength {
            tooltip,
            sequence_control_index,
            grid_pos,
            label,
            ..
        } => Box::new(TriggerSequenceLength::create(
            tooltip.clone(),
            registry,
            Rc::clone(&ccontrols[*sequence_control_index]),
            convert_grid_pos(*grid_pos),
            label.clone(),
        )),
    };
    let feedback_data_len = outline.get_feedback_data_requirement().size();
    (widget, feedback_data_len)
}
