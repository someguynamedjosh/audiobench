use crate::resolved::structure::{DataType, Scope, Variable};
use std::fmt::{self, Debug, Formatter};
use std::ops::{Index, IndexMut};

/// Refers to a [`Scope`] stored in a [`Program`].
///
/// You'll notice that this struct requires no lifetime. This was chosen to allow for easy
/// implementation of tree-like and cyclic data resolved::structures inside the library.
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct ScopeId(usize);

impl Debug for ScopeId {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "s{}", self.0)
    }
}

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct VariableId(usize);

impl Debug for VariableId {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "v{}", self.0)
    }
}

/// Represents an entire program written in the Nodespeak language.
pub struct Program {
    scopes: Vec<Scope>,
    static_init: ScopeId,
    entry_point: ScopeId,
    variables: Vec<Variable>,
    static_vars: Vec<VariableId>,
    inputs: Vec<VariableId>,
    outputs: Vec<VariableId>,
}

impl Debug for Program {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "static initialization: {:?}", self.static_init)?;
        for (index, static_var) in self.static_vars.iter().enumerate() {
            write!(formatter, "\nstatic var {}: {:?}", index, static_var)?;
        }
        write!(formatter, "\nentry point: {:?}", self.entry_point)?;
        for (index, input) in self.inputs.iter().enumerate() {
            write!(formatter, "\ninput {}: {:?}", index, input)?;
        }
        for (index, output) in self.outputs.iter().enumerate() {
            write!(formatter, "\noutput {}: {:?}", index, output)?;
        }
        for (index, scope) in self.scopes.iter().enumerate() {
            write!(formatter, "\ncontents of {:?}:\n", ScopeId(index))?;
            write!(
                formatter,
                "    {}",
                format!("{:?}", scope).replace("\n", "\n    ")
            )?;
        }
        for (index, variable) in self.variables.iter().enumerate() {
            write!(formatter, "\ndetails for {:?}:\n", VariableId(index))?;
            write!(
                formatter,
                "    {}",
                format!("{:?}", variable).replace("\n", "\n    ")
            )?;
        }
        write!(formatter, "")
    }
}

impl Index<ScopeId> for Program {
    type Output = Scope;

    fn index(&self, scope: ScopeId) -> &Self::Output {
        &self.scopes[scope.0]
    }
}

impl IndexMut<ScopeId> for Program {
    fn index_mut(&mut self, scope: ScopeId) -> &mut Self::Output {
        &mut self.scopes[scope.0]
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
            scopes: vec![Scope::new(), Scope::new()],
            static_init: ScopeId(0),
            entry_point: ScopeId(1),
            variables: Vec::new(),
            static_vars: Vec::new(),
            inputs: Vec::new(),
            outputs: Vec::new(),
        }
    }

    pub fn create_scope(&mut self) -> ScopeId {
        let id = ScopeId(self.scopes.len());
        self.scopes.push(Scope::new());
        id
    }

    pub fn modify_variable(&mut self, variable: VariableId, modified: Variable) {
        assert!(variable.0 < self.variables.len());
        self.variables[variable.0] = modified;
    }

    pub fn adopt_variable(&mut self, variable: Variable) -> VariableId {
        let id = VariableId(self.variables.len());
        self.variables.push(variable);
        id
    }

    pub fn set_data_type(&mut self, variable: VariableId, data_type: DataType) {
        assert!(variable.0 < self.variables.len());
        self.variables[variable.0].set_data_type(data_type);
    }

    pub fn get_static_init(&self) -> ScopeId {
        self.static_init
    }

    pub fn get_entry_point(&self) -> ScopeId {
        self.entry_point
    }

    pub fn borrow_static_vars(&self) -> &[VariableId] {
        &self.static_vars[..]
    }

    pub fn add_static_var(&mut self, static_var: VariableId) {
        self.static_vars.push(static_var);
    }

    pub fn borrow_inputs(&self) -> &[VariableId] {
        &self.inputs[..]
    }

    pub fn add_input(&mut self, input: VariableId) {
        self.inputs.push(input);
    }

    pub fn borrow_outputs(&self) -> &[VariableId] {
        &self.outputs[..]
    }

    pub fn add_output(&mut self, output: VariableId) {
        self.outputs.push(output);
    }
}
