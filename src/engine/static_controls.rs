use crate::engine::data_format::{IODataPtr, IOType};
use crate::registry::yaml::YamlNode;
use crate::util::*;
use std::cell::Ref;
use std::fmt::Debug;

pub enum StaticonUpdateRequest {
    /// For when a particular change does not require any action to be expressed.
    Nothing,
    /// For when a particular change can be expressed by updating global_staticon_dyn_data
    UpdateDynData,
    /// For when a particular change requires the entire code to be reocmpiled to be expressed.
    UpdateCode,
}

impl StaticonUpdateRequest {
    /// Returns `UpdateDynData` if `for_data` allows dynamically updating data (I.E.
    /// `is_static_only` returns `false`.) Otherwise, returns `UpdateCode`.
    fn dyn_update_if_allowed(for_data: &impl ControlledData) -> Self {
        if for_data.is_static_only() {
            Self::UpdateCode
        } else {
            Self::UpdateDynData
        }
    }
}

pub struct StaticonDynCode {
    input_data_type: String,
    input_io_type: IOType,
    code: String,
}

pub trait ControlledData: Debug + PtrClonable {
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

/// This is necessary because doing `ControlledData: Clone` makes it impossible to do
/// `Rcrc<dyn ControlledData>` because it must be sized.
trait PtrClonable {
    fn ptr_clone(&self) -> Rcrc<dyn ControlledData>;
}

impl<T: ControlledData + Clone + 'static> PtrClonable for T {
    fn ptr_clone(&self) -> Rcrc<dyn ControlledData> {
        rcrc(self.clone())
    }
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

#[derive(Clone, Debug)]
pub struct ControlledInt {
    require_static_only: bool,
    value: i32,
    range: (i32, i32),
}

impl ControlledInt {
    fn from_yaml(yaml: &YamlNode) -> Result<Self, String> {
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

    pub fn get_value(&self) -> i32 {
        self.value
    }

    pub fn set_value(&mut self, value: i32) -> StaticonUpdateRequest {
        assert!(value >= self.range.0);
        assert!(value <= self.range.1);
        self.value = value;
        StaticonUpdateRequest::dyn_update_if_allowed(self)
    }

    pub fn get_range(&self) -> (i32, i32) {
        self.range
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

#[derive(Clone, Debug)]
pub struct ControlledDuration {
    require_static_only: bool,
    decimal_value: f32,
    fraction_mode: bool,
    fraction_numerator: u8,
    fraction_denominator: u8,
}

impl ControlledDuration {
    fn from_yaml(yaml: &YamlNode) -> Result<Self, String> {
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

    pub fn get_formatted_value(&self) -> String {
        if self.fraction_mode {
            format!("{}/{}", self.fraction_numerator, self.fraction_denominator)
        } else {
            let value = self.decimal_value;
            let decimals = if value < 0.999 {
                3
            } else if value < 9.99 {
                2
            } else if value < 99.9 {
                1
            } else {
                0
            };
            format!("{:.1$}", value, decimals)
        }
    }

    pub fn get_decimal_value(&self) -> f32 {
        self.decimal_value
    }

    pub fn set_decimal_value(&mut self, value: f32) -> StaticonUpdateRequest {
        // This assert does not protect anything but it *is* indicative of a bug.
        debug_assert!(!self.fraction_mode);
        self.decimal_value = value;
        StaticonUpdateRequest::dyn_update_if_allowed(self)
    }

    pub fn get_fractional_value(&self) -> (u8, u8) {
        (self.fraction_numerator, self.fraction_denominator)
    }

    pub fn set_fractional_value(
        &mut self,
        (numerator, denominator): (u8, u8),
    ) -> StaticonUpdateRequest {
        // This assert does not protect anything but it *is* indicative of a bug.
        debug_assert!(self.fraction_mode);
        self.fraction_numerator = numerator;
        self.fraction_denominator = denominator;
        StaticonUpdateRequest::dyn_update_if_allowed(self)
    }

    pub fn is_using_fractional_mode(&self) -> bool {
        self.fraction_mode
    }

    pub fn use_decimal_mode(&mut self) -> StaticonUpdateRequest {
        // This assert does not protect anything but it *is* indicative of a bug.
        debug_assert!(self.fraction_mode);
        self.fraction_mode = false;
        self.decimal_value = self.fraction_numerator as f32 / self.fraction_denominator as f32;
        StaticonUpdateRequest::dyn_update_if_allowed(self)
    }

    pub fn use_fractional_mode(&mut self) -> StaticonUpdateRequest {
        // This assert does not protect anything but it *is* indicative of a bug.
        debug_assert!(!self.fraction_mode);
        self.fraction_mode = true;
        // TODO: Try to convert the decimal value back to fractional?
        self.fraction_numerator = 1;
        self.fraction_denominator = 4;
        StaticonUpdateRequest::dyn_update_if_allowed(self)
    }

    pub fn toggle_mode(&mut self) -> StaticonUpdateRequest {
        if self.fraction_mode {
            self.use_decimal_mode()
        } else {
            self.use_fractional_mode()
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

#[derive(Clone, Debug)]
pub struct ControlledTimingMode {
    require_static_only: bool,
    /// True if time should be measured against how long the song has been running, false if time
    /// should be measured against how long the note has been running.
    use_song_time: bool,
    /// True if time should be measured in seconds, false if time should be measured in beats.
    beat_synchronized: bool,
}

impl ControlledTimingMode {
    fn from_yaml(yaml: &YamlNode) -> Result<Self, String> {
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

    pub fn uses_song_time(&self) -> bool {
        self.use_song_time
    }

    pub fn toggle_source(&mut self) -> StaticonUpdateRequest {
        self.use_song_time = !self.use_song_time;
        StaticonUpdateRequest::dyn_update_if_allowed(self)
    }

    pub fn is_beat_synchronized(&self) -> bool {
        self.beat_synchronized
    }

    pub fn toggle_units(&mut self) -> StaticonUpdateRequest {
        self.beat_synchronized = !self.beat_synchronized;
        StaticonUpdateRequest::dyn_update_if_allowed(self)
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

#[derive(Clone, Debug)]
pub struct ControlledTriggerSequence {
    require_static_only: bool,
    sequence: Vec<bool>,
}

impl ControlledTriggerSequence {
    fn from_yaml(yaml: &YamlNode) -> Result<Self, String> {
        Ok(Self {
            require_static_only: require_static_only_boilerplate!(yaml),
            sequence: vec![true, false, false, false],
        })
    }

    pub fn get_len(&self) -> usize {
        self.sequence.len()
    }

    pub fn set_len(&mut self, len: usize) -> StaticonUpdateRequest {
        self.sequence.resize(len, false);
        // Changing the length changes the data type of the information dynamically passed in for
        // this control, so the code has to be updated.
        StaticonUpdateRequest::UpdateCode
    }

    pub fn get_trigger(&self, index: usize) -> bool {
        assert!(index < self.get_len());
        self.sequence[index]
    }

    pub fn toggle_trigger(&mut self, index: usize) -> StaticonUpdateRequest {
        assert!(index < self.get_len());
        self.sequence[index] = !self.sequence[index];
        StaticonUpdateRequest::dyn_update_if_allowed(self)
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

#[derive(Clone, Debug)]
pub struct ControlledValueSequence {
    require_static_only: bool,
    sequence: Vec<f32>,
}

impl ControlledValueSequence {
    fn from_yaml(yaml: &YamlNode) -> Result<Self, String> {
        Ok(Self {
            require_static_only: require_static_only_boilerplate!(yaml),
            sequence: vec![1.0, -1.0, -1.0, -1.0],
        })
    }

    pub fn get_len(&self) -> usize {
        self.sequence.len()
    }

    pub fn set_len(&mut self, len: usize) -> StaticonUpdateRequest {
        self.sequence.resize(len, -1.0);
        // Changing the length changes the data type of the information dynamically passed in for
        // this control, so the code has to be updated.
        StaticonUpdateRequest::UpdateCode
    }

    pub fn get_value(&self, index: usize) -> f32 {
        assert!(index < self.get_len());
        self.sequence[index]
    }

    pub fn set_value(&mut self, index: usize, value: f32) -> StaticonUpdateRequest {
        assert!(index < self.get_len());
        self.sequence[index] = value;
        StaticonUpdateRequest::dyn_update_if_allowed(self)
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

#[derive(Clone, Debug)]
pub struct ControlledOptionChoice {
    require_static_only: bool,
    options: Vec<String>,
    selected_option: usize,
}

impl ControlledOptionChoice {
    fn from_yaml(yaml: &YamlNode) -> Result<Self, String> {
        let mut options = Vec::new();
        for child in &yaml.unique_child("options")?.children {
            options.push(child.name.clone());
        }
        if options.len() < 2 {
            return Err(format!(
                "ERROR: There must be at least 2 options, but only {} were specified.",
                options.len()
            ));
        }
        let default = if let Ok(child) = yaml.unique_child("default") {
            child.parse_ranged(Some(0), Some(options.len() - 1))?
        } else {
            0
        };
        Ok(Self {
            require_static_only: require_static_only_boilerplate!(yaml),
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

    pub fn set_selected_option(&mut self, selected_option: usize) -> StaticonUpdateRequest {
        assert!(selected_option < self.options.len());
        self.selected_option = selected_option;
        StaticonUpdateRequest::dyn_update_if_allowed(self)
    }
}

#[rustfmt::skip] 
impl ControlledData for ControlledOptionChoice {
    fn is_static_only(&self) -> bool { self.require_static_only }
    fn get_data_type(&self) -> String { "INT".to_owned() }
    fn get_io_type(&self) -> IOType { IOType::Int }
    fn generate_static_code(&self) -> String { self.selected_option.to_string() }
    fn package_dyn_data(&self) -> IODataPtr { IODataPtr::Int(self.selected_option as _) }
}

#[derive(Debug)]
pub enum ArbitraryStaticonData {
    Int(Rcrc<ControlledInt>),
    Duration(Rcrc<ControlledDuration>),
    TimingMode(Rcrc<ControlledTimingMode>),
    TriggerSequence(Rcrc<ControlledTriggerSequence>),
    ValueSequence(Rcrc<ControlledValueSequence>),
    OptionChoice(Rcrc<ControlledOptionChoice>),
}

impl ArbitraryStaticonData {
    fn make_dyn_ptr(&self) -> Rcrc<dyn ControlledData> {
        match self {
            Self::Int(ptr) => Rc::clone(ptr) as _,
            Self::Duration(ptr) => Rc::clone(ptr) as _,
            Self::TimingMode(ptr) => Rc::clone(ptr) as _,
            Self::TriggerSequence(ptr) => Rc::clone(ptr) as _,
            Self::ValueSequence(ptr) => Rc::clone(ptr) as _,
            Self::OptionChoice(ptr) => Rc::clone(ptr) as _,
        }
    }

    fn deep_clone(&self) -> Self {
        match self {
            Self::Int(ptr) => Self::Int(rcrc((*ptr.borrow()).clone())),
            Self::Duration(ptr) => Self::Duration(rcrc((*ptr.borrow()).clone())),
            Self::TimingMode(ptr) => Self::TimingMode(rcrc((*ptr.borrow()).clone())),
            Self::TriggerSequence(ptr) => Self::TriggerSequence(rcrc((*ptr.borrow()).clone())),
            Self::ValueSequence(ptr) => Self::ValueSequence(rcrc((*ptr.borrow()).clone())),
            Self::OptionChoice(ptr) => Self::OptionChoice(rcrc((*ptr.borrow()).clone())),
        }
    }
}

pub fn from_yaml(yaml: &YamlNode) -> Result<Staticon, String> {
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
    Ok(Staticon {
        code_name: name.clone(),
        data: data.make_dyn_ptr(),
        statically_typed_data: data,
    })
}

/// Holds on to packaged data returned by `Staticon::package_dyn_data`. Internally that method
/// requires borrowing from a `RefCell` of a private type so this struct holds on to that borrow
/// while not exposing it to the user of the function.
pub struct PackagedData<'a> {
    borrow: std::cell::Ref<'a, dyn ControlledData>,
    pub data_ref: IODataPtr<'a>,
}

/// Static control, I.E. one that cannot be automated.
#[derive(Debug)]
pub struct Staticon {
    code_name: String,
    data: Rcrc<dyn ControlledData>,
    statically_typed_data: ArbitraryStaticonData,
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

    pub fn borrow_code_name(&self) -> &str {
        &self.code_name
    }

    pub fn borrow_data(&self) -> Ref<dyn ControlledData> {
        self.data.borrow()
    }

    pub fn borrow_statically_typed_data(&self) -> &ArbitraryStaticonData {
        &self.statically_typed_data
    }
}

impl Clone for Staticon {
    fn clone(&self) -> Self {
        Self {
            code_name: self.code_name.clone(),
            data: self.data.borrow().ptr_clone(),
            statically_typed_data: self.statically_typed_data.deep_clone(),
        }
    }
}
