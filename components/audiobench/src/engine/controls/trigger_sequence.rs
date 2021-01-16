use super::{AutomationSource, Control, UpdateRequest};
use crate::engine::codegen::AutomationCode;
use crate::engine::data_transfer::{IOData, IOType};
use crate::engine::parts::JackType;
use crate::registry::yaml::YamlNode;
use shared_util::mini_serde::{MiniDes, MiniSer};

#[derive(Clone, Debug)]
pub struct TriggerSequenceControl {
    sequence: Vec<bool>,
}

impl TriggerSequenceControl {
    pub fn from_yaml(yaml: &YamlNode) -> Result<Self, String> {
        Ok(Self {
            sequence: vec![true, false, false, false],
        })
    }

    pub fn get_len(&self) -> usize {
        self.sequence.len()
    }

    pub fn set_len(&mut self, len: usize) -> UpdateRequest {
        if self.sequence.len() == len {
            return UpdateRequest::Nothing;
        }
        self.sequence.resize(len, false);
        // Changing the length changes the data type of the information dynamically passed in for
        // this control, so the code has to be updated.
        UpdateRequest::UpdateCode
    }

    pub fn get_trigger(&self, index: usize) -> bool {
        assert!(index < self.get_len());
        self.sequence[index]
    }

    pub fn toggle_trigger(&mut self, index: usize) -> UpdateRequest {
        assert!(index < self.get_len());
        self.sequence[index] = !self.sequence[index];
        UpdateRequest::UpdateDynData
    }
}

#[rustfmt::skip] 
impl Control for TriggerSequenceControl {
    fn get_parameter_types(&self) -> Vec<IOType> { unimplemented!() }
    fn get_parameter_values(&self) -> Vec<IOData> { unimplemented!() }
    fn generate_code(&self, params: &[&str], automation_code: &AutomationCode) -> String { 
        unimplemented!() 
    }
    fn serialize(&self, ser: &mut MiniSer) { 
        assert!(self.sequence.len() <= 0xFF);
        ser.u8(self.sequence.len() as u8); 
        for bool in &self.sequence {
            ser.bool(*bool);
        }
    }
    fn deserialize(&mut self, des: &mut MiniDes) -> Result<(), ()> { 
        let len = des.u8()?;
        self.sequence = Vec::new();
        for _ in 0..len {
            self.sequence.push(des.bool()?);
        }
        Ok(())
    }
}
