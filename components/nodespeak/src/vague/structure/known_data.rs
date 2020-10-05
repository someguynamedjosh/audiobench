use super::{DataType, SpecificDataType};
use crate::high_level::problem::FilePosition;
use crate::vague::structure::ScopeId;
use shared_util::prelude::*;

use std::fmt::{self, Debug, Formatter};

#[derive(Clone, Debug)]
pub struct MacroData {
    body: ScopeId,
    header: FilePosition,
    // Yeah, this is really ugly and hacky. But unfortunately I don't see a better way to allow
    // storing the scope a macro was defined in without creating even more of a headache.
    context: crate::resolved::ResolverTable,
}

impl PartialEq for MacroData {
    fn eq(&self, other: &Self) -> bool {
        self.body == other.body
    }
}

impl MacroData {
    pub fn new(body: ScopeId, header: FilePosition) -> MacroData {
        MacroData {
            body,
            header,
            context: crate::resolved::ResolverTable::new(),
        }
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

    pub(crate) fn set_context(&mut self, context: crate::resolved::ResolverTable) {
        self.context = context;
    }

    pub(crate) fn borrow_context(&self) -> &crate::resolved::ResolverTable {
        &self.context
    }
}

#[derive(Clone, PartialEq)]
pub enum KnownData {
    Void,
    Bool(bool),
    Int(i64),
    Float(f64),
    DataType(DataType),
    Macro(MacroData),
    Array(Vec<KnownData>),
}

impl KnownData {
    pub fn build_array(dims: &[usize]) -> KnownData {
        if dims.len() == 0 {
            KnownData::Void
        } else {
            KnownData::Array(
                (0..dims[0])
                    .map(|_| Self::build_array(&dims[1..]))
                    .collect(),
            )
        }
    }

    pub fn new_array(size: usize) -> KnownData {
        KnownData::Array(vec![KnownData::Void; size])
    }

    pub fn collect(items: Vec<KnownData>) -> KnownData {
        debug_assert!(items.len() > 0);
        debug_assert!(
            {
                let dtype = items[0].get_specific_data_type();
                let mut matches = true;
                for item in &items {
                    if item.get_specific_data_type() != dtype {
                        matches = false;
                        break;
                    }
                }
                matches
            },
            "known data array items are incompatible."
        );
        KnownData::Array(items)
    }

    pub fn get_specific_data_type(&self) -> SpecificDataType {
        match self {
            KnownData::Array(data) => {
                assert!(data.len() > 0);
                let first_type = data[0].get_specific_data_type();
                SpecificDataType::Array(data.len(), Box::new(first_type))
            }
            KnownData::Void => SpecificDataType::Void,
            KnownData::Bool(..) => SpecificDataType::Bool,
            KnownData::Int(..) => SpecificDataType::Int,
            KnownData::Float(..) => SpecificDataType::Float,
            KnownData::DataType(..) => SpecificDataType::DataType,
            KnownData::Macro(..) => SpecificDataType::Macro,
        }
    }

    pub fn index(&self, indexes: &[usize]) -> &KnownData {
        if indexes.len() == 0 {
            self
        } else if let Self::Array(items) = self {
            items[indexes[0]].index(&indexes[1..])
        } else {
            panic!("Cannot index non-array data. (Too many indexes?)")
        }
    }

    /// Panics if inflation is invalid.
    pub fn inflate(&self, dimensions: &[usize]) -> KnownData {
        match self {
            Self::Array(items) => {
                assert!(dimensions.len() > 0);
                if items.len() == dimensions[0] {
                    KnownData::Array(items.imc(|x| x.inflate(&dimensions[1..])))
                } else {
                    assert!(items.len() == 1);
                    let item = items[0].inflate(&dimensions[1..]);
                    KnownData::Array((0..dimensions[0]).map(|_| item.clone()).collect())
                }
            }
            _ => {
                if dimensions.len() == 0 {
                    self.clone()
                } else {
                    let item = self.inflate(&dimensions[1..]);
                    KnownData::Array((0..dimensions[0]).map(|_| item.clone()).collect())
                }
            }
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

    pub fn require_data_type(&self) -> &DataType {
        match self {
            KnownData::DataType(value) => value,
            _ => panic!("Expected data to be a data type."),
        }
    }

    pub fn require_macro(&self) -> &MacroData {
        match self {
            KnownData::Macro(value) => value,
            _ => panic!("Expected data to be a macro."),
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

    pub fn matches_data_type(&self, data_type: &SpecificDataType) -> bool {
        match self {
            KnownData::Array(contents) => {
                if let SpecificDataType::Array(len, etype) = data_type {
                    if len == &contents.len() {
                        assert!(contents.len() > 0);
                        return contents[0].matches_data_type(etype);
                    }
                }
                false
            }
            KnownData::Bool(..) => data_type == &SpecificDataType::Bool,
            KnownData::Int(..) => data_type == &SpecificDataType::Int,
            KnownData::Float(..) => data_type == &SpecificDataType::Float,
            KnownData::Macro(..) => data_type == &SpecificDataType::Macro,
            KnownData::DataType(..) => data_type == &SpecificDataType::DataType,
            KnownData::Void => data_type == &SpecificDataType::Void,
        }
    }
}

impl Debug for KnownData {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            KnownData::Void => write!(formatter, "[void]"),
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
            KnownData::DataType(value) => write!(formatter, "{:?}", value),
            KnownData::Macro(value) => write!(formatter, "macro with body at {:?}", value.body),
        }
    }
}
