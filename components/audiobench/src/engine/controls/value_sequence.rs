use crate::{
    engine::{
        codegen::AutomationCode,
        controls::{Control, UpdateRequest},
        data_transfer::{IOData, IOType},
    },
    registry::yaml::YamlNode,
};
use shared_util::mini_serde::{MiniDes, MiniSer};

#[derive(Clone, Debug)]
pub struct ValueSequenceControl {
    sequence: Vec<f32>,
}

impl ValueSequenceControl {
    pub fn from_yaml(yaml: &YamlNode) -> Result<Self, String> {
        Ok(Self {
            sequence: vec![1.0, -1.0, -1.0, -1.0],
        })
    }

    pub fn get_len(&self) -> usize {
        self.sequence.len()
    }

    pub fn set_len(&mut self, len: usize) -> UpdateRequest {
        if self.sequence.len() == len {
            return UpdateRequest::Nothing;
        }
        self.sequence.resize(len, -1.0);
        // Changing the length changes the data type of the information dynamically passed in for
        // this control, so the code has to be updated.
        UpdateRequest::UpdateCode
    }

    pub fn get_value(&self, index: usize) -> f32 {
        assert!(index < self.get_len());
        self.sequence[index]
    }

    pub fn set_value(&mut self, index: usize, value: f32) -> UpdateRequest {
        assert!(index < self.get_len());
        if self.sequence[index] == value {
            return UpdateRequest::Nothing;
        }
        self.sequence[index] = value;
        UpdateRequest::UpdateDynData
    }
}

#[rustfmt::skip] 
impl Control for ValueSequenceControl {
    fn get_parameter_types(&self) -> Vec<IOType> { unimplemented!() }
    fn get_parameter_values(&self) -> Vec<IOData> { unimplemented!() }
    fn generate_code(&self, params: &[&str], automation_code: &AutomationCode) -> String { 
        unimplemented!() 
    }
    fn serialize(&self, ser: &mut MiniSer) { 
        assert!(self.sequence.len() <= 0xFF);
        ser.u8(self.sequence.len() as u8); 
        for value in &self.sequence {
            ser.f32_in_range(*value, -1.0, 1.0);
        }
    }
    fn deserialize(&mut self, des: &mut MiniDes) -> Result<(), ()> { 
        let len = des.u8()?;
        self.sequence.clear();
        for _ in 0..len {
            self.sequence.push(des.f32_in_range(-1.0, 1.0)?);
        }
        Ok(())
    }
}
