use super::{AutomationSource, Control, IOData, IOType};
use crate::engine::parts::JackType;
use crate::registry::yaml::YamlNode;
use crate::engine::codegen::AutomationCode;

#[derive(Clone, Debug)]
pub struct AutomationLane {
    pub range: (f32, f32),
    pub connection: AutomationSource,
}

#[derive(Clone, Debug)]
pub struct FloatInRangeControl {
    pub range: (f32, f32),
    pub value: f32,
    pub default: f32,
    pub automation: Vec<AutomationLane>,
    pub suffix: String,
    // TODO: Automation.
}

impl FloatInRangeControl {
    pub fn from_yaml(yaml: &YamlNode) -> Result<Self, String> {
        let min = yaml.unique_child("min")?.parse()?;
        let max = yaml.unique_child("max")?.parse_ranged(Some(min), None)?;
        let default = if let Ok(child) = yaml.unique_child("default") {
            let default = child.parse_ranged(Some(min), Some(max))?;
            default
        } else {
            min
        };
        let suffix = yaml
            .unique_child("suffix")
            .map(|child| child.value.clone())
            .unwrap_or("".to_owned());
        Ok(Self {
            range: (min, max),
            value: default,
            default,
            automation: Vec::new(),
            suffix,
        })
    }
}

#[rustfmt::skip]
impl Control for FloatInRangeControl {
    fn is_static_only(&self) -> bool { false }
    fn acceptable_automation(&self) -> Vec<JackType> { vec![JackType::Audio] }
    fn connect_automation(&mut self, from: AutomationSource) {
        unimplemented!();
    }
    fn get_parameter_types(&self) -> Vec<IOType> { vec![] }
    fn get_parameter_values(&self) -> Vec<IOData> { vec![] }
    fn generate_code(&self, params: &[&str], automation_code: &AutomationCode) -> String { 
        unimplemented!() 
    }
    fn serialize(&self, buffer: &mut Vec<u8>) { unimplemented!() }
    fn deserialize(&mut self, data: &mut &[u8]) -> Result<(), ()> { unimplemented!() }
}
