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
pub struct OptionChoiceControl {
    options: Vec<String>,
    selected_option: usize,
}

impl OptionChoiceControl {
    pub fn from_yaml(mut yaml: YamlNode) -> Result<Self, String> {
        let mut options = Vec::new();
        for (name, child) in yaml.map_entry("options")?.map_entries()? {
            options.push(name);
        }
        if options.len() < 2 {
            return Err(format!(
                "ERROR: There must be at least 2 options, but only {} were specified.",
                options.len()
            ));
        }
        let default = if let Ok(child) = yaml.map_entry("default") {
            child.parse_ranged(Some(0), Some(options.len() - 1))?
        } else {
            0
        };
        Ok(Self {
            options,
            selected_option: default,
        })
    }

    pub fn get_options(&self) -> &[String] {
        &self.options[..]
    }

    pub fn get_selected_option(&self) -> usize {
        self.selected_option
    }

    pub fn set_selected_option(&mut self, selected_option: usize) -> UpdateRequest {
        assert!(selected_option < self.options.len());
        if self.selected_option == selected_option {
            return UpdateRequest::Nothing;
        }
        self.selected_option = selected_option;
        UpdateRequest::UpdateDynData
    }
}

#[rustfmt::skip] 
impl Control for OptionChoiceControl {
    fn get_parameter_types(&self) -> Vec<IOType> { vec![IOType::Int] }
    fn get_parameter_values(&self) -> Vec<IOData> { vec![IOData::Int(self.selected_option as _)] }
    fn generate_code(&self, params: &[&str], _automation_code: &AutomationCode) -> String { 
        format!("{}", params[0])
    }
    fn serialize(&self, ser: &mut MiniSer) { 
        ser.u8(self.selected_option as _); 
    }
    fn deserialize(&mut self, des: &mut MiniDes) -> Result<(), ()> { 
        self.selected_option = des.u8()? as _;
        if self.selected_option >= self.options.len() {
            Err(())
        } else {
            Ok(())
        }
    }
}
