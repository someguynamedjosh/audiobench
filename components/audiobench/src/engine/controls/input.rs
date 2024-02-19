use shared_util::mini_serde::{MiniDes, MiniSer};

use crate::{
    engine::{
        codegen::AutomationCode,
        controls::{AutomationSource, Control},
        data_transfer::{IOData, IOType},
        parts::JackType,
    },
    registry::yaml::YamlNode,
};

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
                code: "StaticTrigger(false)",
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
    pub const fn new(typ: JackType, default: usize) -> Self {
        Self {
            typ,
            default,
            connection: None,
        }
    }

    pub fn from_yaml(mut yaml: YamlNode) -> Result<Self, String> {
        let typ = JackType::from_yaml(&yaml.map_entry("datatype")?)?;
        let default = if let Ok(child) = yaml.map_entry("default") {
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

    pub fn disconnect(&mut self) {
        self.connection = None;
    }
}

#[rustfmt::skip]
impl Control for InputControl {
    fn acceptable_automation(&self) -> Vec<JackType> { vec![self.typ] }
    fn connect_automation(&mut self, from: AutomationSource) {
        assert_eq!(from.get_type(), self.typ);
        self.connection = Some(from);
    }
    fn get_connected_automation<'a>(&'a self) -> Vec<&'a AutomationSource> {
        self.connection.iter().collect()
    }
    fn remove_automation_by_index(&mut self, index: usize) {
        assert_eq!(index, 0);
        self.connection = None;
    }

    fn get_parameter_types(&self) -> Vec<IOType> { vec![] }
    fn get_parameter_values(&self) -> Vec<IOData> { vec![] }
    fn generate_code(&self, _params: &[&str], automation_code: &AutomationCode) -> String { 
        if let Some(connection) = &self.connection {
            automation_code.value_of(connection)
        } else {
            self.get_used_default().unwrap().code.to_owned()
        }
    }
    fn serialize(&self, ser: &mut MiniSer) {
        assert!(self.default < 16);
        ser.u4(self.default as _);
    }
    fn deserialize(&mut self, des: &mut MiniDes) -> Result<(), ()> {
        self.default = des.u4()? as _;
        Ok(())
    }
}
