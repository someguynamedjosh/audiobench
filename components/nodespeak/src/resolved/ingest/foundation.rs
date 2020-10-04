use super::{problems, PossiblyKnownData};
use crate::high_level::problem::{CompileProblem, FilePosition};
use crate::resolved::structure as o;
use crate::vague::structure as i;
use std::collections::{HashMap, HashSet};

pub fn ingest(program: &mut i::Program) -> Result<o::Program, CompileProblem> {
    let entry_point = program.get_entry_point();
    let inputs = program[entry_point].borrow_inputs().clone();
    let old_inputs = inputs.clone();
    let outputs = program[entry_point].borrow_outputs().clone();
    let old_outputs = outputs.clone();
    let mut resolver = ScopeResolver::new(program);
    resolver.entry_point(entry_point)?;
    let inputs: Vec<_> = inputs
        .into_iter()
        .map(|id| {
            resolver
                .get_var_info(id)
                .expect("undefined input, should be caught in vague phase.")
                .clone()
        })
        .collect();
    let outputs: Vec<_> = outputs
        .into_iter()
        .map(|id| {
            resolver
                .get_var_info(id)
                .expect("undefined input, should be caught in vague phase.")
                .clone()
        })
        .collect();

    for (index, (id, dtype)) in inputs.into_iter().enumerate() {
        if let Option::Some(var_id) = id {
            resolver.target.add_input(var_id);
        } else {
            let pos = resolver.source[old_inputs[index]].get_definition().clone();
            return Err(problems::compile_time_input(pos, &dtype));
        }
    }
    for (index, (id, dtype)) in outputs.into_iter().enumerate() {
        if let Option::Some(var_id) = id {
            resolver.target.add_output(var_id);
        } else {
            let pos = resolver.source[old_outputs[index]].get_definition().clone();
            return Err(problems::compile_time_output(pos, &dtype));
        }
    }
    Result::Ok(resolver.target)
}

#[derive(Clone, Debug)]
pub(crate) struct ResolverTable {
    variables: HashMap<i::VariableId, (Option<o::VariableId>, i::DataType)>,
}

impl ResolverTable {
    pub(crate) fn new() -> ResolverTable {
        ResolverTable {
            variables: HashMap::new(),
        }
    }
}

pub(super) struct ScopeResolver<'a> {
    pub(super) source: &'a mut i::Program,
    pub(super) target: o::Program,
    pub(super) current_scope: o::ScopeId,
    table: ResolverTable,
    // Even though VariableIds are global (so we don't have to worry about id
    // conflicts), we still have to worry about a single variable having
    // multiple conversions. For example, type parameters can be resolved to
    // different values depending on the types used for the inputs and outputs
    // of the macro.
    stack: Vec<ResolverTable>,
    temp_values: HashMap<i::VariableId, PossiblyKnownData>,
    // Variables that should be marked as unknown once we leave the current branch body because
    // we don't know if the branch body will occur.
    dirty_values: HashSet<i::VariableId>,
    dirty_values_stack: Vec<HashSet<i::VariableId>>,
}

