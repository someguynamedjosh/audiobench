use super::{
    problems, PossiblyKnownData, ResolvedStatement, ResolvedVCExpression, ResolvedVPExpression,
    ScopeResolver,
};
use crate::high_level::problem::{CompileProblem, FilePosition};
use crate::resolved::structure as o;
use crate::vague::structure as i;

impl<'a> ScopeResolver<'a> {
    fn resolve_creation_point(
        &mut self,
        old_var_id: i::VariableId,
        dtype: &i::VPExpression,
        position: &FilePosition,
    ) -> Result<ResolvedStatement, CompileProblem> {
        let resolved_dtype = self.resolve_vp_expression(dtype)?;
        if resolved_dtype.borrow_data_type() != &i::DataType::DataType {
            return Err(problems::not_data_type(
                dtype.clone_position(),
                resolved_dtype.borrow_data_type(),
            ));
        }
        let data_type = if let ResolvedVPExpression::Interpreted(data, ..) = resolved_dtype {
            if let i::KnownData::DataType(in_type) = data {
                in_type
            } else {
                unreachable!("Already checked that the value is a data type.");
            }
        } else {
            unreachable!("DATA_TYPE is ct-only, it cannot be a modified expression.");
        };
        let resolved_id = if let Some(data_type) = Self::resolve_data_type(&data_type) {
            let resolved_var = o::Variable::new(position.clone(), data_type);
            Some(self.target.adopt_variable(resolved_var))
        } else {
            None
        };
        if data_type.is_automatic() {
            self.add_unresolved_auto_var(old_var_id);
        }
        self.set_var_info(old_var_id, resolved_id, data_type);
        if let Some(data) = self.source[old_var_id].borrow_initial_value() {
            let mut pkd = PossiblyKnownData::from_known_data(data);
            if let PossiblyKnownData::Macro(mdata) = &mut pkd {
                mdata.set_context(self.borrow_table().clone());
            }
            self.set_temporary_value(old_var_id, pkd);
        }

        Ok(ResolvedStatement::Interpreted)
    }

    fn resolve_assert(
        &mut self,
        condition: &i::VPExpression,
        position: &FilePosition,
    ) -> Result<ResolvedStatement, CompileProblem> {
        let rcondition = self.resolve_vp_expression(condition)?;
        if rcondition.borrow_data_type() != &i::DataType::Bool {
            return Err(problems::vpe_wrong_type(
                rcondition.clone_position(),
                &i::DataType::Bool,
                rcondition.borrow_data_type(),
            ));
        }
        if let ResolvedVPExpression::Interpreted(data, ..) = &rcondition {
            // Safe because we already checked it's a bool.
            let value = data.require_bool();
            if value {
                Ok(ResolvedStatement::Interpreted)
            } else {
                Err(problems::guaranteed_assert(position.clone()))
            }
        } else {
            Ok(ResolvedStatement::Modified(o::Statement::Assert(
                Box::new(rcondition.as_vp_expression()?),
                position.clone(),
            )))
        }
    }

