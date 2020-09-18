use std::fmt::{self, Debug, Formatter};

#[derive(Clone, PartialEq)]
pub enum DataType {
    B1,
    I32,
    F32,
    Array(usize, Box<DataType>),
}

impl DataType {
    fn collect_dimensions_impl(&self, dims: &mut Vec<usize>) {
        match self {
            Self::Array(len, etype) => {
                dims.push(*len);
                etype.collect_dimensions_impl(dims);
            }
            _ => (),
        }
    }

    pub fn collect_dimensions(&self) -> Vec<usize> {
        let mut result = Vec::new();
        self.collect_dimensions_impl(&mut result);
        result
    }

    pub fn with_different_base(&self, new_base: DataType) -> Self {
        match self {
            Self::Array(size, etyp) => {
                Self::Array(*size, Box::new(etyp.with_different_base(new_base)))
            }
            _ => new_base,
        }
    }

    pub fn size(&self) -> usize {
        match self {
            Self::B1 => 1,
            Self::I32 => 4,
            Self::F32 => 4,
            Self::Array(len, etyp) => len * etyp.size(),
        }
    }
}

impl Debug for DataType {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            Self::B1 => write!(formatter, "b1"),
            Self::I32 => write!(formatter, "i32"),
            Self::F32 => write!(formatter, "f32"),
            Self::Array(len, base) => write!(formatter, "[{}]{:?}", len, base),
        }
    }
}

#[derive(Clone, Copy)]
pub enum StorageLocation {
    /// Indicates a variable which is part of the input struct which is passed to the main body.
    Input,
    /// Indicates a variable which is part of the output struct which is passed to the main body.
    Output,
    /// Indicates a variable which is part of the static data struct which is passed to the static
    /// body and main body and is intended to be instantiated and modified exclusively by the
    /// program across successive calls of the main body.
    Static,
    /// Indicates a variable which only exists inside the static initialization function.
    StaticBody,
    /// Indicates a variable which only exists inside the body of the main function.
    MainBody,
}

impl Debug for StorageLocation {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(
            formatter,
            "{}",
            match self {
                Self::Input => "in the input data struct",
                Self::Output => "in the output data struct",
                Self::Static => "in the static data struct",
                Self::StaticBody => "in the body of the static init function",
                Self::MainBody => "in the body of the main function",
            }
        )
    }
}

#[derive(Clone)]
pub struct Variable {
    typ: DataType,
    loc: StorageLocation,
}

impl Debug for Variable {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "{:?} {:?}", self.typ, self.loc)
    }
}

impl Variable {
    pub fn new(typ: DataType, loc: StorageLocation) -> Variable {
        Variable { typ, loc }
    }

    pub fn borrow_type(&self) -> &DataType {
        &self.typ
    }

    pub fn get_location(&self) -> StorageLocation {
        self.loc
    }
}
