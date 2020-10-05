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
