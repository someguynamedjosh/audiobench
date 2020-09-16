use super::DataType;
use crate::high_level::problem::FilePosition;

use std::fmt::{self, Debug, Formatter};

#[derive(Clone)]
pub struct Variable {
    definition: FilePosition,
    data_type: DataType,
}

impl Debug for Variable {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "data type: {:?}", self.data_type)
    }
}

impl Variable {
    pub fn new(definition: FilePosition, data_type: DataType) -> Variable {
        Variable {
            definition,
            data_type,
        }
    }

    pub fn set_definition(&mut self, new_definition: FilePosition) {
        self.definition = new_definition;
    }

    pub fn get_definition(&self) -> &FilePosition {
        &self.definition
    }

    pub fn set_data_type(&mut self, data_type: DataType) {
        self.data_type = data_type;
    }

    pub fn borrow_data_type(&self) -> &DataType {
        &self.data_type
    }

    pub fn borrow_data_type_mut(&mut self) -> &mut DataType {
        &mut self.data_type
    }
}