impl<'a> ScopeResolver<'a> {
    fn new(source: &'a mut i::Program) -> ScopeResolver<'a> {
        let target = o::Program::new();
        let entry_point = target.get_entry_point();
        ScopeResolver {
            source,
            target,
            current_scope: entry_point,
            table: ResolverTable::new(),
            stack: Vec::new(),
            temp_values: HashMap::new(),
            dirty_values: HashSet::new(),
            dirty_values_stack: Vec::new(),
        }
    }

    // Pushes the current state of the conversion table onto the stack. The state
    // can be restored with pop_table().
    pub(super) fn push_table(&mut self) {
        self.stack.push(self.table.clone());
    }

    pub(super) fn pop_table(&mut self) {
        self.table = self
            .stack
            .pop()
            .expect("Encountered extra unexpected stack pop");
    }

    pub(super) fn borrow_table(&self) -> &ResolverTable {
        &self.table
    }

    pub(super) fn push_temp_table(&mut self, table: ResolverTable) {
        let old_table = std::mem::replace(&mut self.table, table);
        self.stack.push(old_table);
    }

    // Adds all the entries in the second-to-top table to the top table. This is used by function
    // calls when processing the outputs. This is because they need to access both stuff that only
    // exist in the temporary table and stuff that exists in the rest of the program.
    pub(super) fn fuse_top_table(&mut self) {
        assert!(self.stack.len() > 0);
        let top = self.stack.len() - 1;
        let top_table = &self.stack[top];
        self.table.variables.extend(
            top_table
                .variables
                .iter()
                .map(|(k, v)| (k.clone(), v.clone())),
        );
    }

    /// Any variables that are modified during this period will be marked as dirty. When
    /// leave_branch_body is called, all dirty variables will be marked as unknown because we do not
    /// know at compile time if the branch body will be executed. This also keeps track of
    /// everything in a stack, so it can be called multiple times for nested branch bodies. Each
    /// corresponding call of leave_branch_body will only reset values marked dirty during the
    /// corresponding branch body.
    pub(super) fn enter_branch_body(&mut self) {
        self.dirty_values_stack.push(self.dirty_values.clone());
        self.dirty_values.clear();
    }

    /// Marks dirty values as unknown. See enter_branch_body.
    pub(super) fn exit_branch_body(&mut self) {
        let var_ids = self.dirty_values.iter().cloned().collect::<Vec<_>>();
        for var_id in var_ids.into_iter() {
            // Some of these might be None because they might have been modified inside a function
            // or something like that. We don't have to worry about them because they're gone now
            // so they can't hold a known value anyway.
            if self.get_var_info(var_id).is_some() {
                self.reset_temporary_value(var_id);
            }
        }
        self.dirty_values = self
            .dirty_values_stack
            .pop()
            .expect("Too many calls to exit_branch_body, missing a call to enter_branch_body.");
    }

    pub(super) fn set_var_info(
        &mut self,
        var: i::VariableId,
        resolved_var: Option<o::VariableId>,
        dtype: i::DataType,
    ) {
        assert!(
            !self.table.variables.contains_key(&var),
            "Cannot have multiple sets of info for a single variable."
        );
        let resolved = dtype.actual_type.is_some();
        self.table.variables.insert(var, (resolved_var, dtype));
        if resolved {
            self.reset_temporary_value(var);
        } else {
            self.temp_values.insert(var, PossiblyKnownData::Unknown);
        }
    }

    pub(super) fn get_var_info(
        &self,
        source: i::VariableId,
    ) -> Option<&(Option<o::VariableId>, i::DataType)> {
        self.table.variables.get(&source)
    }

    pub(super) fn resolve_bounded_var(
        &mut self,
        var: i::VariableId,
        resolved_var: Option<o::VariableId>,
        dtype: i::SpecificDataType,
    ) {
        debug_assert!(self.table.variables.contains_key(&var));
        // Go back and resolve the var in any tables in the stack too in case we entered a scope
        // after the variable was first declared.
        for stack_index in (0..self.stack.len()).rev() {
            if !self.stack[stack_index].variables.contains_key(&var) {
                break;
            }
            let entry = self.stack[stack_index].variables.get_mut(&var).unwrap();
            entry.0 = resolved_var;
            entry.1.actual_type = Some(dtype.clone());
        }
        let entry = self.table.variables.get_mut(&var).unwrap();
        entry.0 = resolved_var;
        entry.1.actual_type = Some(dtype);
    }

    pub(super) fn set_temporary_value(&mut self, var: i::VariableId, value: PossiblyKnownData) {
        self.temp_values.insert(var, value);
        self.dirty_values.insert(var);
    }

    pub(super) fn set_temporary_item(
        &mut self,
        var: i::VariableId,
        indexes: &[usize],
        data: PossiblyKnownData,
    ) {
        let value = self.borrow_temporary_value_mut(var);
        let mut item = value;
        for index in indexes {
            if let PossiblyKnownData::Array(items) = item {
                debug_assert!(
                    *index < items.len(),
                    "Bad index, should be handled elsewhere."
                );
                item = &mut items[*index];
            } else {
                unreachable!("Bad index, should be handled elsewhere.");
            }
        }
        *item = data;
    }

    pub(super) fn reset_temporary_value(&mut self, var: i::VariableId) {
        self.dirty_values.insert(var);
        let dims = if let Some((_, typ)) = self.get_var_info(var) {
            debug_assert!(typ.actual_type.is_some());
            typ.actual_type.as_ref().unwrap().collect_dims()
        } else {
            eprintln!("{:?}", var);
            unreachable!("Variable used before declared, should be handled elsewhere.");
        };
        self.temp_values
            .insert(var, PossiblyKnownData::unknown_array(&dims[..]));
    }

    pub(super) fn reset_temporary_range(&mut self, var: i::VariableId, indexes: &[usize]) {
        let dims = if let Some((_, typ)) = self.get_var_info(var) {
            debug_assert!(typ.actual_type.is_some());
            typ.actual_type.as_ref().unwrap().collect_dims()
        } else {
            unreachable!("Variable used before declared, should be handled elsewhere.");
        };
        let range = self.borrow_temporary_item_mut(var, indexes);
        *range = PossiblyKnownData::unknown_array(&dims[indexes.len()..]);
    }

    pub(super) fn borrow_temporary_value(&mut self, var: i::VariableId) -> &PossiblyKnownData {
        if !self.temp_values.contains_key(&var) {
            self.reset_temporary_value(var)
        }
        self.temp_values.get(&var).unwrap() // We just populated the value.
    }

    pub(super) fn borrow_temporary_value_mut(
        &mut self,
        var: i::VariableId,
    ) -> &mut PossiblyKnownData {
        self.dirty_values.insert(var);
        if !self.temp_values.contains_key(&var) {
            self.reset_temporary_value(var)
        }
        self.temp_values.get_mut(&var).unwrap() // We just populated the value.
    }

    pub(super) fn borrow_temporary_item(
        &mut self,
        var: i::VariableId,
        indexes: &[usize],
    ) -> &PossiblyKnownData {
        self.borrow_temporary_item_mut(var, indexes)
    }

    pub(super) fn borrow_temporary_item_mut(
        &mut self,
        var: i::VariableId,
        indexes: &[usize],
    ) -> &mut PossiblyKnownData {
        let mut item = self.borrow_temporary_value_mut(var);
        for index in indexes {
            if let PossiblyKnownData::Array(items) = item {
                debug_assert!(
                    *index < items.len(),
                    "Bad index, should have been handled earlier."
                );
                item = &mut items[*index];
            } else {
                unreachable!("Bad index, should have been handled earlier.");
            }
        }
        item
    }

    pub(super) fn int_literal(value: i64, position: FilePosition) -> o::VPExpression {
        o::VPExpression::Literal(o::KnownData::Int(value), position)
    }

    fn entry_point(&mut self, root_scope: i::ScopeId) -> Result<o::ScopeId, CompileProblem> {
        let builtin_scope = self.source.get_builtins_scope();
        for statement in self.source[builtin_scope].borrow_body().clone() {
            if let ResolvedStatement::Modified(..) = self.resolve_statement(&statement)? {
                unreachable!("Builtin code should not have any part evaluated at run time.");
            }
        }
        self.current_scope = self.target.get_entry_point();
        let old_body = self.source[root_scope].borrow_body().clone();
        for statement in old_body {
            if let ResolvedStatement::Modified(new) = self.resolve_statement(&statement)? {
                self.target[self.current_scope].add_statement(new);
            }
        }
        Ok(self.current_scope)
    }
}

#[derive(Clone, Debug)]
pub(super) enum ResolvedVPExpression {
    /// A simpler or resolved version of the expression was found.
    Modified(o::VPExpression, i::DataType),
    /// The entire value of the expression has a determinate value.
    Interpreted(i::KnownData, FilePosition, i::DataType),
}

impl ResolvedVPExpression {
    pub(super) fn borrow_data_type(&self) -> &i::DataType {
        match self {
            Self::Modified(_, dtype) => dtype,
            Self::Interpreted(_, _, dtype) => dtype,
        }
    }

