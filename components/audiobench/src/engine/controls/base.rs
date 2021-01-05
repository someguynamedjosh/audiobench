use crate::registry::mini_bin;
use crate::engine::codegen::AutomationCode;
use crate::registry::yaml::YamlNode;
use crate::engine::parts::{JackType, Module};
use shared_util::prelude::*;
use std::cell::{Ref, RefMut};
use std::fmt::Debug;
use super::{InputControl, FloatInRangeControl};

use paste::paste;

/// Represents the data type of a variable which is either an input or output in the generated
/// program. E.G. `IOType::FloatArray(20)` would be the type of `input [20]FLOAT some_data;`.
#[derive(Eq, PartialEq, Clone, Copy)]
pub enum IOType {
    Bool,
    Int,
    Float,
    BoolArray(usize),
    IntArray(usize),
    FloatArray(usize),
}

impl IOType {
    /// Returns an IOType which represents the type `[dimension]self`. For example, if dimension is
    /// 5 and `self` is `BoolArray(20)`, `wrap()` will return `BoolArray(100)`.
    pub(crate) fn wrap(self, dimension: usize) -> Self {
        match self {
            Self::Bool => Self::BoolArray(dimension),
            Self::Int => Self::IntArray(dimension),
            Self::Float => Self::FloatArray(dimension),
            Self::BoolArray(size) => Self::BoolArray(dimension * size),
            Self::IntArray(size) => Self::IntArray(dimension * size),
            Self::FloatArray(size) => Self::FloatArray(dimension * size),
        }
    }

    pub(crate) fn get_packed_size(&self) -> usize {
        match self {
            Self::Bool => 1,
            Self::Int => 4,
            Self::Float => 4,
            Self::BoolArray(size) => *size,
            Self::IntArray(size) => 4 * *size,
            Self::FloatArray(size) => 4 * *size,
        }
    }
}

/// Represents data that can be passed to the program or received from the program.
#[derive(PartialEq, Debug)]
pub enum IODataPtr<'a> {
    Bool(bool),
    Int(i32),
    Float(f32),
    BoolArray(&'a [bool]),
    IntArray(&'a [i32]),
    FloatArray(&'a [f32]),
}

impl From<bool> for IODataPtr<'static> {
    fn from(data: bool) -> Self {
        Self::Bool(data)
    }
}

impl From<i32> for IODataPtr<'static> {
    fn from(data: i32) -> Self {
        Self::Int(data)
    }
}

impl From<f32> for IODataPtr<'static> {
    fn from(data: f32) -> Self {
        Self::Float(data)
    }
}

impl<'a> From<&'a [bool]> for IODataPtr<'a> {
    fn from(data: &'a [bool]) -> Self {
        Self::BoolArray(data)
    }
}

impl<'a> From<&'a [i32]> for IODataPtr<'a> {
    fn from(data: &'a [i32]) -> Self {
        Self::IntArray(data)
    }
}

impl<'a> From<&'a [f32]> for IODataPtr<'a> {
    fn from(data: &'a [f32]) -> Self {
        Self::FloatArray(data)
    }
}

impl<'a> From<&'a IOData> for IODataPtr<'a> {
    fn from(data: &'a IOData) -> Self {
        data.borrow()
    }
}

impl<'a> IODataPtr<'a> {
    pub fn get_data_type(&self) -> IOType {
        match self {
            Self::Bool(..) => IOType::Bool,
            Self::Int(..) => IOType::Int,
            Self::Float(..) => IOType::Float,
            Self::BoolArray(arr) => IOType::BoolArray(arr.len()),
            Self::IntArray(arr) => IOType::IntArray(arr.len()),
            Self::FloatArray(arr) => IOType::FloatArray(arr.len()),
        }
    }

    pub fn to_owned(&self) -> IOData {
        match self {
            Self::Bool(value) => IOData::Bool(*value),
            Self::Int(value) => IOData::Int(*value),
            Self::Float(value) => IOData::Float(*value),
            Self::BoolArray(slice_ptr) => {
                IOData::BoolArray(Vec::from(*slice_ptr).into_boxed_slice())
            }
            Self::IntArray(slice_ptr) => {
                IOData::IntArray(Vec::from(*slice_ptr).into_boxed_slice())
            }
            Self::FloatArray(slice_ptr) => {
                IOData::FloatArray(Vec::from(*slice_ptr).into_boxed_slice())
            }
        }
    }

