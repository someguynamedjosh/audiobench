use std::fmt::{self, Debug, Formatter};

#[derive(Clone, PartialEq)]
pub enum DataType {
    Bool,
    Int,
    Float,
    Array(usize, Box<DataType>),
}

impl Debug for DataType {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            DataType::Bool => write!(formatter, "BOOL"),
            DataType::Int => write!(formatter, "INT"),
            DataType::Float => write!(formatter, "FLOAT"),
            DataType::Array(length, element_type) => {
                write!(formatter, "[{}]{:?}", length, element_type)
            }
        }
    }
}
