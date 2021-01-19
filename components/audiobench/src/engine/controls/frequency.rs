use super::{Control, UpdateRequest};
use crate::engine::codegen::AutomationCode;
use crate::engine::data_transfer::{IOData, IOType};
use crate::registry::yaml::YamlNode;
use shared_util::mini_serde::{MiniDes, MiniSer};

#[derive(Clone, Debug)]
pub struct FrequencyControl {
    value: f32,
}

impl FrequencyControl {
    pub const MIN_FREQUENCY: f32 = 0.0003;
    pub const MAX_FREQUENCY: f32 = 99_999.999;

    pub fn from_yaml(yaml: &YamlNode) -> Result<Self, String> {
        let value = if let Ok(child) = yaml.unique_child("default") {
            child.parse_ranged(Some(Self::MIN_FREQUENCY), None)?
        } else {
            1.0
        };
        Ok(Self { value })
    }

    pub fn get_value(&self) -> f32 {
        self.value
    }

    pub fn set_value(&mut self, value: f32) -> UpdateRequest {
        assert!(value >= Self::MIN_FREQUENCY);
        assert!(value <= Self::MAX_FREQUENCY);
        if value == self.value {
            return UpdateRequest::Nothing;
        }
        self.value = value;
        UpdateRequest::UpdateDynData
    }

    pub fn get_formatted_value(&self) -> String {
        let value = self.value;
        let (decimals, kilo) = if value < 10.0 - 0.005 {
            (2, false)
        } else if value < 100.0 - 0.05 {
            (1, false)
        } else if value < 1_000.0 - 0.5 {
            (0, false)
        } else if value < 10_000.0 - 5.0 {
            (1, true)
        } else {
            // if value < 100_000.0
            (0, true)
        };
        if kilo {
            format!("{:.1$}kHz", value / 1000.0, decimals)
        } else {
            format!("{:.1$}Hz", value, decimals)
        }
    }
}

#[rustfmt::skip]
impl Control for FrequencyControl {
    fn get_parameter_types(&self) -> Vec<IOType> { unimplemented!() }
    fn get_parameter_values(&self) -> Vec<IOData> { unimplemented!() }
    fn generate_code(&self, params: &[&str], automation_code: &AutomationCode) -> String { 
        unimplemented!() 
    }
    fn serialize(&self, ser: &mut MiniSer) { 
        ser.f32(self.value);
    }
    fn deserialize(&mut self, des: &mut MiniDes) -> Result<(), ()> { 
        self.value = des.f32()?;
        if self.value >= Self::MIN_FREQUENCY && self.value <= Self::MAX_FREQUENCY {
            Ok(())
        } else {
            Err(())
        }
    }
}
