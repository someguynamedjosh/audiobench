use super::{AutomationSource, Control, IOData, IOType, UpdateRequest};
use crate::engine::codegen::AutomationCode;
use crate::engine::parts::JackType;
use crate::registry::yaml::YamlNode;
use shared_util::mini_serde::{MiniDes, MiniSer};

#[derive(Clone, Debug)]
pub struct TimingModeControl {
    /// True if time should be measured against how long the song has been running, false if time
    /// should be measured against how long the note has been running.
    use_elapsed_time: bool,
    /// True if time should be measured in seconds, false if time should be measured in beats.
    beat_synchronized: bool,
}

impl TimingModeControl {
    pub fn from_yaml(yaml: &YamlNode) -> Result<Self, String> {
        let use_elapsed_time = if let Ok(child) = yaml.unique_child("default_source") {
            child.parse_enumerated(&["note", "song"])? == 1
        } else {
            false
        };
        let beat_synchronized = if let Ok(child) = yaml.unique_child("default_units") {
            child.parse_enumerated(&["seconds", "beats"])? == 1
        } else {
            false
        };
        Ok(Self {
            use_elapsed_time,
            beat_synchronized,
        })
    }

    fn get_raw_value(&self) -> u8 {
        let source_flag = if self.use_elapsed_time { 0b1 } else { 0b0 };
        let unit_flag = if self.beat_synchronized { 0b10 } else { 0b00 };
        source_flag | unit_flag
    }

    pub fn uses_elapsed_time(&self) -> bool {
        self.use_elapsed_time
    }

    pub fn toggle_source(&mut self) -> UpdateRequest {
        self.use_elapsed_time = !self.use_elapsed_time;
        UpdateRequest::UpdateDynData
    }

    pub fn is_beat_synchronized(&self) -> bool {
        self.beat_synchronized
    }

    pub fn toggle_units(&mut self) -> UpdateRequest {
        self.beat_synchronized = !self.beat_synchronized;
        UpdateRequest::UpdateDynData
    }
}

#[rustfmt::skip] 
impl Control for TimingModeControl {
    fn get_parameter_types(&self) -> Vec<IOType> { vec![IOType::Int] }
    fn get_parameter_values(&self) -> Vec<IOData> { vec![IOData::Int(self.get_raw_value() as _)] }
    fn generate_code(&self, params: &[&str], automation_code: &AutomationCode) -> String { 
        assert!(params.len() == 1);
        params[0].to_owned()
    }
    fn serialize(&self, ser: &mut MiniSer) { 
        ser.u8(self.get_raw_value()); 
    }
    fn deserialize(&mut self, des: &mut MiniDes) -> Result<(), ()> { 
        let raw_value = des.u8()?;
        if raw_value > 0b11 {
            return Err(());
        }
        self.use_elapsed_time = raw_value & 0b1 == 0b1;
        self.beat_synchronized = raw_value & 0b10 == 0b10;
        Ok(())
    }
}
