use shared_util::mini_serde::{MiniDes, MiniSer};

use super::{AutomationSource, Control, IOData, IOType};
use crate::engine::codegen::AutomationCode;
use crate::engine::parts::JackType;
use crate::registry::yaml::YamlNode;

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
    fn acceptable_automation(&self) -> Vec<JackType> { vec![JackType::Audio] }
    fn connect_automation(&mut self, from: AutomationSource) {
        let range = self.range;
        self.automation.push(AutomationLane {
            connection: from,
            range,
        });
    }
    fn get_connected_automation<'a>(
        &'a self,
    ) -> Box<dyn Iterator<Item = &'a AutomationSource> + 'a> {
        Box::new(self.automation.iter().map(|item| &item.connection))
    }

    fn get_parameter_types(&self) -> Vec<IOType> { 
        let length = 1 + self.automation.len() * 2;
        vec![IOType::FloatArray]
    }
    fn get_parameter_values(&self) -> Vec<IOData> { 
        let mut values = vec![self.value];
        for lane in &self.automation {
            values.push(lane.range.0);
            values.push(lane.range.1);
        }
        vec![IOData::FloatArray(values)] 
    }
    fn generate_code(&self, params: &[&str], automation_code: &AutomationCode) -> String {
        let mut code = params[0].to_owned();
        let mut index = 1;
        for lane in &self.automation {
            code.push_str(&format!(
                " + ({} * {} * {})",
                params[index],
                params[index + 1],
                automation_code.value_of(&lane.connection)
            ));
        }
        code
    }
    fn serialize(&self, ser: &mut MiniSer) { unimplemented!() }
    fn deserialize(&mut self, des: &mut MiniDes) -> Result<(), ()> { unimplemented!() }
}