    pub(super) fn resolve_assign_statement(
        &mut self,
        target: &i::VCExpression,
        value: &i::VPExpression,
        position: &FilePosition,
    ) -> Result<ResolvedStatement, CompileProblem> {
        let lhs = self.resolve_vc_expression(target)?;
        let rhs = self.resolve_vp_expression(value)?;
        let mut resolved_out_type = None;
        if lhs.borrow_data_type().is_automatic() {
            let old_auto_type = &self.get_var_info(lhs.get_base()).unwrap().1;
            let actual_type = old_auto_type.with_different_base(rhs.borrow_data_type().clone());
            let resolved_var = if let Some(rtype) = Self::resolve_data_type(&actual_type) {
                resolved_out_type = Some(rtype.clone());
                let def_pos = self.source[lhs.get_base()].get_definition().clone();
                let var = o::Variable::new(def_pos, rtype);
                Some(self.target.adopt_variable(var))
            } else {
                None
            };
            self.resolve_auto_var(lhs.get_base(), resolved_var, actual_type);
        } else {
            resolved_out_type = Self::resolve_data_type(lhs.borrow_data_type());
            let ok = match Self::biggest_type(lhs.borrow_data_type(), rhs.borrow_data_type()) {
                Ok(bct) => &bct == lhs.borrow_data_type(),
                Err(..) => false,
            };
            if !ok {
                return Err(problems::mismatched_assign(
                    position.clone(),
                    target.clone_position(),
                    lhs.borrow_data_type(),
                    value.clone_position(),
                    rhs.borrow_data_type(),
                ));
            }
        }
        if let (
            ResolvedVCExpression::Specific {
                var,
                indexes,
                typ: lhs_type,
                ..
            },
            ResolvedVPExpression::Interpreted(value_data, _, rhs_type),
        ) = (&lhs, &rhs)
        {
            if lhs_type.is_automatic() {
                self.set_temporary_value(*var, PossiblyKnownData::from_known_data(&value_data));
            } else {
                // lhs[indexes..][shared_indexes..][extra_indexes..] = rhs[shared_indexes..]
                // shared indexes are the indexes that are the same between lhs and rhs
                // extra indexes are extra dimensinos the lhs has. If there are none, then everything
                // becomes much simpler: lhs[indexes..] = rhs, which is what the later if statement
                // is for.
                let lhs_dims = lhs_type.collect_dims();
                let rhs_dims = rhs_type.collect_dims();
                let num_shared_dims = rhs_dims.len();
                let num_extra_dims = lhs_dims.len() - num_shared_dims;
                let extra_dims = Vec::from(&lhs_dims[num_shared_dims..]);

                if extra_dims.len() == 0 {
                    self.set_temporary_item(
                        *var,
                        &indexes[..],
                        PossiblyKnownData::from_known_data(value_data),
                    );
                } else {
                    let mut lhs_indexes = vec![0; indexes.len() + num_shared_dims + num_extra_dims];
                    for (index, value) in indexes.iter().enumerate() {
                        lhs_indexes[index] = *value;
                    }
                    let num_indexes = indexes.len();
                    for shared_indexes in crate::util::nd_index_iter(rhs_dims) {
                        for (offset, value) in shared_indexes.iter().enumerate() {
                            lhs_indexes[num_indexes + offset] = *value;
                        }
                        let rhs_item = value_data.index(&shared_indexes[..]);
                        let rhs_pkd = PossiblyKnownData::from_known_data(rhs_item);
                        for extra_indexes in crate::util::nd_index_iter(extra_dims.clone()) {
                            for (offset, value) in extra_indexes.iter().enumerate() {
                                lhs_indexes[num_indexes + num_shared_dims + offset] = *value;
                            }
                            self.set_temporary_item(*var, &lhs_indexes[..], rhs_pkd.clone());
                        }
                    }
                }
            }
        } else {
            match &lhs {
                ResolvedVCExpression::Specific { var, indexes, .. }
                | ResolvedVCExpression::Modified {
                    base: var, indexes, ..
                } => {
                    self.reset_temporary_range(*var, indexes);
                }
            }
        }
        if resolved_out_type.is_some() {
            // Always return a modified expression. This way, even if we really know what the result
            // of the expression is at compile time, we can still ensure that if something about
            // this does end up being needed specifically at run time then it will be there.
            Ok(ResolvedStatement::Modified(o::Statement::Assign {
                target: Box::new(lhs.as_vc_expression(self)?),
                value: Box::new(rhs.as_vp_expression()?),
                position: position.clone(),
            }))
        } else {
            Ok(ResolvedStatement::Interpreted)
        }
    }

    // Use for clauses that might be executed but we don't know.
    fn resolve_clause_body(&mut self, body: i::ScopeId) -> Result<o::ScopeId, CompileProblem> {
        let scope = self.target.create_scope();
        let old_scope = self.current_scope;
        self.current_scope = scope;
        self.enter_branch_body();

        for statement in self.source[body].borrow_body().clone() {
            let res = self.resolve_statement(&statement)?;
            if let ResolvedStatement::Modified(rstatement) = res {
                self.target[self.current_scope].add_statement(rstatement);
            }
        }

        self.exit_branch_body();
        self.current_scope = old_scope;
        Ok(scope)
    }

    // Use for clauses that we are certain will be executed.
    fn resolve_clause_body_in_place(&mut self, body: i::ScopeId) -> Result<(), CompileProblem> {
        for statement in self.source[body].borrow_body().clone() {
            let res = self.resolve_statement(&statement)?;
            if let ResolvedStatement::Modified(rstatement) = res {
                self.target[self.current_scope].add_statement(rstatement);
            }
        }
        Ok(())
    }

