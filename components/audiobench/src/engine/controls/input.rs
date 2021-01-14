use super::{AutomationSource, Control, IOData, IOType};
use crate::engine::codegen::AutomationCode;
use crate::engine::parts::JackType;
use crate::registry::yaml::YamlNode;
use std::collections::HashMap;

struct DefaultInputDescription {
    name: &'static str,
    code: &'static str,
    icon: &'static str,
}

fn default_option_descriptions_for(typ: JackType) -> &'static [DefaultInputDescription] {
    match typ {
        JackType::Pitch => &[DefaultInputDescription {
            name: "Note Pitch",
            code: "StaticControlSignal(note_input.pitch)",
            icon: "Factory:note",
        }],
        JackType::Waveform => &[
            DefaultInputDescription {
                name: "Silence",
                code: "flat_waveform",
                // TODO: Better icon.
                icon: "Factory:nothing",
            },
            DefaultInputDescription {
                name: "Ramp Up",
                code: "ramp_up_waveform",
                icon: "Factory:ramp_up",
            },
            DefaultInputDescription {
                name: "Ramp Down",
                code: "ramp_down_waveform",
                icon: "Factory:ramp_down",
            },
            DefaultInputDescription {
                name: "Sine Wave",
                code: "sine_waveform",
                icon: "Factory:sine_wave",
            },
        ],
        JackType::Audio => &[DefaultInputDescription {
            name: "Silence",
            code: "StaticMonoAudio(0f0)",
            icon: "Factory:nothing",
        }],
        JackType::Trigger => &[
            DefaultInputDescription {
                name: "Note Start",
                code: "global_start_trigger",
                icon: "Factory:note_down",
            },
            DefaultInputDescription {
                name: "Note Release",
                code: "global_release_trigger",
                icon: "Factory:note_up",
            },
            DefaultInputDescription {
                name: "Never",
                code: "StaticTrigger(False)",
                icon: "Factory:nothing",
            },
        ],
    }
}

#[derive(Clone, Debug)]
pub struct DefaultInput {
    pub name: &'static str,
    pub code: &'static str,
    pub icon: usize,
}

fn default_options_for(typ: JackType, icon_indexes: &HashMap<String, usize>) -> Vec<DefaultInput> {
    default_option_descriptions_for(typ)
        .iter()
        .map(|desc| DefaultInput {
            name: desc.name,
            code: desc.code,
            // The factory library should have all the listed icons.
            icon: *icon_indexes.get(desc.icon).unwrap(),
        })
        .collect()
}

#[derive(Clone, Debug)]
pub struct InputControl {
    typ: JackType,
    default: usize,
    connection: Option<AutomationSource>,
}

impl InputControl {
    pub fn from_yaml(yaml: &YamlNode) -> Result<Self, String> {
        let typ = JackType::from_yaml(yaml.unique_child("type")?)?;
        let default = if let Ok(child) = yaml.unique_child("default") {
            child.parse()?
        } else {
            0
        };
        Ok(Self {
            typ,
            default,
            connection: None,
        })
    }

    pub fn get_type(&self) -> JackType {
        self.typ
    }
}

#[rustfmt::skip]
impl Control for InputControl {
    fn is_static_only(&self) -> bool { false }
    fn acceptable_automation(&self) -> Vec<JackType> { vec![self.typ] }
    fn connect_automation(&mut self, from: AutomationSource) {
        assert_eq!(from.get_type(), self.typ);
        self.connection = Some(from);
    }
    fn get_parameter_types(&self) -> Vec<IOType> { vec![] }
    fn get_parameter_values(&self) -> Vec<IOData> { vec![] }
    fn generate_code(&self, params: &[&str], automation_code: &AutomationCode) -> String { 
        unimplemented!() 
    }
    fn serialize(&self, buffer: &mut Vec<u8>) { unimplemented!() }
    fn deserialize(&mut self, data: &mut &[u8]) -> Result<(), ()> { unimplemented!() }
}
