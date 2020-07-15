use super::*;

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

yaml_widget_boilerplate::make_widget_outline_enum![
    DurationBox,
    EnvelopeGraph,
    HertzBox,
    IntBox,
    Knob,
    OptionBox,
    TimingSelector,
    TriggerSequence,
    TriggerSequenceLength,
    WaveformGraph,
];