    fn resolve_branch(
        &mut self,
        clauses: &Vec<(i::VPExpression, i::ScopeId)>,
        else_clause: &Option<i::ScopeId>,
        position: &FilePosition,
    ) -> Result<ResolvedStatement, CompileProblem> {
        let mut rclauses = Vec::new();
        for (condition, body) in clauses {
            let rcond = self.resolve_vp_expression(condition)?;
            if rcond.borrow_data_type() != &i::DataType::Bool {
                return Err(problems::vpe_wrong_type(
                    position.clone(),
                    &i::DataType::Bool,
                    rcond.borrow_data_type(),
                ));
            }
            if let ResolvedVPExpression::Interpreted(value, ..) = &rcond {
                // This is safe because we just checked that it is a bool.
                let value = value.require_bool();
                if value && rclauses.len() == 0 {
                    // If this clause is guaranteed to happen and no clauses could have happened
                    // before it, then this is the only clause that will execute. Resolve it in
                    // place and don't bother with the other clauses after it.
                    self.resolve_clause_body_in_place(*body)?;
                    return Ok(ResolvedStatement::Interpreted);
                } else if !value {
                    // If we know the clause won't be executed, just skip it.
                    continue;
                }
            }
            // We get here if we don't know what the condition is, or if we know what the condition
            // is but we can't do any optimizations about it.
            let rbody = self.resolve_clause_body(*body)?;
            rclauses.push((rcond.as_vp_expression()?, rbody));
        }
        let else_clause = if let Some(body) = else_clause {
            if rclauses.len() == 0 {
                // If no clauses could have been executed before the else clause, then we know that
                // the else clause must be executed, so just resolve it in place and return.
                self.resolve_clause_body_in_place(*body)?;
                return Ok(ResolvedStatement::Interpreted);
            } else {
                Some(self.resolve_clause_body(*body)?)
            }
        } else {
            None
        };
        if rclauses.len() == 0 {
            Ok(ResolvedStatement::Interpreted)
        } else {
            Ok(ResolvedStatement::Modified(o::Statement::Branch {
                clauses: rclauses,
                else_clause,
                position: position.clone(),
            }))
        }
    }

    fn resolve_for_loop(
        &mut self,
        allow_unroll: bool,
        counter: i::VariableId,
        start: &i::VPExpression,
        end: &i::VPExpression,
        body: i::ScopeId,
        position: &FilePosition,
    ) -> Result<ResolvedStatement, CompileProblem> {
        // Consider this:
        // Int val = 123; for i = 0 to 10 { other = val; val = val + i; }
        // If we just resolved the scope once, we would write "other = val" because val is known to
        // be 123 at that point. But since it is assigned to later on, we don't actually know that
        // val will be 123. But since it happens after we have already resolved the previous
        // statement, we can't retroactively change it. So instead, we resolve everything in the
        // for loop once, using enter_branch_body() at the start. Once that is complete,
        // exit_branch_body() will mark any variables that could have possibly been assigned as
        // Unknown. We can then go in and resolve the loop body for a second time which will yield
        // the correct code. This second resolving will not have any side effects because any
        // modified known values will be set back to unknown by exit_branch_body(), so known values
        // are either unchanged or invalidated by the first resolving and never actually changed.

        let counter_pos = self.source[counter].get_definition().clone();
        let rcounter = o::Variable::new(counter_pos, o::DataType::Int);
        let rcounter = self.target.adopt_variable(rcounter);
        self.set_var_info(counter, Some(rcounter), i::DataType::Int);
        let body = self.source[body].borrow_body().clone();
        let old_scope = self.current_scope;
        let rstart = self.resolve_vp_expression(start)?;
        let rend = self.resolve_vp_expression(end)?;
        if rstart.borrow_data_type() != &i::DataType::Int {
            return Err(problems::vpe_wrong_type(
                position.clone(),
                &i::DataType::Int,
                rstart.borrow_data_type(),
            ));
        }
        if rend.borrow_data_type() != &i::DataType::Int {
            return Err(problems::vpe_wrong_type(
                position.clone(),
                &i::DataType::Int,
                rend.borrow_data_type(),
            ));
        }
        if let (
            ResolvedVPExpression::Interpreted(start, ..),
            ResolvedVPExpression::Interpreted(end, ..),
            true,
        ) = (&rstart, &rend, allow_unroll)
        {
            // We just checked that they're ints.
            let start = start.require_int();
            let end = end.require_int();
            for i in start..end {
                self.set_temporary_value(counter, PossiblyKnownData::Int(i));
                self.push_table();
                for statement in &body {
                    let res = self.resolve_statement(statement)?;
                    if let ResolvedStatement::Modified(rstatement) = res {
                        self.target[self.current_scope].add_statement(rstatement);
                    }
                }
                self.pop_table();
            }
            return Ok(ResolvedStatement::Interpreted);
        }

        let throwaway_scope = self.target.create_scope();
        self.current_scope = throwaway_scope;
        self.push_table();
        self.enter_branch_body();
        for statement in &body {
            // Don't bother adding it to the scope, it's junk code.
            self.resolve_statement(statement)?;
        }
        self.exit_branch_body();
        self.pop_table();

        let real_scope = self.target.create_scope();
        self.current_scope = real_scope;
        // No enter branch body this time. Everything that gets assigned to with a fixed value will
        // legitimately have that value by the end of the loop. Since the previous section of code
        // just marked everything assigned during the loop as unknown, any known value as a result
        // of this next loop is the product of data that does not depend on the state of the loop.
        for statement in &body {
            let res = self.resolve_statement(statement)?;
            if let ResolvedStatement::Modified(rstatement) = res {
                self.target[self.current_scope].add_statement(rstatement);
            }
        }
        self.current_scope = old_scope;

        Ok(ResolvedStatement::Modified(o::Statement::ForLoop {
            counter: rcounter,
            start: Box::new(rstart.as_vp_expression()?),
            end: Box::new(rend.as_vp_expression()?),
            body: real_scope,
            position: position.clone(),
        }))
    }

