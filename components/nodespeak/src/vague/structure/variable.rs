use super::{DataType, KnownData, MacroData};
use crate::high_level::problem::FilePosition;

use std::fmt::{self, Debug, Formatter};

#[derive(Clone)]
pub struct Variable {
    definition: FilePosition,
    initial_value: Option<KnownData>,
    read_only: bool,
}

impl Debug for Variable {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "\ninitial value: {:?}", self.initial_value)
    }
}

impl Variable {
    fn new_impl(
        definition: FilePosition,
        initial_value: Option<KnownData>,
        read_only: bool,
    ) -> Variable {
        Variable {
            definition,
            initial_value: initial_value,
            read_only,
        }
    }

    pub fn variable(definition: FilePosition, initial_value: Option<KnownData>) -> Variable {
        Self::new_impl(definition, initial_value, false)
    }

    pub fn constant(definition: FilePosition, value: KnownData) -> Variable {
        Self::new_impl(definition, Option::Some(value), true)
    }

    pub fn macro_def(macro_data: MacroData) -> Variable {
        Self::constant(
            macro_data.get_header().clone(),
            KnownData::Macro(macro_data),
        )
    }

    pub fn data_type(definition: FilePosition, value: DataType) -> Variable {
        Self::constant(definition, KnownData::DataType(value))
    }

    pub fn automatic(definition: FilePosition) -> Variable {
        Variable::variable(definition, Option::None)
    }

    pub fn void(definition: FilePosition) -> Variable {
        Variable::variable(definition, Option::None)
    }

    pub fn set_definition(&mut self, new_definition: FilePosition) {
        self.definition = new_definition;
    }

    pub fn get_definition(&self) -> &FilePosition {
        &self.definition
    }

    pub fn set_initial_value(&mut self, value: Option<KnownData>) {
        self.initial_value = value;
    }

    pub fn borrow_initial_value(&self) -> &Option<KnownData> {
        &self.initial_value
    }

    pub fn is_read_only(&self) -> bool {
        self.read_only
    }
}
