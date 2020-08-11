use crate::engine::data_format::{IODataPtr, IOType};
use crate::registry::yaml::YamlNode;
use crate::util::*;
use std::cell::Ref;

pub enum StaticonUpdateRequest {
    /// For when a particular change does not require any action to be expressed.
    Nothing,
    /// For when a particular change can be expressed by updating global_staticon_dyn_data
    UpdateDynData,
    /// For when a particular change requires the entire code to be reocmpiled to be expressed.
    UpdateCode,
}

pub struct StaticonDynCode {
    input_data_type: String,
    input_io_type: IOType,
    code: String,
}

pub trait ControlledData {
    /// Returns true if the control's value must be available at compile time. This will cause the
    /// code to be recompiled every time the user changes the value, so it should be avoided if at
    /// all possible.
    fn is_static_only(&self) -> bool;

    /// Returns the data type this control has in the generated code.
    fn get_data_type(&self) -> String;

    /// Returns the IOType that should be used when dynamically transferring data. This should never
    /// be called if is_static_only is false.
    fn get_io_type(&self) -> IOType;

    /// Returns code that provides the current value of this control without allowing it to change
    /// in real time.
    fn generate_static_code(&self) -> String;

    /// Returns an IODataPtr containing an up-to-date value that can be passed in to the program
    /// dynamically to update the value of the static control. This should never be called if
    /// is_static_only is false.
    fn package_dyn_data(&self) -> IODataPtr;
}

macro_rules! require_static_only_boilerplate {
    ($yaml:ident) => {
        if let Ok(child) = $yaml.unique_child("require_static_only") {
            child.parse_enumerated(&["false", "true"])? == 1
        } else {
            false
        }
    };
}

pub struct ControlledInt {
    require_static_only: bool,
    value: i32,
    range: (i32, i32),
}

impl ControlledInt {
    fn from_yaml(yaml: YamlNode) -> Result<Self, String> {
        let min = yaml.unique_child("min")?.parse()?;
        let max = yaml.unique_child("min")?.parse_ranged(Some(min), None)?;
        let default = if let Ok(child) = yaml.unique_child("default") {
            let default = child.parse_ranged(Some(min), Some(max))?;
            if default < min {
                return Err(format!(
                    "ERROR: The default value '{}' is smaller than the minimum value '{}'.",
                    default, min
                ));
            } else if default > max {
                return Err(format!(
                    "ERROR: The default value '{}' is bigger than the maximum value '{}'.",
                    default, max
                ));
            }
            default
        } else {
            min
        };
        Ok(Self {
            require_static_only: require_static_only_boilerplate!(yaml),
            value: default,
            range: (min, max),
        })
    }
}

#[rustfmt::skip] // Keeps trying to ruin my perfectly fine one-line functions.
impl ControlledData for ControlledInt {
    fn is_static_only(&self) -> bool { self.require_static_only }
    fn get_data_type(&self) -> String { "INT".to_owned() }
    fn get_io_type(&self) -> IOType { IOType::Int }
    fn generate_static_code(&self) -> String { self.value.to_string() }
    fn package_dyn_data(&self) -> IODataPtr { IODataPtr::Int(self.value) }
}

pub struct ControlledDuration {
    require_static_only: bool,
    decimal_value: f32,
    fraction_mode: bool,
    fraction_numerator: u8,
    fraction_denominator: u8,
}

impl ControlledDuration {
    fn from_yaml(yaml: YamlNode) -> Result<Self, String> {
        let fraction_mode = if let Ok(child) = yaml.unique_child("default_mode") {
            child.parse_enumerated(&["decimal", "fraction"])? == 1
        } else {
            false
        };
        let (decimal_value, fraction_numerator, fraction_denominator) =
            if let Ok(child) = yaml.unique_child("default") {
                (child.parse_ranged(Some(0.0), None)?, 1, 1)
            } else {
                (0.1, 1, 4)
            };
        Ok(Self {
            require_static_only: require_static_only_boilerplate!(yaml),
            decimal_value,
            fraction_mode,
            fraction_numerator,
            fraction_denominator,
        })
    }

