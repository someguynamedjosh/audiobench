use super::DataType;
use std::fmt::{self, Debug, Formatter};

#[derive(Clone, PartialEq)]
pub enum KnownData {
    Int(i64),
    Float(f64),
    Bool(bool),
    Array(Vec<KnownData>),
}

impl Debug for KnownData {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            Self::Int(value) => write!(formatter, "{}i32", value),
            Self::Float(value) => write!(formatter, "{}f32", value),
            Self::Bool(value) => write!(formatter, "{}b1", if *value { "true" } else { "false" }),
            Self::Array(values) => {
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

impl KnownData {
    pub fn build_array(dimensions: &[usize], element: KnownData) -> KnownData {
        if dimensions.len() == 0 {
            element
        } else {
            let flat_element = Self::build_array(&dimensions[1..], element);
            KnownData::Array(vec![flat_element; dimensions[0]])
        }
    }

    pub fn get_type(&self) -> DataType {
        match self {
            Self::Int(..) => DataType::I32,
            Self::Float(..) => DataType::F32,
            Self::Bool(..) => DataType::B1,
            Self::Array(data) => {
                assert!(data.len() > 0);
                DataType::Array(data.len(), Box::new(data[0].get_type()))
            }
        }
    }

    pub fn binary_data(&self) -> u32 {
        match self {
            Self::Bool(value) => {
                if *value {
                    1
                } else {
                    0
                }
            }
            Self::Int(value) => *value as i32 as u32,
            Self::Float(value) => f32::to_bits(*value as f32),
            Self::Array(..) => unimplemented!(),
        }
    }

    fn add_binary_data(&self, to: &mut Vec<u8>) {
        match self {
            Self::Bool(value) => to.push(if *value { 1 } else { 0 }),
            Self::Int(value) => {
                for byte in (*value as i32).to_le_bytes().iter() {
                    to.push(*byte);
                }
            }
            Self::Float(value) => {
                for byte in (*value as f32).to_le_bytes().iter() {
                    to.push(*byte);
                }
            }
            Self::Array(values) => {
                for value in values {
                    value.add_binary_data(to);
                }
            }
        }
    }

    pub fn arbitrary_len_binary_data(&self) -> Vec<u8> {
        let mut data = Vec::new();
        self.add_binary_data(&mut data);
        data
    }

    pub fn require_int(&self) -> i64 {
        if let Self::Int(value) = self {
            *value
        } else {
            panic!("Required an Int, but got a {:?}.", self)
        }
    }

    pub fn require_float(&self) -> f64 {
        if let Self::Float(value) = self {
            *value
        } else {
            panic!("Required an Float, but got a {:?}.", self)
        }
    }
}
