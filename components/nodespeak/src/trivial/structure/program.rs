use super::Variable;
use crate::trivial::structure::Instruction;
use std::fmt::{self, Debug, Formatter};
use std::ops::{Index, IndexMut};

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct VariableId(usize);

impl Debug for VariableId {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "tv{}", self.0)
    }
}

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
struct Label {
    occurs_in_static_body: bool,
}

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct LabelId(usize);

impl LabelId {
    pub fn raw(&self) -> usize {
        self.0
    }
}

impl Debug for LabelId {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "l{}", self.0)
    }
}

pub struct Program {
    static_init: Vec<Instruction>,
    instructions: Vec<Instruction>,
    variables: Vec<Variable>,
    static_vars: Vec<VariableId>,
    inputs: Vec<VariableId>,
    outputs: Vec<VariableId>,
    errors: Vec<String>,
    labels: Vec<Label>,
}

impl Debug for Program {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        writeln!(formatter, "variables:")?;
        for (index, variable) in self.variables.iter().enumerate() {
            writeln!(formatter, "  tv{}: {:?}", index, variable)?;
        }
        write!(formatter, "static vars:")?;
        for variable in self.static_vars.iter() {
            write!(formatter, " {:?}", variable)?;
        }
        writeln!(formatter)?;
        write!(formatter, "inputs:")?;
        for variable in self.inputs.iter() {
            write!(formatter, " {:?}", variable)?;
        }
        writeln!(formatter)?;
        write!(formatter, "outputs:")?;
        for variable in self.outputs.iter() {
            write!(formatter, " {:?}", variable)?;
        }
        writeln!(formatter)?;
        writeln!(formatter, "{} labels", self.labels.len())?;
        writeln!(formatter, "error codes:")?;
        for (code, description) in self.errors.iter().enumerate() {
            writeln!(formatter, "  {}: {}", code, description)?;
        }
        writeln!(formatter, "static init:")?;
        for instruction in self.static_init.iter() {
            writeln!(formatter, "  {:?}", instruction)?;
        }
        writeln!(formatter, "instructions:")?;
        for instruction in self.instructions.iter() {
            writeln!(formatter, "  {:?}", instruction)?;
        }
        write!(formatter, "")
    }
}

impl Index<VariableId> for Program {
    type Output = Variable;

    fn index(&self, variable: VariableId) -> &Self::Output {
        &self.variables[variable.0]
    }
}

impl IndexMut<VariableId> for Program {
    fn index_mut(&mut self, variable: VariableId) -> &mut Self::Output {
        &mut self.variables[variable.0]
    }
}

impl Program {
    pub fn new() -> Program {
        Program {
            instructions: Vec::new(),
            static_init: Vec::new(),
            variables: Vec::new(),
            static_vars: Vec::new(),
            inputs: Vec::new(),
            outputs: Vec::new(),
            errors: vec!["Success".to_owned()],
            labels: Vec::new(),
        }
    }

    pub fn add_instruction(&mut self, instruction: Instruction) {
        self.instructions.push(instruction);
    }

    pub fn add_static_init_instruction(&mut self, instruction: Instruction) {
        self.static_init.push(instruction);
    }

    pub fn borrow_instructions(&self) -> &Vec<Instruction> {
        &self.instructions
    }

    pub fn borrow_static_init_instructions(&self) -> &Vec<Instruction> {
        &self.static_init
    }

    pub fn adopt_variable(&mut self, variable: Variable) -> VariableId {
        let id = VariableId(self.variables.len());
        self.variables.push(variable);
        id
    }

    pub fn borrow_variable(&self, id: VariableId) -> &Variable {
        &self.variables[id.0]
    }

    pub fn borrow_variable_mut(&mut self, id: VariableId) -> &mut Variable {
        &mut self.variables[id.0]
    }

    pub fn iterate_all_variables(&self) -> impl Iterator<Item = VariableId> {
        (0..self.variables.len()).map(|i| VariableId(i))
    }

    pub fn create_label(&mut self, occurs_in_static_body: bool) -> LabelId {
        let id = LabelId(self.labels.len());
        self.labels.push(Label {
            occurs_in_static_body,
        });
        id
    }

    pub fn is_label_in_static_body(&self, label: LabelId) -> bool {
        assert!(label.0 < self.labels.len());
        self.labels[label.0].occurs_in_static_body
    }

    pub fn iterate_all_labels(&self) -> impl Iterator<Item = LabelId> {
        (0..self.labels.len()).map(|i| LabelId(i))
    }

    pub fn add_error(&mut self, description: String) -> u32 {
        let code = self.errors.len() as u32;
        self.errors.push(description);
        code
    }

    pub fn borrow_error_descriptions(&self) -> &Vec<String> {
        &self.errors
    }
}
