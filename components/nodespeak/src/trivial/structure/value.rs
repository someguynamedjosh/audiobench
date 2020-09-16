use super::{DataType, KnownData, Program, VariableId};
use crate::shared::ProxyMode;

use std::fmt::{self, Debug, Formatter};

#[derive(Clone)]
pub enum ValueBase {
    Literal(KnownData),
    Variable(VariableId),
}

impl Debug for ValueBase {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            Self::Literal(data) => write!(formatter, "{:?}", data),
            Self::Variable(var) => write!(formatter, "{:?}", var),
        }
    }
}

impl ValueBase {
    pub fn get_type(&self, program: &Program) -> DataType {
        match self {
            Self::Literal(data) => data.get_type(),
            Self::Variable(var) => program[*var].borrow_type().clone(),
        }
    }
}

#[derive(Clone)]
pub struct Value {
    pub base: ValueBase,
    pub dimensions: Vec<(usize, ProxyMode)>,
}

impl Value {
    fn new(base: ValueBase, base_type: &DataType) -> Value {
        Value {
            base,
            dimensions: base_type
                .collect_dimensions()
                .iter()
                .map(|len| (*len, ProxyMode::Keep))
                .collect(),
        }
    }

    pub fn variable(variable: VariableId, program: &Program) -> Value {
        Self::new(
            ValueBase::Variable(variable),
            program[variable].borrow_type(),
        )
    }

    pub fn literal(data: KnownData) -> Value {
        let base_type = data.get_type();
        Self::new(ValueBase::Literal(data), &base_type)
    }

    pub fn get_type(&self, program: &Program) -> DataType {
        self.base.get_type(program)
    }

    pub fn inflate(&mut self, dimensions: &[usize]) {
        let mut new_dimensions = Vec::new();
        for (index, target_dimension) in dimensions.iter().cloned().enumerate() {
            if index >= self.dimensions.len() {
                new_dimensions.push((target_dimension, ProxyMode::Discard));
            } else if self.dimensions[index].0 == target_dimension {
                new_dimensions.push(self.dimensions[index].clone());
            } else {
                assert!(
                    self.dimensions[index].0 == 1,
                    "Illegal inflation should have been caught earlier."
                );
                new_dimensions.push((target_dimension, ProxyMode::Collapse));
            }
        }
        self.dimensions = new_dimensions;
    }
}

impl Debug for Value {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        if self.dimensions.len() == 0 {
            write!(formatter, "{:?}", self.base)?;
        } else {
            write!(formatter, "{{")?;
            for (index, (len, mode)) in self.dimensions.iter().enumerate() {
                match mode {
                    ProxyMode::Keep => write!(formatter, "{}", len)?,
                    ProxyMode::Discard => write!(formatter, "{}>X", len)?,
                    ProxyMode::Collapse => write!(formatter, "{}>1", len)?,
                }
                if index < self.dimensions.len() - 1 {
                    write!(formatter, ", ")?;
                }
            }
            write!(formatter, "}}{:?}", self.base)?;
        }
        write!(formatter, "")
    }
}
