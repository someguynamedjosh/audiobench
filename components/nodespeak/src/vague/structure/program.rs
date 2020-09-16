use crate::high_level::problem::FilePosition;
use crate::vague::structure::{self, Scope, Variable};
use std::fmt::{self, Debug, Formatter};
use std::ops::{Index, IndexMut};

/// Refers to a [`Scope`] stored in a [`Program`].
///
/// You'll notice that this struct requires no lifetime. This was chosen to allow for easy
/// implementation of tree-like and cyclic data vague::structures inside the library.
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
    builtins_scope: ScopeId,
    entry_point: ScopeId,
    variables: Vec<Variable>,
}

impl Debug for Program {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "builtins defined at: {:?}", self.builtins_scope)?;
        write!(formatter, "entry point: {:?}", self.entry_point)?;
        for (index, scope) in self.scopes.iter().enumerate() {
            write!(formatter, "\ncontents of {:?}:\n", ScopeId(index))?;
            write!(
                formatter,
                "    {}",
                format!("{:?}", scope).replace("\n", "\n    ")
            )?;
        }
        writeln!(formatter, "\n")?;
        for (index, variable) in self.variables.iter().enumerate() {
            writeln!(
                formatter,
                "initial value of {:?}: {:?}",
                VariableId(index),
                variable.borrow_initial_value()
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
        let mut prog = Program {
            scopes: vec![Scope::new(), Scope::from_parent(ScopeId(0))],
            builtins_scope: ScopeId(0),
            entry_point: ScopeId(1),
            variables: Vec::new(),
        };
        structure::add_builtins(&mut prog);
        prog
    }

    pub fn create_scope(&mut self) -> ScopeId {
        let id = ScopeId(self.scopes.len());
        self.scopes.push(Scope::from_parent(self.builtins_scope));
        id
    }

    pub fn create_child_scope(&mut self, parent: ScopeId) -> ScopeId {
        assert!(parent.0 < self.scopes.len());
        let id = ScopeId(self.scopes.len());
        self.scopes.push(Scope::from_parent(parent));
        id
    }

    pub fn borrow_all_scopes(&self) -> &Vec<Scope> {
        &self.scopes
    }

    // ===SYMBOLS/VARIABLES=========================================================================
    pub fn lookup_symbol(&self, scope: ScopeId, symbol: &str) -> Option<VariableId> {
        match self[scope].borrow_symbols().get(symbol) {
            Option::Some(value_) => Option::Some(*value_),
            Option::None => match self[scope].get_parent() {
                Option::Some(parent) => self.lookup_symbol(parent, symbol),
                Option::None => Option::None,
            },
        }
    }

    pub fn adopt_variable(&mut self, variable: Variable) -> VariableId {
        let id = VariableId(self.variables.len());
        self.variables.push(variable);
        id
    }

    pub fn get_entry_point(&self) -> ScopeId {
        self.entry_point
    }

    pub fn set_entry_point(&mut self, new_entry_point: ScopeId) {
        self.entry_point = new_entry_point;
    }

    pub fn get_builtins_scope(&self) -> ScopeId {
        self.builtins_scope
    }

    pub fn adopt_and_define_symbol(
        &mut self,
        scope: ScopeId,
        symbol: &str,
        definition: Variable,
    ) -> VariableId {
        let id = self.adopt_variable(definition);
        self[scope].define_symbol(symbol, id);
        id
    }

    pub fn adopt_and_define_intermediate(
        &mut self,
        scope: ScopeId,
        definition: Variable,
    ) -> VariableId {
        let id = self.adopt_variable(definition);
        self[scope].define_intermediate(id);
        id
    }

    pub fn make_intermediate_auto_var(
        &mut self,
        scope: ScopeId,
        position: FilePosition,
    ) -> VariableId {
        self.adopt_and_define_intermediate(scope, Variable::automatic(position))
    }
}
