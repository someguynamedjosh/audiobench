use crate::{
    engine::{
        codegen::AutomationCode,
        controls::{
            DurationControl, FloatInRangeControl, FrequencyControl, InputControl, IntControl,
            OptionChoiceControl, TimingModeControl, TriggerSequenceControl, ValueSequenceControl,
        },
        data_transfer::{IOData, IOType},
        parts::{JackType, Module},
    },
    registry::yaml::YamlNode,
};
use paste::paste;
use shared_util::{
    mini_serde::{MiniDes, MiniSer},
    prelude::*,
};
use std::fmt::Debug;

#[derive(Debug)]
pub enum UpdateRequest {
    /// For when a particular change does not require any action to be expressed.
    Nothing,
    /// For when a particular change can be expressed by updating global_dyn_data
    UpdateDynData,
    /// For when a particular change requires the entire code to be reocmpiled to be expressed.
    UpdateCode,
}

#[derive(Clone, Debug)]
pub struct AutomationSource {
    pub module: Rcrc<Module>,
    pub output_index: usize,
    pub output_type: JackType,
}

impl AutomationSource {
    pub fn get_type(&self) -> JackType {
        self.output_type
    }
}

pub trait Control: Debug {
    /// Returns a vector of output types that this control accepts automation wires from. Default
    /// implementation returns an empty vector.
    fn acceptable_automation(&self) -> Vec<JackType> {
        vec![]
    }

    /// Called when the user connects some automation of a type given by acceptable_automation.
    fn connect_automation(&mut self, from: AutomationSource) {
        if self.acceptable_automation().len() == 0 {
            panic!("connect_automation called on control that does not accept automation.");
        } else {
            panic!(
                "Control says it accepts automation but has not implemented connect_automation."
            );
        }
    }

    /// Called to retrieve a list of automation sources that should be serialized for this control.
    fn get_connected_automation<'a>(&'a self) -> Vec<&'a AutomationSource> {
        Vec::new()
    }

    fn remove_automation_by_index(&mut self, index: usize) {
        if self.get_connected_automation().len() == 0 {
            panic!("There is no automation to remove.");
        } else {
            panic!("This type is missing an implementation of remove_automation_by_index.");
        }
    }

    /// Returns a list of parameter types that should be transferred to the code for this control.
    fn get_parameter_types(&self) -> Vec<IOType>;

    /// Returns the value for each parameter as defined by get_parameter_types.
    fn get_parameter_values(&self) -> Vec<IOData>;

    /// Returns code that provides the current value of this control. The provided string array
    /// contains code which provides the value of each parameter as defined in get_parameter_types.
    /// automation_code.of(automation_source) can be used to get the value of a particular
    /// automation source.
    fn generate_code(&self, params: &[&str], automation_code: &AutomationCode) -> String;

    fn serialize(&self, ser: &mut MiniSer);

    fn deserialize(&mut self, des: &mut MiniDes) -> Result<(), ()>;
}

macro_rules! any_control_enum {
    ($($control_types:ident),* $(,)?) => {
        paste! {
            #[derive(Debug)]
            pub enum AnyControl {
                $($control_types (Rcrc<[<$control_types Control>]>)),*
            }

            impl AnyControl {
                pub fn as_dyn_ptr(&self) -> Rcrc<dyn Control> {
                    match self {
                        $(Self::$control_types(ptr) => Rc::clone(ptr) as _),*
                    }
                }

                pub fn deep_clone(&self) -> Self {
                    match self {
                        $(Self::$control_types(ptr)
                            => Self::$control_types(rcrc((*ptr.borrow()).clone()))),*
                    }
                }
            }

            pub fn from_yaml(yaml: &YamlNode) -> Result<(String, AnyControl), String> {
                let name = yaml.name.clone();
                let typ = yaml.value.trim();
                let control = match typ {
                    $(stringify!($control_types) => AnyControl::$control_types(rcrc(
                        [<$control_types Control>]::from_yaml(yaml)?
                    ))),*,
                    _ => {
                        return Err(format!(
                            "ERROR: '{}' is an invalid control type (found at {}).",
                            typ, &yaml.full_name
                        ))
                    }
                };
                Ok((name, control))
            }
        }
    }
}

any_control_enum! {
    Input,
    FloatInRange,
    Int,
    Duration,
    TimingMode,
    TriggerSequence,
    ValueSequence,
    OptionChoice,
    Frequency,
}
