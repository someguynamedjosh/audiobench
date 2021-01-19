use crate::gui::module_widgets::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FeedbackMode {
    None,
    ControlSignal { control_index: usize },
    ManualValue { name: String },
}

yaml_widget_boilerplate::make_widget_outline_enum![
    DurationBox,
    EnvelopeGraph,
    FrequencyBox,
    HSlider,
    Input,
    IntBox,
    Knob,
    MiniKnob,
    OptionBox,
    TimingSelector,
    TriggerSequence,
    TriggerSequenceLength,
    ValueSequence,
    ValueSequenceLength,
    WaveformGraph,
];