    pub fn unwrap_bool(self) -> bool {
        if let Self::Bool(value) = self {
            value
        } else {
            panic!("Tried to call unwrap_bool on {:?}", self)
        }
    }

    pub fn unwrap_int(self) -> i32 {
        if let Self::Int(value) = self {
            value
        } else {
            panic!("Tried to call unwrap_int on {:?}", self)
        }
    }

    pub fn unwrap_float(self) -> f32 {
        if let Self::Float(value) = self {
            value
        } else {
            panic!("Tried to call unwrap_float on {:?}", self)
        }
    }

    pub fn unwrap_bool_array(self) -> &'a [bool] {
        if let Self::BoolArray(value) = self {
            value
        } else {
            panic!("Tried to call unwrap_bool_array on {:?}", self)
        }
    }

    pub fn unwrap_int_array(self) -> &'a [i32] {
        if let Self::IntArray(value) = self {
            value
        } else {
            panic!("Tried to call unwrap_int_array on {:?}", self)
        }
    }

    pub fn unwrap_float_array(self) -> &'a [f32] {
        if let Self::FloatArray(value) = self {
            value
        } else {
            panic!("Tried to call unwrap_float_array on {:?}", self)
        }
    }

    fn write_packed_data(&self, target: &mut [u8]) {
        assert!(self.get_data_type().get_packed_size() == target.len());
        match self {
            Self::Bool(value) => target[0] = if *value { 1 } else { 0 },
            Self::Int(value) => {
                let bytes = value.to_ne_bytes();
                for i in 0..4 {
                    target[i] = bytes[i];
                }
            }
            Self::Float(value) => {
                let bytes = value.to_ne_bytes();
                for i in 0..4 {
                    target[i] = bytes[i];
                }
            }
            Self::BoolArray(arr) => {
                for (index, value) in arr.iter().enumerate() {
                    target[index] = if *value { 1 } else { 0 };
                }
            }
            Self::IntArray(arr) => {
                for (index, value) in arr.iter().enumerate() {
                    let bytes = value.to_ne_bytes();
                    for i in 0..4 {
                        target[index * 4 + i] = bytes[i];
                    }
                }
            }
            Self::FloatArray(arr) => {
                for (index, value) in arr.iter().enumerate() {
                    let bytes = value.to_ne_bytes();
                    for i in 0..4 {
                        target[index * 4 + i] = bytes[i];
                    }
                }
            }
        }
    }
}

#[derive(PartialEq, Debug)]
pub enum IOData {
    Bool(bool),
    Int(i32),
    Float(f32),
    BoolArray(Box<[bool]>),
    IntArray(Box<[i32]>),
    FloatArray(Box<[f32]>),
}

impl IOData {
    pub fn borrow(&self) -> IODataPtr {
        match self {
            Self::Bool(value) => IODataPtr::Bool(*value),
            Self::Int(value) => IODataPtr::Int(*value),
            Self::Float(value) => IODataPtr::Float(*value),
            Self::BoolArray(value) => IODataPtr::BoolArray(&value[..]),
            Self::IntArray(value) => IODataPtr::IntArray(&value[..]),
            Self::FloatArray(value) => IODataPtr::FloatArray(&value[..]),
        }
    }
}

#[derive(Debug)]
pub enum UpdateRequest {
    /// For when a particular change does not require any action to be expressed.
    Nothing,
    /// For when a particular change can be expressed by updating global_dyn_data
    UpdateDynData,
    /// For when a particular change requires the entire code to be reocmpiled to be expressed.
    UpdateCode,
}

impl UpdateRequest {
    /// Returns `UpdateDynData` if `for_data` allows dynamically updating data (I.E.
    /// `is_static_only` returns `false`.) Otherwise, returns `UpdateCode`.
    fn dyn_update_if_allowed(for_data: &impl Control) -> Self {
        if for_data.is_static_only() {
            Self::UpdateCode
        } else {
            Self::UpdateDynData
        }
    }
}

