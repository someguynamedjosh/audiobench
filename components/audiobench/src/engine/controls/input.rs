use crate::{
    engine::{
        codegen::AutomationCode,
        controls::{AutomationSource, Control},
        data_transfer::{IOData, IOType},
        parts::JackType,
    },
    registry::yaml::YamlNode,
};
use shared_util::mini_serde::{MiniDes, MiniSer};

pub struct DefaultInputDescription {
    pub name: &'static str,
    pub code: &'static str,
    pub icon: &'static str,
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
                code: "start_trigger",
                icon: "Factory:note_down",
            },
            DefaultInputDescription {
                name: "Note Release",
                code: "release_trigger",
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
pub struct InputControl {
    typ: JackType,
    default: usize,
    connection: Option<AutomationSource>,
}

impl InputControl {
    pub fn from_yaml(yaml: &YamlNode) -> Result<Self, String> {
        let typ = JackType::from_yaml(yaml.unique_child("type")?)?;
        let default = if let Ok(child) = yaml.unique_child("default") {
            let mut names = Vec::new();
            for option in default_option_descriptions_for(typ) {
                names.push(option.name.to_lowercase().replace(' ', "_"));
            }
            let name_refs: Vec<_> = names.iter().map(|e| &e[..]).collect();
            child.parse_enumerated(&name_refs[..])?
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

    /// Returns None if this input is connected and not using its default.
    pub fn get_used_default(&self) -> Option<&'static DefaultInputDescription> {
        if self.connection.is_none() {
            Some(&default_option_descriptions_for(self.get_type())[self.default])
        } else {
            None
        }
    }

    pub fn next_default(&mut self) {
        self.default = (self.default + 1) % default_option_descriptions_for(self.get_type()).len();
    }
}

#[rustfmt::skip]
impl Control for InputControl {
    fn acceptable_automation(&self) -> Vec<JackType> { vec![self.typ] }
    fn connect_automation(&mut self, from: AutomationSource) {
        assert_eq!(from.get_type(), self.typ);
        self.connection = Some(from);
    }
    fn get_connected_automation<'a>(&'a self) -> Box<dyn Iterator<Item = &'a AutomationSource> + 'a> {
        Box::new(self.connection.iter())
    }

    fn get_parameter_types(&self) -> Vec<IOType> { vec![] }
    fn get_parameter_values(&self) -> Vec<IOData> { vec![] }
    fn generate_code(&self, params: &[&str], automation_code: &AutomationCode) -> String { 
        if let Some(connection) = &self.connection {
            automation_code.value_of(connection)
        } else {
            self.get_used_default().unwrap().code.to_owned()
        }
    }
    fn serialize(&self, ser: &mut MiniSer) { unimplemented!() }
    fn deserialize(&mut self, des: &mut MiniDes) -> Result<(), ()> { unimplemented!() }
}