    fn get_raw_value(&self) -> f32 {
        if self.fraction_mode {
            self.fraction_numerator as f32 / self.fraction_denominator as f32
        } else {
            self.decimal_value
        }
    }
}

#[rustfmt::skip] 
impl ControlledData for ControlledDuration {
    fn is_static_only(&self) -> bool { self.require_static_only }
    fn get_data_type(&self) -> String { "FLOAT".to_owned() }
    fn get_io_type(&self) -> IOType { IOType::Float }
    fn generate_static_code(&self) -> String { format!("{:.05}", self.get_raw_value()) }
    fn package_dyn_data(&self) -> IODataPtr { IODataPtr::Float(self.get_raw_value()) }
}

pub struct ControlledTimingMode {
    require_static_only: bool,
    /// True if time should be measured against how long the song has been running, false if time
    /// should be measured against how long the note has been running.
    use_song_time: bool,
    /// True if time should be measured in seconds, false if time should be measured in beats.
    beat_synchronized: bool,
}

impl ControlledTimingMode {
    fn from_yaml(yaml: YamlNode) -> Result<Self, String> {
        let use_song_time = if let Ok(child) = yaml.unique_child("default_source") {
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
            require_static_only: require_static_only_boilerplate!(yaml),
            use_song_time,
            beat_synchronized,
        })
    }

    fn get_raw_value(&self) -> i32 {
        let source_flag = if self.use_song_time { 0b1 } else { 0b0 };
        let unit_flag = if self.beat_synchronized { 0b10 } else { 0b00 };
        source_flag | unit_flag
    }
}

#[rustfmt::skip] 
impl ControlledData for ControlledTimingMode {
    fn is_static_only(&self) -> bool { self.require_static_only }
    fn get_data_type(&self) -> String { "INT".to_owned() }
    fn get_io_type(&self) -> IOType { IOType::Int }
    fn generate_static_code(&self) -> String { self.get_raw_value().to_string() }
    fn package_dyn_data(&self) -> IODataPtr { IODataPtr::Int(self.get_raw_value()) }
}

pub struct ControlledTriggerSequence {
    require_static_only: bool,
    sequence: Vec<bool>,
}

impl ControlledTriggerSequence {
    fn from_yaml(yaml: YamlNode) -> Result<Self, String> {
        Ok(Self {
            require_static_only: require_static_only_boilerplate!(yaml),
            sequence: Vec::new(),
        })
    }
}

#[rustfmt::skip] 
impl ControlledData for ControlledTriggerSequence {
    fn is_static_only(&self) -> bool { self.require_static_only }
    fn get_data_type(&self) -> String { format!("[{}]BOOL", self.sequence.len()) }
    fn get_io_type(&self) -> IOType { IOType::BoolArray(self.sequence.len()) }
    fn generate_static_code(&self) -> String {
        let mut result = "[".to_owned();
        for value in &self.sequence {
            result.push_str(if *value { "TRUE," } else { "FALSE," });
        }
        result.push_str("]");
        result
    }
    fn package_dyn_data(&self) -> IODataPtr { IODataPtr::BoolArray(&self.sequence[..]) }
}

pub struct ControlledValueSequence {
    require_static_only: bool,
    sequence: Vec<f32>,
}

impl ControlledValueSequence {
    fn from_yaml(yaml: YamlNode) -> Result<Self, String> {
        Ok(Self {
            require_static_only: require_static_only_boilerplate!(yaml),
            sequence: Vec::new(),
        })
    }
}

