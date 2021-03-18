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

impl Control for FloatInRangeControl {
    fn acceptable_automation(&self) -> Vec<JackType> {
        vec![JackType::Audio]
    }

    fn connect_automation(&mut self, from: AutomationSource) {
        let range = self.range;
        self.automation.push(AutomationLane {
            connection: from,
            range,
        });
    }

    fn get_connected_automation<'a>(&'a self) -> Vec<&'a AutomationSource> {
        self.automation
            .iter()
            .map(|item| &item.connection)
            .collect()
    }

    fn remove_automation_by_index(&mut self, index: usize) {
        self.automation.remove(index);
    }

    fn get_parameter_types(&self) -> Vec<IOType> {
        vec![IOType::FloatArray]
    }

    fn get_parameter_values(&self) -> Vec<IOData> {
        let mut values = vec![self.value];
        for lane in &self.automation {
            // This is the result of simplifying the expression
            // (value + 1) * 0.5 * (max - min) + min
            // so that computing it only requires one multiplication and one addition.
            let a = (lane.range.1 - lane.range.0) * 0.5;
            let b = (lane.range.1 + lane.range.0) * 0.5;
            values.push(a);
            values.push(b);
        }
        vec![IOData::FloatArray(values)]
    }

    fn generate_code(&self, params: &[&str], automation_code: &AutomationCode) -> String {
        if self.automation.len() == 0 {
            format!("StaticControlSignal({}[1])", params[0])
        } else {
            let mut code = String::new();
            let mut index = 2; // Julia indexing starts at 1.
            let mut first = Some(());
            let num_lanes = self.automation.len() as f32;
            code.push_str("(");
            for lane in &self.automation {
                if !first.take().is_some() {
                    code.push_str(" .+ ");
                }
                code.push_str(&format!(
                    "(a2cs({}) .* {}[{}] .+ {}[{}])",
                    automation_code.value_of(&lane.connection),
                    params[0],
                    index,
                    params[0],
                    index + 1,
                ));
                index += 2;
            }
            code.push_str(&format!(") / Float32({})", self.automation.len()));
            code
        }
    }

    fn serialize(&self, ser: &mut MiniSer) {
        if self.automation.len() == 0 {
            ser.f32_in_range(self.value, self.range.0, self.range.1);
        } else {
            for lane in &self.automation {
                ser.f32_in_range(lane.range.0, self.range.0, self.range.1);
                ser.f32_in_range(lane.range.1, self.range.0, self.range.1);
            }
        }
    }

    fn deserialize(&mut self, des: &mut MiniDes) -> Result<(), ()> {
        let (min, max) = self.range;
        if self.automation.len() == 0 {
            self.value = des.f32_in_range(min, max)?;
        } else {
            self.value = self.default;
            for lane in &mut self.automation {
                lane.range.0 = des.f32_in_range(min, max)?;
                lane.range.1 = des.f32_in_range(min, max)?;
            }
        }
        Ok(())
    }
}
