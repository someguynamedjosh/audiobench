use crate::vague::structure::{ScopeId, Statement, VariableId};
use std::collections::HashMap;
use std::fmt::{self, Debug, Formatter};

pub struct Scope {
    symbols: HashMap<String, VariableId>,
    intermediates: Vec<VariableId>,
    body: Vec<Statement>,
    inputs: Vec<VariableId>,
    outputs: Vec<VariableId>,
    parent: Option<ScopeId>,
}

impl Debug for Scope {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self.parent {
            Option::Some(value) => write!(formatter, "parent: {:?}", value)?,
            Option::None => write!(formatter, "parent: None")?,
        }
        write!(formatter, "\ndefines:")?;
        for (key, value) in self.symbols.iter() {
            write!(formatter, "\n    {:?} = {}", value, key)?;
        }
        for (index, value) in self.intermediates.iter().enumerate() {
            write!(formatter, "\n    {:?} = intermediate #{}", value, index + 1)?;
        }
        for (index, value) in self.inputs.iter().enumerate() {
            write!(formatter, "\ninput {}: {:?}", index + 1, value)?;
        }
        for (index, value) in self.outputs.iter().enumerate() {
            write!(formatter, "\noutput {}: {:?}", index + 1, value)?;
        }
        write!(formatter, "\nbody:")?;
        for statement in self.body.iter() {
            write!(formatter, "\n    {:?}", statement)?;
        }
        write!(formatter, "")
    }
}

impl Scope {
    pub fn new() -> Scope {
        Scope {
            symbols: HashMap::new(),
            intermediates: Vec::new(),
            body: Vec::new(),
            inputs: Vec::new(),
            outputs: Vec::new(),
            parent: Option::None,
        }
    }

    pub fn from_parent(parent: ScopeId) -> Scope {
        Scope {
            symbols: HashMap::new(),
            intermediates: Vec::new(),
            body: Vec::new(),
            inputs: Vec::new(),
            outputs: Vec::new(),
            parent: Option::Some(parent),
        }
    }

    pub fn get_parent(&self) -> Option<ScopeId> {
        self.parent.clone()
    }

    pub fn add_statement(&mut self, statement: Statement) {
        self.body.push(statement)
    }

    pub fn define_symbol(&mut self, symbol: &str, definition: VariableId) {
        self.symbols.insert(symbol.to_owned(), definition);
    }

    pub fn define_intermediate(&mut self, definition: VariableId) {
        self.intermediates.push(definition);
    }

    pub fn clear_inputs(&mut self) {
        self.inputs.clear();
    }

    pub fn clear_outputs(&mut self) {
        self.outputs.clear();
    }

    pub fn add_input(&mut self, input: VariableId) {
        self.inputs.push(input);
    }

    pub fn add_output(&mut self, output: VariableId) {
        self.outputs.push(output);
    }

    pub fn borrow_body(&self) -> &Vec<Statement> {
        &self.body
    }

    pub fn borrow_symbols(&self) -> &HashMap<String, VariableId> {
        &self.symbols
    }

    pub fn borrow_intermediates(&self) -> &Vec<VariableId> {
        &self.intermediates
    }

    pub fn get_input(&self, index: usize) -> VariableId {
        self.inputs[index]
    }

    pub fn borrow_inputs(&self) -> &Vec<VariableId> {
        &self.inputs
    }

    pub fn get_output(&self, index: usize) -> VariableId {
        self.outputs[index]
    }

    pub fn borrow_outputs(&self) -> &Vec<VariableId> {
        &self.outputs
    }

    pub fn get_single_output(&self) -> Option<VariableId> {
        if self.outputs.len() == 1 {
            Option::Some(self.outputs[0])
        } else {
            Option::None
        }
    }
}