#[rustfmt::skip] 
impl ControlledData for ControlledValueSequence {
    fn is_static_only(&self) -> bool { self.require_static_only }
    fn get_data_type(&self) -> String { format!("[{}]FLOAT", self.sequence.len()) }
    fn get_io_type(&self) -> IOType { IOType::FloatArray(self.sequence.len()) }
    fn generate_static_code(&self) -> String {
        let mut result = "[".to_owned();
        for value in &self.sequence {
            result.push_str(&format!("{:.02},", value));
        }
        result.push_str("]");
        result
    }
    fn package_dyn_data(&self) -> IODataPtr { IODataPtr::FloatArray(&self.sequence[..]) }
}

pub enum ArbitraryStaticonData {
    Int(Rcrc<ControlledInt>),
    Duration(Rcrc<ControlledDuration>),
    TimingMode(Rcrc<ControlledTimingMode>),
    TriggerSequence(Rcrc<ControlledTriggerSequence>),
    ValueSequence(Rcrc<ControlledValueSequence>),
}

impl ArbitraryStaticonData {
    fn make_dyn_ptr(&self) -> Rcrc<dyn ControlledData> {
        match self {
            Self::Int(ptr) => Rc::clone(ptr) as _,
        }
    }
}

/// Creates a `Staticon` from a yaml definition. Additionally returns an ArbitraryStaticonData which
/// can be used to retrieve a statically-typed `Rcrc` to the underlying data that the `Staticon`
/// controls.
pub fn from_yaml(yaml: YamlNode) -> Result<(Staticon, ArbitraryStaticonData), String> {
    let name = yaml.name.clone();
    let typ = yaml.value.trim();
    let data = match typ {
        "Int" => ArbitraryStaticonData::Int(rcrc(ControlledInt::from_yaml(yaml)?)),
        "Duration" => ArbitraryStaticonData::Duration(rcrc(ControlledDuration::from_yaml(yaml)?)),
        "TimingMode" => {
            ArbitraryStaticonData::TimingMode(rcrc(ControlledTimingMode::from_yaml(yaml)?))
        }
        "TriggerSequence" => ArbitraryStaticonData::TriggerSequence(rcrc(
            ControlledTriggerSequence::from_yaml(yaml)?,
        )),
        "ValueSequence" => {
            ArbitraryStaticonData::ValueSequence(rcrc(ControlledValueSequence::from_yaml(yaml)?))
        }
        _ => return Err(format!("ERROR: {} is an invalid staticon type.", typ)),
    };
    Ok((
        Staticon {
            code_name: name.clone(),
            data: data.make_dyn_ptr(),
        },
        data,
    ))
}

/// Holds on to packaged data returned by `Staticon::package_dyn_data`. Internally that method
/// requires borrowing from a `RefCell` of a private type so this struct holds on to that borrow
/// while not exposing it to the user of the function.
pub struct PackagedData<'a> {
    borrow: std::cell::Ref<'a, dyn ControlledData>,
    pub data_ref: IODataPtr<'a>,
}

/// Static control, I.E. one that cannot be automated.
pub struct Staticon {
    code_name: String,
    data: Rcrc<dyn ControlledData>,
}

impl Staticon {
    pub fn is_static_only(&self) -> bool {
        self.data.borrow().is_static_only()
    }

    pub fn get_io_type(&self) -> IOType {
        self.data.borrow().get_io_type()
    }

    pub fn generate_static_code(&self) -> String {
        format!(
            "{} {} = {};",
            self.data.borrow().get_data_type(),
            self.code_name,
            self.data.borrow().generate_static_code()
        )
    }

    /// The first string is a line that should be added to the inputs. The second string is a line
    /// that should be added to the actual module code where this control is used.
    pub fn generate_dynamic_code(&self, unique_input_name: &str) -> (String, String) {
        assert!(!self.is_static_only());
        let data = self.data.borrow();
        (
            format!("input {} {};", data.get_data_type(), unique_input_name,),
            format!(
                "{} {} = {};",
                data.get_data_type(),
                self.code_name,
                unique_input_name,
            ),
        )
    }

    pub fn borrow_data(&self) -> Ref<dyn ControlledData> {
        self.data.borrow()
    }
}