#[derive(Clone, Debug)]
pub struct AutomationSource {
    module: Rcrc<Module>,
    output_index: usize,
    output_type: JackType,
}

impl AutomationSource {
    pub fn get_type(&self) -> JackType {
        self.output_type
    }
}

pub trait Control: Debug {
    /// Returns true if the control's value must be available at compile time. This will cause the
    /// code to be recompiled every time the user changes the value, so it should be avoided if at
    /// all possible.
    fn is_static_only(&self) -> bool;

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

    /// Returns a list of parameter types that should be transferred to the code for this control.
    fn get_parameter_types(&self) -> Vec<IOType>;

    /// Returns the value for each parameter as defined by get_parameter_types.
    fn get_parameter_values(&self) -> Vec<IOData>;

    /// Returns code that provides the current value of this control. The provided string array
    /// contains code which provides the value of each parameter as defined in get_parameter_types.
    /// automation_code.of(automation_source) can be used to get the value of a particular
    /// automation source.
    fn generate_code(&self, params: &[&str], automation_code: &AutomationCode) -> String;

    fn serialize(&self, buffer: &mut Vec<u8>);

    fn deserialize(&mut self, data: &mut &[u8]) -> Result<(), ()>;
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
pub struct IntControl {
    require_static_only: bool,
    value: i16,
    range: (i16, i16),
}

impl IntControl {
    fn from_yaml(yaml: &YamlNode) -> Result<Self, String> {
        let min = yaml.unique_child("min")?.parse()?;
        let max = yaml.unique_child("max")?.parse_ranged(Some(min), None)?;
        let default = if let Ok(child) = yaml.unique_child("default") {
            let default = child.parse_ranged(Some(min), Some(max))?;
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
        UpdateRequest::dyn_update_if_allowed(self)
    }

    pub fn get_range(&self) -> (i16, i16) {
        self.range
    }
}

#[rustfmt::skip] // Keeps trying to ruin my perfectly fine one-line functions.
impl Control for IntControl {
    fn is_static_only(&self) -> bool { self.require_static_only }
    fn get_parameter_types(&self) -> Vec<IOType> { vec![IOType::Int] }
    fn get_parameter_values(&self) -> Vec<IOData> { unimplemented!() }
    fn generate_code(&self, params: &[&str], automation_code: &AutomationCode) -> String { 
        unimplemented!() 
    }
    fn serialize(&self, buffer: &mut Vec<u8>) { mini_bin::ser_i16(buffer, self.value); }
    fn deserialize(&mut self, slice: &mut &[u8]) -> Result<(), ()> { 
        self.value = mini_bin::des_i16(slice)?;
        if self.value < self.range.0 || self.value > self.range.1 {
            Err(())
        } else {
            Ok(())
        }
    }
}

#[derive(Clone, Debug)]
pub struct DurationControl {
    require_static_only: bool,
    decimal_value: f32,
    fraction_mode: bool,
    fraction_numerator: u8,
    fraction_denominator: u8,
}

impl DurationControl {
    fn from_yaml(yaml: &YamlNode) -> Result<Self, String> {
        let fraction_mode = if let Ok(child) = yaml.unique_child("default_format") {
            child.parse_enumerated(&["decimal", "fractional"])? == 1
        } else {
            false
        };
        // Oof ouch owie my indentation
        let (decimal_value, fraction_numerator, fraction_denominator) =
            if let Ok(child) = yaml.unique_child("default") {
                if fraction_mode {
                    let (num, den) = child.parse_custom(|value| {
                        let slash_index = value.find("/").ok_or_else(|| {
                            format!("ERROR: Not a valid fraction, missing a slash.")
                        })?;
                        let tex_num = &value[..slash_index].trim();
                        let tex_den = &value[slash_index + 1..].trim();
                        let num = tex_num
                            .parse()
                            .map_err(|_| format!("ERROR: The numerator is not valid."))?;
                        let den = tex_den
                            .parse()
                            .map_err(|_| format!("ERROR: The numerator is not valid."))?;
                        Ok((num, den))
                    })?;
                    (1.0, num, den)
                } else {
                    (child.parse_ranged(Some(0.0), None)?, 1, 1)
                }
            } else {
                (1.0, 1, 1)
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

    pub fn set_decimal_value(&mut self, value: f32) -> UpdateRequest {
        // This assert does not protect anything but it *is* indicative of a bug.
        debug_assert!(!self.fraction_mode);
        if self.decimal_value == value {
            return UpdateRequest::Nothing;
        }
        self.decimal_value = value;
        UpdateRequest::dyn_update_if_allowed(self)
    }

    pub fn get_fractional_value(&self) -> (u8, u8) {
        (self.fraction_numerator, self.fraction_denominator)
    }

    pub fn set_fractional_value(&mut self, (numerator, denominator): (u8, u8)) -> UpdateRequest {
        // This assert does not protect anything but it *is* indicative of a bug.
        debug_assert!(self.fraction_mode);
        if self.fraction_numerator == numerator && self.fraction_denominator == denominator {
            return UpdateRequest::Nothing;
        }
        self.fraction_numerator = numerator;
        self.fraction_denominator = denominator;
        UpdateRequest::dyn_update_if_allowed(self)
    }

    pub fn is_using_fractional_mode(&self) -> bool {
        self.fraction_mode
    }

    pub fn use_decimal_mode(&mut self) -> UpdateRequest {
        // This assert does not protect anything but it *is* indicative of a bug.
        debug_assert!(self.fraction_mode);
        self.fraction_mode = false;
        self.decimal_value = self.fraction_numerator as f32 / self.fraction_denominator as f32;
        UpdateRequest::dyn_update_if_allowed(self)
    }

    pub fn use_fractional_mode(&mut self) -> UpdateRequest {
        // This assert does not protect anything but it *is* indicative of a bug.
        debug_assert!(!self.fraction_mode);
        self.fraction_mode = true;
        // TODO: Try to convert the decimal value back to fractional?
        self.fraction_numerator = 1;
        self.fraction_denominator = 4;
        UpdateRequest::dyn_update_if_allowed(self)
    }

    pub fn toggle_mode(&mut self) -> UpdateRequest {
        if self.fraction_mode {
            self.use_decimal_mode()
        } else {
            self.use_fractional_mode()
        }
    }
}

#[rustfmt::skip] 
impl Control for DurationControl {
    fn is_static_only(&self) -> bool { self.require_static_only }
    fn get_parameter_types(&self) -> Vec<IOType> { unimplemented!() }
    fn get_parameter_values(&self) -> Vec<IOData> { unimplemented!() }
    fn generate_code(&self, params: &[&str], automation_code: &AutomationCode) -> String { 
        unimplemented!() 
    }
    fn serialize(&self, buffer: &mut Vec<u8>) { 
        if self.fraction_mode {
            mini_bin::ser_u8(buffer, 1);
            mini_bin::ser_u8(buffer, self.fraction_numerator);
            mini_bin::ser_u8(buffer, self.fraction_denominator);
        } else {
            mini_bin::ser_u8(buffer, 0);
            mini_bin::ser_f32(buffer, self.decimal_value);
        }
    }
    fn deserialize(&mut self, slice: &mut &[u8]) -> Result<(), ()> { 
        let mode = mini_bin::des_u8(slice)?;
        if mode == 1 {
            self.fraction_mode = true;
            self.fraction_numerator = mini_bin::des_u8(slice)?;
            self.fraction_denominator = mini_bin::des_u8(slice)?;
        } else {
            self.fraction_mode = false;
            self.decimal_value = mini_bin::des_f32(slice)?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct TimingModeControl {
    require_static_only: bool,
    /// True if time should be measured against how long the song has been running, false if time
    /// should be measured against how long the note has been running.
    use_elapsed_time: bool,
    /// True if time should be measured in seconds, false if time should be measured in beats.
    beat_synchronized: bool,
}

impl TimingModeControl {
    fn from_yaml(yaml: &YamlNode) -> Result<Self, String> {
        let use_elapsed_time = if let Ok(child) = yaml.unique_child("default_source") {
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
            use_elapsed_time,
            beat_synchronized,
        })
    }

    fn get_raw_value(&self) -> u8 {
        let source_flag = if self.use_elapsed_time { 0b1 } else { 0b0 };
        let unit_flag = if self.beat_synchronized { 0b10 } else { 0b00 };
        source_flag | unit_flag
    }

    pub fn uses_elapsed_time(&self) -> bool {
        self.use_elapsed_time
    }

    pub fn toggle_source(&mut self) -> UpdateRequest {
        self.use_elapsed_time = !self.use_elapsed_time;
        UpdateRequest::dyn_update_if_allowed(self)
    }

    pub fn is_beat_synchronized(&self) -> bool {
        self.beat_synchronized
    }

    pub fn toggle_units(&mut self) -> UpdateRequest {
        self.beat_synchronized = !self.beat_synchronized;
        UpdateRequest::dyn_update_if_allowed(self)
    }
}

#[rustfmt::skip] 
impl Control for TimingModeControl {
    fn is_static_only(&self) -> bool { self.require_static_only }
    fn get_parameter_types(&self) -> Vec<IOType> { unimplemented!() }
    fn get_parameter_values(&self) -> Vec<IOData> { unimplemented!() }
    fn generate_code(&self, params: &[&str], automation_code: &AutomationCode) -> String { 
        unimplemented!() 
    }
    fn serialize(&self, buffer: &mut Vec<u8>) { mini_bin::ser_u8(buffer, self.get_raw_value()); }
    fn deserialize(&mut self, slice: &mut &[u8]) -> Result<(), ()> { 
        let raw_value = mini_bin::des_u8(slice)?;
        if raw_value > 0b11 {
            return Err(());
        }
        self.use_elapsed_time = raw_value & 0b1 == 0b1;
        self.beat_synchronized = raw_value & 0b10 == 0b10;
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct TriggerSequenceControl {
    require_static_only: bool,
    sequence: Vec<bool>,
}

impl TriggerSequenceControl {
    fn from_yaml(yaml: &YamlNode) -> Result<Self, String> {
        Ok(Self {
            require_static_only: require_static_only_boilerplate!(yaml),
            sequence: vec![true, false, false, false],
        })
    }

    pub fn get_len(&self) -> usize {
        self.sequence.len()
    }

    pub fn set_len(&mut self, len: usize) -> UpdateRequest {
        if self.sequence.len() == len {
            return UpdateRequest::Nothing;
        }
        self.sequence.resize(len, false);
        // Changing the length changes the data type of the information dynamically passed in for
        // this control, so the code has to be updated.
        UpdateRequest::UpdateCode
    }

    pub fn get_trigger(&self, index: usize) -> bool {
        assert!(index < self.get_len());
        self.sequence[index]
    }

    pub fn toggle_trigger(&mut self, index: usize) -> UpdateRequest {
        assert!(index < self.get_len());
        self.sequence[index] = !self.sequence[index];
        UpdateRequest::dyn_update_if_allowed(self)
    }
}

#[rustfmt::skip] 
impl Control for TriggerSequenceControl {
    fn is_static_only(&self) -> bool { self.require_static_only }
    fn get_parameter_types(&self) -> Vec<IOType> { unimplemented!() }
    fn get_parameter_values(&self) -> Vec<IOData> { unimplemented!() }
    fn generate_code(&self, params: &[&str], automation_code: &AutomationCode) -> String { 
        unimplemented!() 
    }
    fn serialize(&self, buffer: &mut Vec<u8>) { 
        assert!(self.sequence.len() <= 0xFF);
        mini_bin::ser_u8(buffer, self.sequence.len() as u8); 
        mini_bin::ser_bool_slice(buffer, &self.sequence[..]); 
    }
    fn deserialize(&mut self, slice: &mut &[u8]) -> Result<(), ()> { 
        let len = mini_bin::des_u8(slice)?;
        self.sequence = mini_bin::des_bool_slice(slice, len as usize)?;
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct ValueSequenceControl {
    require_static_only: bool,
    sequence: Vec<f32>,
}

impl ValueSequenceControl {
    fn from_yaml(yaml: &YamlNode) -> Result<Self, String> {
        Ok(Self {
            require_static_only: require_static_only_boilerplate!(yaml),
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
        UpdateRequest::dyn_update_if_allowed(self)
    }
}

#[rustfmt::skip] 
impl Control for ValueSequenceControl {
    fn is_static_only(&self) -> bool { self.require_static_only }
    fn get_parameter_types(&self) -> Vec<IOType> { unimplemented!() }
    fn get_parameter_values(&self) -> Vec<IOData> { unimplemented!() }
    fn generate_code(&self, params: &[&str], automation_code: &AutomationCode) -> String { 
        unimplemented!() 
    }
    fn serialize(&self, buffer: &mut Vec<u8>) { 
        assert!(self.sequence.len() <= 0xFF);
        mini_bin::ser_u8(buffer, self.sequence.len() as u8); 
        for value in &self.sequence {
            let packed = mini_bin::pack_value(*value, (-1.0, 1.0));
            mini_bin::ser_u16(buffer, packed);
        }
    }
    fn deserialize(&mut self, slice: &mut &[u8]) -> Result<(), ()> { 
        let len = mini_bin::des_u8(slice)?;
        self.sequence.clear();
        for _ in 0..len {
            let packed = mini_bin::des_u16(slice)?;
            self.sequence.push(mini_bin::unpack_value(packed, (-1.0, 1.0)));
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct OptionChoiceControl {
    require_static_only: bool,
    options: Vec<String>,
    selected_option: usize,
}

impl OptionChoiceControl {
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

    pub fn set_selected_option(&mut self, selected_option: usize) -> UpdateRequest {
        assert!(selected_option < self.options.len());
        if self.selected_option == selected_option {
            return UpdateRequest::Nothing;
        }
        self.selected_option = selected_option;
        UpdateRequest::dyn_update_if_allowed(self)
    }
}

#[rustfmt::skip] 
impl Control for OptionChoiceControl {
    fn is_static_only(&self) -> bool { self.require_static_only }
    fn get_parameter_types(&self) -> Vec<IOType> { unimplemented!() }
    fn get_parameter_values(&self) -> Vec<IOData> { unimplemented!() }
    fn generate_code(&self, params: &[&str], automation_code: &AutomationCode) -> String { 
        unimplemented!() 
    }
    fn serialize(&self, buffer: &mut Vec<u8>) { 
        mini_bin::ser_u8(buffer, self.selected_option as _); 
    }
    fn deserialize(&mut self, slice: &mut &[u8]) -> Result<(), ()> { 
        self.selected_option = mini_bin::des_u8(slice)? as _;
        if self.selected_option >= self.options.len() {
            Err(())
        } else {
            Ok(())
        }
    }
}

#[derive(Clone, Debug)]
pub struct FrequencyControl {
    require_static_only: bool,
    value: f32,
}

impl FrequencyControl {
    pub const MIN_FREQUENCY: f32 = 0.0003;
    pub const MAX_FREQUENCY: f32 = 99_999.999;

    fn from_yaml(yaml: &YamlNode) -> Result<Self, String> {
        let value = if let Ok(child) = yaml.unique_child("default") {
            child.parse_ranged(Some(Self::MIN_FREQUENCY), None)?
        } else {
            1.0
        };
        Ok(Self {
            require_static_only: require_static_only_boilerplate!(yaml),
            value,
        })
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
        UpdateRequest::dyn_update_if_allowed(self)
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
    fn is_static_only(&self) -> bool { self.require_static_only }
    fn get_parameter_types(&self) -> Vec<IOType> { unimplemented!() }
    fn get_parameter_values(&self) -> Vec<IOData> { unimplemented!() }
    fn generate_code(&self, params: &[&str], automation_code: &AutomationCode) -> String { 
        unimplemented!() 
    }
    fn serialize(&self, buffer: &mut Vec<u8>) { 
        mini_bin::ser_f32(buffer, self.value);
    }
    fn deserialize(&mut self, slice: &mut &[u8]) -> Result<(), ()> { 
        self.value = mini_bin::des_f32(slice)?;
        if self.value >= Self::MIN_FREQUENCY && self.value <= Self::MAX_FREQUENCY {
            Ok(())
        } else {
            Err(())
        }
    }
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
