use crate::resolved::structure as o;
use crate::vague::structure as i;
use std::fmt::{self, Debug, Formatter};

#[derive(Clone, PartialEq)]
pub enum PossiblyKnownData {
    Void,
    Bool(bool),
    Int(i64),
    Float(f64),
    DataType(i::DataType),
    Macro(i::MacroData),
    Array(Vec<PossiblyKnownData>),
    Unknown,
}

impl PossiblyKnownData {
    pub fn unknown_array(dims: &[usize]) -> PossiblyKnownData {
        if dims.len() == 0 {
            PossiblyKnownData::Unknown
        } else {
            PossiblyKnownData::Array(
                (0..dims[0])
                    .map(|_| Self::unknown_array(&dims[1..]))
                    .collect(),
            )
        }
    }

    pub fn collect(items: Vec<PossiblyKnownData>) -> PossiblyKnownData {
        debug_assert!(items.len() > 0);
        debug_assert!(
            {
                let dtype = items[0].get_data_type();
                let mut matches = true;
                for item in &items {
                    if item.get_data_type() != dtype {
                        matches = false;
                        break;
                    }
                }
                matches
            },
            "known data array items are incompatible."
        );
        PossiblyKnownData::Array(items)
    }

    pub fn from_known_data(kd: &i::KnownData) -> Self {
        match kd {
            i::KnownData::Void => Self::Void,
            i::KnownData::Bool(value) => Self::Bool(*value),
            i::KnownData::Int(value) => Self::Int(*value),
            i::KnownData::Float(value) => Self::Float(*value),
            i::KnownData::DataType(value) => Self::DataType(value.clone()),
            i::KnownData::Macro(value) => Self::Macro(value.clone()),
            i::KnownData::Array(items) => {
                Self::Array(items.iter().map(|i| Self::from_known_data(i)).collect())
            }
        }
    }

    pub fn to_known_data(&self) -> Result<i::KnownData, ()> {
        match self {
            Self::Unknown => Err(()),
            Self::Bool(value) => Ok(i::KnownData::Bool(*value)),
            Self::Int(value) => Ok(i::KnownData::Int(*value)),
            Self::Float(value) => Ok(i::KnownData::Float(*value)),
            Self::DataType(value) => Ok(i::KnownData::DataType(value.clone())),
            Self::Macro(value) => Ok(i::KnownData::Macro(value.clone())),
            Self::Void => Ok(i::KnownData::Void),
            Self::Array(items) => {
                let mut kitems = Vec::with_capacity(items.len());
                for item in items {
                    kitems.push(item.to_known_data()?);
                }
                Ok(i::KnownData::Array(kitems))
            }
        }
    }

    pub fn to_resolved_data(&self) -> Result<o::KnownData, ()> {
        match self {
            Self::Bool(value) => Ok(o::KnownData::Bool(*value)),
            Self::Int(value) => Ok(o::KnownData::Int(*value)),
            Self::Float(value) => Ok(o::KnownData::Float(*value)),
            Self::Array(items) => {
                let mut ritems = Vec::with_capacity(items.len());
                for item in items {
                    ritems.push(item.to_resolved_data()?);
                }
                Ok(o::KnownData::Array(ritems))
            }
            _ => Err(()),
        }
    }

    pub fn is_known(&self) -> bool {
        match self {
            Self::Unknown => false,
            Self::Array(items) => {
                for item in items {
                    if !item.is_known() {
                        return false;
                    }
                }
                true
            }
            _ => true,
        }
    }

    pub fn get_data_type(&self) -> Option<i::DataType> {
        Some(match self {
            PossiblyKnownData::Array(data) => {
                assert!(data.len() > 0);
                if !self.is_known() {
                    return None;
                }
                let first_type = data[0].get_data_type().unwrap();
                i::DataType::Array(data.len(), Box::new(first_type))
            }
            PossiblyKnownData::Void => i::DataType::Void,
            PossiblyKnownData::Bool(..) => i::DataType::Bool,
            PossiblyKnownData::Int(..) => i::DataType::Int,
            PossiblyKnownData::Float(..) => i::DataType::Float,
            PossiblyKnownData::DataType(..) => i::DataType::DataType,
            PossiblyKnownData::Macro(..) => i::DataType::Macro,
            PossiblyKnownData::Unknown => {
                return None;
            }
        })
    }

    pub fn require_bool(&self) -> bool {
        match self {
            PossiblyKnownData::Bool(value) => *value,
            _ => panic!("Expected data to be a bool."),
        }
    }

    pub fn require_int(&self) -> i64 {
        match self {
            PossiblyKnownData::Int(value) => *value,
            _ => panic!("Expected data to be an int."),
        }
    }

    pub fn require_float(&self) -> f64 {
        match self {
            PossiblyKnownData::Float(value) => *value,
            _ => panic!("Expected data to be a float."),
        }
    }

    pub fn require_data_type(&self) -> &i::DataType {
        match self {
            PossiblyKnownData::DataType(value) => value,
            _ => panic!("Expected data to be a data type."),
        }
    }

    pub fn require_macro(&self) -> &i::MacroData {
        match self {
            PossiblyKnownData::Macro(value) => value,
            _ => panic!("Expected data to be a macro."),
        }
    }

    pub fn require_array(&self) -> &Vec<PossiblyKnownData> {
        match self {
            PossiblyKnownData::Array(value) => value,
            _ => panic!("Expected data to be an array."),
        }
    }

    pub fn require_array_mut(&mut self) -> &mut Vec<PossiblyKnownData> {
        match self {
            PossiblyKnownData::Array(value) => value,
            _ => panic!("Expected data to be an array."),
        }
    }
}

impl Debug for PossiblyKnownData {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            PossiblyKnownData::Void => write!(formatter, "[void]"),
            PossiblyKnownData::Bool(value) => {
                write!(formatter, "{}", if *value { "true" } else { "false" })
            }
            PossiblyKnownData::Int(value) => write!(formatter, "{}", value),
            PossiblyKnownData::Float(value) => write!(formatter, "{}", value),
            PossiblyKnownData::Array(values) => {
                write!(formatter, "[")?;
                if values.len() > 0 {
                    for value in &values[..values.len() - 1] {
                        write!(formatter, "{:?}, ", value)?;
                    }
                    write!(formatter, "{:?}", values[values.len() - 1])?;
                }
                write!(formatter, "]")
            }
            PossiblyKnownData::DataType(value) => write!(formatter, "{:?}", value),
            PossiblyKnownData::Macro(value) => {
                write!(formatter, "macro with body at {:?}", value.get_body())
            }
            PossiblyKnownData::Unknown => write!(formatter, "Unknown"),
        }
    }
}
