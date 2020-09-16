use std::fmt::{self, Debug, Formatter};

#[derive(Clone, PartialEq)]
pub enum DataType {
    Automatic,
    Bool,
    Int,
    Float,
    Void,
    DataType,
    Macro,
    Array(usize, Box<DataType>),
}

impl DataType {
    pub fn equivalent(&self, other: &Self) -> bool {
        match self {
            // If it's a basic type, just check if it is equal to the other one.
            Self::Automatic
            | Self::Bool
            | Self::Int
            | Self::Float
            | Self::Void
            | Self::DataType
            | Self::Macro => self == other,
            Self::Array(my_size, my_etype) => {
                if let Self::Array(size, etype) = other {
                    my_size == size && my_etype.equivalent(etype)
                } else {
                    false
                }
            }
        }
    }

    pub fn make_array(dims: &[usize], base: Self) -> Self {
        if dims.len() > 0 {
            Self::Array(dims[0], Box::new(Self::make_array(&dims[1..], base)))
        } else {
            base
        }
    }

    fn collect_dims_impl(&self, dims: &mut Vec<usize>) {
        if let Self::Array(size, btype) = self {
            dims.push(*size);
            btype.collect_dims_impl(dims);
        }
    }

    pub fn collect_dims(&self) -> Vec<usize> {
        let mut dims = Vec::new();
        self.collect_dims_impl(&mut dims);
        dims
    }

    pub fn is_automatic(&self) -> bool {
        match self {
            Self::Automatic => true,
            Self::Array(_, etype) => etype.is_automatic(),
            _ => false,
        }
    }

    pub fn with_different_base(&self, new_base: DataType) -> Self {
        match self {
            Self::Array(size, etyp) => {
                Self::Array(*size, Box::new(etyp.with_different_base(new_base)))
            }
            _ => new_base,
        }
    }
}

impl Debug for DataType {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            Self::Automatic => write!(formatter, "AUTO"),
            Self::Bool => write!(formatter, "BOOL"),
            Self::Int => write!(formatter, "INT"),
            Self::Float => write!(formatter, "FLOAT"),
            Self::Void => write!(formatter, "VOID"),
            Self::DataType => write!(formatter, "DATA_TYPE"),
            Self::Macro => write!(formatter, "MACRO"),
            Self::Array(size, etype) => write!(formatter, "[{}]{:?}", size, etype),
        }
    }
}