    fn resolve_static_init(
        &mut self,
        body: i::ScopeId,
        exports: &Vec<i::VariableId>,
        position: &FilePosition,
    ) -> Result<ResolvedStatement, CompileProblem> {
        let old_scope = self.current_scope;
        self.current_scope = self.target.get_static_init();
        // Changes to the table can be made inside the static block. We don't want that to effect
        // what happens when we return back to the run time scope.
        self.push_table();
        for statement in self.source[body].borrow_body().clone() {
            if let ResolvedStatement::Modified(new) = self.resolve_statement(&statement)? {
                self.target[self.current_scope].add_statement(new);
            }
        }
        let mut exported_var_info = Vec::new();
        for export in exports {
            let info = self.get_var_info(*export);
            if let Some((Some(id), typ)) = info {
                exported_var_info.push((*export, *id, typ.clone()));
                self.target.add_static_var(*id);
            } else {
                panic!("TODO: Nice error, cannot export ct-only variable.");
            }
        }
        self.pop_table();
        self.current_scope = old_scope;
        for (id, rid, typ) in exported_var_info {
            self.set_var_info(id, Some(rid), typ);
            // I don't know if this is necessary but I don't want to solve any bugs involving it
            // not being here.
            self.reset_temporary_value(id);
        }
        Ok(ResolvedStatement::Interpreted)
    }

    fn resolve_raw_vp_expression(
        &mut self,
        expr: &i::VPExpression,
    ) -> Result<ResolvedStatement, CompileProblem> {
        let resolved_expr = self.resolve_vp_expression(expr)?;
        if let ResolvedVPExpression::Interpreted(data, _, dtype) = resolved_expr {
            if data != i::KnownData::Void {
                return Err(problems::dangling_value(expr.clone_position(), &dtype));
            } else {
                Ok(ResolvedStatement::Interpreted)
            }
        } else {
            return Err(problems::dangling_value(
                expr.clone_position(),
                resolved_expr.borrow_data_type(),
            ));
        }
    }

    pub(super) fn resolve_statement(
        &mut self,
        statement: &i::Statement,
    ) -> Result<ResolvedStatement, CompileProblem> {
        match statement {
            i::Statement::CreationPoint {
                var,
                var_type,
                position,
            } => self.resolve_creation_point(*var, var_type, position),
            i::Statement::Assert(value, position) => self.resolve_assert(value, position),
            i::Statement::Return(..) => unimplemented!(),
            i::Statement::Assign {
                target,
                value,
                position,
            } => self.resolve_assign_statement(target, value, position),
            i::Statement::Branch {
                clauses,
                else_clause,
                position,
            } => self.resolve_branch(clauses, else_clause, position),
            i::Statement::ForLoop {
                allow_unroll,
                counter,
                start,
                end,
                body,
                position,
            } => self.resolve_for_loop(*allow_unroll, *counter, start, end, *body, position),
            i::Statement::StaticInit {
                body,
                exports,
                position,
            } => self.resolve_static_init(*body, exports, position),
            i::Statement::RawVPExpression(expr) => self.resolve_raw_vp_expression(expr),
        }
    }
}
