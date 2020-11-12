use super::*;

pub enum FeedbackDataRequirement {
    None,
    Autocon { control_index: usize },
    Custom { code_name: String, size: usize },
}

impl FeedbackDataRequirement {
    pub fn size(&self) -> usize {
        match self {
            Self::None => 0,
            Self::Autocon { .. } => 1,
            Self::Custom { size, .. } => *size,
        }
    }
}

yaml_widget_boilerplate::make_widget_outline_enum![
    TimingSelector
];

// yaml_widget_boilerplate::make_widget_outline_enum![
//     DurationBox,
//     EnvelopeGraph,
//     FrequencyBox,
//     HSlider,
//     IntBox,
//     Knob,
//     MiniKnob,
//     OptionBox,
//     TimingSelector,
//     TriggerSequence,
//     TriggerSequenceLength,
//     ValueSequence,
//     ValueSequenceLength,
//     WaveformGraph,
// ];
