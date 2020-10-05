use crate::resolved::structure::Statement;
use std::fmt::{self, Debug, Formatter};

pub struct Scope {
    body: Vec<Statement>,
}

impl Debug for Scope {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        for statement in self.body.iter() {
            writeln!(formatter, "{:?}", statement)?;
        }
        write!(formatter, "")
    }
}

impl Scope {
    pub fn new() -> Scope {
        Scope { body: Vec::new() }
    }

    pub fn add_statement(&mut self, statement: Statement) {
        self.body.push(statement)
    }

    pub fn borrow_body(&self) -> &Vec<Statement> {
        &self.body
    }
}
