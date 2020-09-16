use super::problems;
use crate::ast::structure as i;
use crate::high_level::compiler::{PerformanceCounters, SourceSet};
use crate::high_level::problem::{CompileProblem, FilePosition};
use crate::vague::structure as o;
use std::collections::HashSet;

#[derive(Clone, Default)]
pub(super) struct AuxScopeData {
    pub(super) included_files: HashSet<usize>,
}

pub(super) struct VagueIngester<'a> {
    pub(super) target: o::Program,
    pub(super) current_scope: o::ScopeId,
    pub(super) current_file_id: usize,
    pub(super) source_set: &'a SourceSet,
    pub(super) perf_counters: &'a mut PerformanceCounters,
    pub(super) aux_scope_data: AuxScopeData,
    aux_scope_stack: Vec<AuxScopeData>,
}

impl<'a> VagueIngester<'a> {
    pub(super) fn make_position(&self, node: &i::Node) -> FilePosition {
        FilePosition::from_pair(node, self.current_file_id)
    }

    pub(super) fn lookup_identifier(
        &self,
        node: &i::Node,
    ) -> Result<o::VariableId, CompileProblem> {
        debug_assert!(node.as_rule() == i::Rule::identifier);
        match self.target.lookup_symbol(self.current_scope, node.as_str()) {
            Option::Some(entity) => Result::Ok(entity),
            Option::None => {
                let position = self.make_position(node);
                Result::Err(problems::no_entity_with_name(position))
            }
        }
    }

    pub(super) fn lookup_identifier_without_error(&self, name: &str) -> Option<o::VariableId> {
        self.target.lookup_symbol(self.current_scope, name)
    }

    pub(super) fn add_statement(&mut self, statement: o::Statement) {
        self.target[self.current_scope].add_statement(statement);
    }

    pub(super) fn create_variable_in_scope(
        &mut self,
        scope: o::ScopeId,
        data_type: o::VPExpression,
        name: &str,
        decl_pos: FilePosition,
    ) -> o::VariableId {
        let var = o::Variable::variable(decl_pos.clone(), None);
        let var_id = self.target.adopt_and_define_symbol(scope, name, var);
        self.target[scope].add_statement(o::Statement::CreationPoint {
            var: var_id,
            var_type: Box::new(data_type),
            position: decl_pos,
        });
        var_id
    }

    pub(super) fn create_variable(
        &mut self,
        data_type: o::VPExpression,
        name: &str,
        decl_pos: FilePosition,
    ) -> o::VariableId {
        self.create_variable_in_scope(self.current_scope, data_type, name, decl_pos)
    }

    pub(super) fn enter_scope(&mut self) {
        self.aux_scope_stack.push(self.aux_scope_data.clone());
    }

    pub(super) fn exit_scope(&mut self) {
        self.aux_scope_data = self
            .aux_scope_stack
            .pop()
            .expect("exit_scope called without a matching enter_scope.");
    }

    pub(super) fn execute(&mut self, source: &mut i::Program) -> Result<(), CompileProblem> {
        let root_node = source.next().expect("bad AST");
        debug_assert!(root_node.as_rule() == i::Rule::root);
        for child in root_node.into_inner() {
            if child.as_rule() == i::Rule::EOI {
                break;
            }
            self.convert_statement(child)?;
        }
        Ok(())
    }
}

pub fn ingest(
    source: &mut i::Program,
    source_set: &SourceSet,
    perf_counters: &mut PerformanceCounters,
) -> Result<o::Program, CompileProblem> {
    let target = o::Program::new();
    let init_scope = target.get_entry_point();
    let mut ingester = VagueIngester {
        target,
        current_scope: init_scope,
        current_file_id: 1, // 0 is for fake builtin code.
        source_set,
        perf_counters,
        aux_scope_data: Default::default(),
        aux_scope_stack: Vec::new(),
    };
    ingester.execute(source)?;
    Ok(ingester.target)
}