    pub(super) fn borrow_actual_data_type(&self) -> &i::SpecificDataType {
        // As soon as we have a resolved expression we should know the specific data type it is.
        self.borrow_data_type()
            .actual_type
            .as_ref()
            .expect("Resolved expression does not have a known data type")
    }

    pub(super) fn clone_position(&self) -> FilePosition {
        match self {
            Self::Modified(expr, _) => expr.clone_position(),
            Self::Interpreted(_, pos, _) => pos.clone(),
        }
    }

    /// If the result is already an expression, returns that. If the result is interpreted, returns
    /// a literal expression containing the interpreted value.
    pub(super) fn as_vp_expression(self) -> Result<o::VPExpression, CompileProblem> {
        match self {
            Self::Modified(expr, ..) => Ok(expr),
            Self::Interpreted(data, pos, dtype) => {
                if let Ok(rdata) = ScopeResolver::resolve_known_data(&data) {
                    Ok(o::VPExpression::Literal(rdata, pos))
                } else {
                    Err(problems::value_not_run_time_compatible(pos, &dtype))
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
pub(super) enum ResolvedVCExpression {
    /// We are not sure what variable / element the VCE is targeting.
    Modified {
        vce: o::VCExpression,
        typ: i::DataType,
        // These are used to set unknown values for anything this expression might be targeting.
        // The indexes array contains any indexes that are known at compile time.
        base: i::VariableId,
        indexes: Vec<usize>,
    },
    /// We know exactly what variable / element the VCE is targeting.
    Specific {
        var: i::VariableId,
        indexes: Vec<usize>,
        pos: FilePosition,
        typ: i::DataType,
    },
}

impl ResolvedVCExpression {
    pub(super) fn borrow_data_type(&self) -> &i::DataType {
        match self {
            Self::Modified { typ, .. } => typ,
            Self::Specific { typ, .. } => typ,
        }
    }

    pub(super) fn borrow_actual_data_type(&self) -> Option<&i::SpecificDataType> {
        self.borrow_data_type().actual_type.as_ref()
    }

    pub(super) fn clone_position(&self) -> FilePosition {
        match self {
            Self::Modified { vce, .. } => vce.position.clone(),
            Self::Specific { pos, .. } => pos.clone(),
        }
    }

    /// If the result is already an expression, returns that. If the result is specific, returns
    /// a VCE referencing the specific variable.
    pub(super) fn as_vc_expression(
        self,
        resolver: &ScopeResolver,
    ) -> Result<o::VCExpression, CompileProblem> {
        match self {
            Self::Modified { vce, .. } => Ok(vce),
            Self::Specific {
                var, indexes, pos, ..
            } => {
                let (var_id, _var_type) = resolver.get_var_info(var).expect(
                    "Variable used before defined, should have been caught in vague phase.",
                );
                let var_id = var_id.expect(
                    "Cannot assign to a ct-only var at runtime. Should be checked by the caller.",
                );
                let indexes = indexes
                    .iter()
                    .map(|i| {
                        o::VPExpression::Literal(
                            o::KnownData::Int(*i as i64),
                            FilePosition::placeholder(),
                        )
                    })
                    .collect();
                Ok(o::VCExpression::index(var_id, indexes, pos))
            }
        }
    }

    pub(super) fn get_base(&self) -> i::VariableId {
        match self {
            Self::Modified { base, .. } => *base,
            Self::Specific { var, .. } => *var,
        }
    }
}

#[derive(Clone, Debug)]
pub(super) enum ResolvedStatement {
    /// A simpler or resolved version of the statement was found.
    Modified(o::Statement),
    /// The statement had an effect that was entirely predictable at compile time.
    Interpreted,
}
