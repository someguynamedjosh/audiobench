use super::DataType;
use crate::high_level::problem::FilePosition;
use crate::resolved::structure::ScopeId;

use std::fmt::{self, Debug, Formatter};

#[derive(Clone, Debug)]
pub struct MacroData {
    body: ScopeId,
    header: FilePosition,
}

impl PartialEq for MacroData {
    fn eq(&self, other: &Self) -> bool {
        self.body == other.body
    }
}

impl MacroData {
    pub fn new(body: ScopeId, header: FilePosition) -> MacroData {
        MacroData { body, header }
    }

    pub fn set_header(&mut self, new_header: FilePosition) {
        self.header = new_header;
    }

    pub fn get_header(&self) -> &FilePosition {
        &self.header
    }

    pub fn get_body(&self) -> ScopeId {
        self.body
    }
}

#[derive(Clone, PartialEq)]
pub enum KnownData {
    Bool(bool),
    Int(i64),
    Float(f64),
    Array(Vec<KnownData>),
}

impl KnownData {
    pub fn new_array(size: usize, fill_with: KnownData) -> KnownData {
        KnownData::Array(vec![fill_with; size])
    }

    pub fn collect(items: Vec<KnownData>) -> KnownData {
        debug_assert!(items.len() > 0);
        debug_assert!({
            let dtype = items[0].get_data_type();
            let mut valid = true;
            for item in &items {
                if item.get_data_type() != dtype {
                    valid = false;
                    break;
                }
            }
            valid
        });
        KnownData::Array(items)
    }

    pub fn get_data_type(&self) -> DataType {
        match self {
            KnownData::Array(data) => {
                DataType::Array(data.len(), Box::new(data[0].get_data_type()))
            }
            KnownData::Bool(..) => DataType::Bool,
            KnownData::Int(..) => DataType::Int,
            KnownData::Float(..) => DataType::Float,
        }
    }

    pub fn require_bool(&self) -> bool {
        match self {
            KnownData::Bool(value) => *value,
            _ => panic!("Expected data to be a bool."),
        }
    }

    pub fn require_int(&self) -> i64 {
        match self {
            KnownData::Int(value) => *value,
            _ => panic!("Expected data to be an int."),
        }
    }

    pub fn require_float(&self) -> f64 {
        match self {
            KnownData::Float(value) => *value,
            _ => panic!("Expected data to be a float."),
        }
    }

    pub fn require_array(&self) -> &Vec<KnownData> {
        match self {
            KnownData::Array(value) => value,
            _ => panic!("Expected data to be an array."),
        }
    }

    pub fn require_array_mut(&mut self) -> &mut Vec<KnownData> {
        match self {
            KnownData::Array(value) => value,
            _ => panic!("Expected data to be an array."),
        }
    }

    pub fn matches_data_type(&self, data_type: &DataType) -> bool {
        match self {
            KnownData::Array(contents) => {
                assert!(contents.len() > 0);
                if let DataType::Array(len, etype) = data_type {
                    if contents.len() == *len {
                        return contents[0].matches_data_type(etype);
                    }
                }
                false
            }
            KnownData::Bool(..) => data_type == &DataType::Bool,
            KnownData::Int(..) => data_type == &DataType::Int,
            KnownData::Float(..) => data_type == &DataType::Float,
        }
    }
}

impl Debug for KnownData {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            KnownData::Bool(value) => {
                write!(formatter, "{}", if *value { "true" } else { "false" })
            }
            KnownData::Int(value) => write!(formatter, "{}", value),
            KnownData::Float(value) => write!(formatter, "{}", value),
            KnownData::Array(values) => {
                write!(formatter, "[")?;
                if values.len() > 0 {
                    for value in &values[..values.len() - 1] {
                        write!(formatter, "{:?}, ", value)?;
                    }
                    write!(formatter, "{:?}", values[values.len() - 1])?;
                }
                write!(formatter, "]")
            }
        }
    }
}
