use super::{AutomationSource, Control, IOData, IOType, UpdateRequest};
use crate::engine::codegen::AutomationCode;
use crate::engine::parts::JackType;
use crate::registry::yaml::YamlNode;
use shared_util::mini_serde::{MiniDes, MiniSer};

#[derive(Clone, Debug)]
pub struct IntControl {
    value: i16,
    range: (i16, i16),
}

impl IntControl {
    pub fn from_yaml(yaml: &YamlNode) -> Result<Self, String> {
        let min = yaml.unique_child("min")?.parse()?;
        let max = yaml.unique_child("max")?.parse_ranged(Some(min), None)?;
        let default = if let Ok(child) = yaml.unique_child("default") {
            let default = child.parse_ranged(Some(min), Some(max))?;
            default
        } else {
            min
        };
        Ok(Self {
            value: default,
            range: (min, max),
        })
    }

    pub fn get_value(&self) -> i16 {
        self.value
    }

    pub fn set_value(&mut self, value: i16) -> UpdateRequest {
        assert!(value >= self.range.0);
        assert!(value <= self.range.1);
        if self.value == value {
            return UpdateRequest::Nothing;
        }
        self.value = value;
        UpdateRequest::UpdateDynData
    }

    pub fn get_range(&self) -> (i16, i16) {
        self.range
    }
}

#[rustfmt::skip] // Keeps trying to ruin my perfectly fine one-line functions.
impl Control for IntControl {
    fn get_parameter_types(&self) -> Vec<IOType> { vec![IOType::Int] }
    fn get_parameter_values(&self) -> Vec<IOData> { unimplemented!() }
    fn generate_code(&self, params: &[&str], automation_code: &AutomationCode) -> String { 
        unimplemented!() 
    }
    fn serialize(&self, ser: &mut MiniSer) { ser.i16(self.value); }
    fn deserialize(&mut self, des: &mut MiniDes) -> Result<(), ()> { 
        self.value = des.i16()?;
        if self.value < self.range.0 || self.value > self.range.1 {
            Err(())
        } else {
            Ok(())
        }
    }
}