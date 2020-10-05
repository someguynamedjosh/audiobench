use super::{problems, PossiblyKnownData, ResolvedStatement, ResolvedVPExpression, ScopeResolver};
use crate::high_level::problem::{CompileProblem, FilePosition};
use crate::resolved::structure as o;
use crate::vague::structure as i;

impl<'a> ScopeResolver<'a> {
    fn resolve_vp_variable(
        &mut self,
        var_id: i::VariableId,
        position: &FilePosition,
    ) -> Result<ResolvedVPExpression, CompileProblem> {
        let (resolved_id, dtype) = self
            .get_var_info(var_id)
            .expect("Variable used before defined, should have been caught by the previous phase.");
        if dtype.actual_type.is_none() {
            return Err(problems::unresolved_bounded_var(position.clone()));
        }
        let (resolved_id, dtype) = (resolved_id.clone(), dtype.clone());
        let value = self.borrow_temporary_value(var_id);
        if let Ok(kvalue) = value.to_known_data() {
            let typ = kvalue.get_specific_data_type();
            return Ok(ResolvedVPExpression::Interpreted(
                kvalue,
                position.clone(),
                dtype,
            ));
        }
        let resolved_id = if let Some(value) = resolved_id {
            value
        } else {
            return Err(problems::value_not_run_time_compatible(
                position.clone(),
                &dtype,
            ));
        };
        Ok(ResolvedVPExpression::Modified(
            o::VPExpression::Variable(resolved_id, position.clone()),
            dtype.clone(),
        ))
    }

    fn resolve_collect(
        &mut self,
        items: &Vec<i::VPExpression>,
        position: &FilePosition,
    ) -> Result<ResolvedVPExpression, CompileProblem> {
        debug_assert!(items.len() > 0);
        let mut resolved_items = Vec::new();
        for item in items {
            resolved_items.push(self.resolve_vp_expression(item)?);
        }
        let typ = resolved_items[0].borrow_data_type();
        let mut all_known = true;
        for item in &resolved_items {
            if item.borrow_data_type() != typ {
                return Err(problems::bad_array_literal(
                    item.clone_position(),
                    item.borrow_data_type(),
                    resolved_items[0].clone_position(),
                    resolved_items[0].borrow_data_type(),
                ));
            }
            if let ResolvedVPExpression::Modified(..) = item {
                all_known = false;
            }
        }

        let atype = i::SpecificDataType::Array(
            resolved_items.len(),
            Box::new(typ.actual_type.as_ref().unwrap().clone()),
        );
        Ok(if all_known {
            let mut data_items = Vec::new();
            for item in resolved_items {
                if let ResolvedVPExpression::Interpreted(data, ..) = item {
                    data_items.push(data);
                } else {
                    unreachable!("Already checked that all items are known.");
                }
            }
            ResolvedVPExpression::Interpreted(
                i::KnownData::Array(data_items),
                position.clone(),
                atype.into(),
            )
        } else {
            let mut vp_exprs = Vec::new();
            for item in resolved_items {
                vp_exprs.push(item.as_vp_expression()?)
            }
            ResolvedVPExpression::Modified(
                o::VPExpression::Collect(vp_exprs, position.clone()),
                atype.into(),
            )
        })
    }

    fn resolve_type_bound(
        &mut self,
        lower: &Option<Box<i::VPExpression>>,
        upper: &Box<i::VPExpression>,
        position: &FilePosition,
    ) -> Result<ResolvedVPExpression, CompileProblem> {
        let lower = if let Some(lower) = lower {
            let result = self.resolve_vp_expression(lower)?;
            if result.borrow_actual_data_type() != &i::SpecificDataType::DataType {
                return Err(problems::not_data_type(
                    result.clone_position(),
                    result.borrow_data_type(),
                ));
            }
            if let ResolvedVPExpression::Interpreted(i::KnownData::DataType(typ), ..) = result {
                // TODO: Error for nested type bound.
                Some(typ.actual_type.unwrap())
            } else {
                // Data types should only exist as ::Interpreted.
                unreachable!("Data type at runtime")
            }
        } else {
            None
        };
        let upper = {
            let result = self.resolve_vp_expression(upper)?;
            if result.borrow_actual_data_type() != &i::SpecificDataType::DataType {
                return Err(problems::not_data_type(
                    result.clone_position(),
                    result.borrow_data_type(),
                ));
            }
            if let ResolvedVPExpression::Interpreted(i::KnownData::DataType(typ), ..) = result {
                // TODO: Error for nested type bound.
                Some(typ.actual_type.unwrap())
            } else {
                // Data types should only exist as ::Interpreted.
                unreachable!("Data type at runtime")
            }
        };
        Ok(ResolvedVPExpression::Interpreted(
            i::KnownData::DataType(i::DataType {
                actual_type: None,
                bounds: i::Bounds::from_tuple((lower, upper)),
            }),
            position.clone(),
            i::SpecificDataType::DataType.into(),
        ))
    }

    fn resolve_build_array_type(
        &mut self,
        dimensions: &Vec<i::VPExpression>,
        base: &i::VPExpression,
        position: &FilePosition,
    ) -> Result<ResolvedVPExpression, CompileProblem> {
        let mut int_dims = Vec::new();
        for dim in dimensions.iter().rev() {
            let resolved = self.resolve_vp_expression(dim)?;
            let rpos = resolved.clone_position();
            if let ResolvedVPExpression::Interpreted(data, ..) = resolved {
                if let i::KnownData::Int(value) = data {
                    if value < 1 {
                        return Err(problems::array_size_less_than_one(rpos, value));
                    }
                    int_dims.push(value as usize);
                } else {
                    return Err(problems::array_size_not_int(
                        rpos,
                        &data.get_specific_data_type(),
                    ));
                }
            } else {
                return Err(problems::array_size_not_resolved(rpos));
            }
        }
        let resolved_base = self.resolve_vp_expression(base)?;
        if resolved_base.borrow_actual_data_type() != &i::SpecificDataType::DataType {
            return Err(problems::array_base_not_data_type(
                resolved_base.clone_position(),
                resolved_base.borrow_data_type(),
            ));
        }
        if let ResolvedVPExpression::Interpreted(data, ..) = resolved_base {
            if let i::KnownData::DataType(dtype, ..) = data {
                let mut final_type = dtype.clone();
                for dim in int_dims {
                    let bounds = final_type.bounds.as_tuple();
                    let actual_type = final_type.actual_type.as_ref();
                    // Add the index to the actual type (if there is one) and each of the bounds
                    // (if they exist.)
                    final_type = i::DataType {
                        bounds: i::Bounds::from_tuple((
                            bounds.0.map(|etype| {
                                i::SpecificDataType::Array(dim, Box::new(etype.clone()))
                            }),
                            bounds.1.map(|etype| {
                                i::SpecificDataType::Array(dim, Box::new(etype.clone()))
                            }),
                        )),
                        actual_type: actual_type
                            .map(|etype| i::SpecificDataType::Array(dim, Box::new(etype.clone()))),
                    };
                }
                Ok(ResolvedVPExpression::Interpreted(
                    i::KnownData::DataType(final_type),
                    position.clone(),
                    i::SpecificDataType::DataType.into(),
                ))
            } else {
                unreachable!("We already checked that the expr was a data type.")
            }
        } else {
            unreachable!("Data is both dynamic and a data type. That should be impossible.");
        }
    }

    fn resolve_vp_index_impl(
        &mut self,
        resolved_base: ResolvedVPExpression,
        index: &i::VPExpression,
        optional: bool,
        full_expression: &FilePosition,
    ) -> Result<ResolvedVPExpression, CompileProblem> {
        let resolved_index = self.resolve_vp_expression(index)?;
        let etype = resolved_base.borrow_data_type().indexed(optional);
        let etype = if let Some(etype) = etype {
            etype
        } else {
            return Err(problems::cannot_index(
                full_expression.clone(),
                index.clone_position(),
                &resolved_base.borrow_data_type().min().unwrap(),
            ));
        };
        let array_length =
            if let i::SpecificDataType::Array(len, ..) = resolved_base.borrow_actual_data_type() {
                *len
            } else {
                if optional {
                    let mut resolved_base = resolved_base;
                    // This should only change the upper type bound. The function will debug_assert
                    // if the actual_type was changed.
                    resolved_base.set_data_type(etype);
                    return Ok(resolved_base);
                } else {
                    unreachable!("Cannot index should have been handled earlier.");
                }
            };
        if resolved_index.borrow_actual_data_type() != &i::SpecificDataType::Int {
            return Err(problems::array_index_not_int(
                index.clone_position(),
                resolved_index.borrow_data_type(),
                full_expression.clone(),
            ));
        }
        Self::value_bound_error_helper(
            index.clone_position(),
            index.clone_position(),
            resolved_index.borrow_data_type(),
            &(i::SpecificDataType::Int.into()),
        )?;

        match resolved_index {
            // If the index is compile-time constant.
            ResolvedVPExpression::Interpreted(data, index_pos, dtype) => {
                // Safe because we already checked that it was an int.
                let value = data.require_int();
                // Check that the index is in bounds.
                if value < 0 {
                    return Err(problems::array_index_less_than_zero(
                        index.clone_position(),
                        value,
                        full_expression.clone(),
                    ));
                }
                if value as usize >= array_length {
                    return Err(problems::array_index_too_big(
                        index.clone_position(),
                        value as usize,
                        array_length,
                        full_expression.clone(),
                    ));
                }
                Ok(match resolved_base {
                    // If the base is also compile-time constant, return a new constant value.
                    ResolvedVPExpression::Interpreted(base_data, base_pos, base_type) => {
                        let element = base_data.require_array()[value as usize].clone();
                        let mut pos = base_pos;
                        pos.include_other(&index_pos);
                        ResolvedVPExpression::Interpreted(element, pos, etype)
                    }
                    ResolvedVPExpression::Modified(base_expr, base_type) => {
                        let pos = FilePosition::union(&[&base_expr.clone_position(), &index_pos]);
                        let index = Self::int_literal(value, index_pos);
                        ResolvedVPExpression::Modified(
                            // Otherwise, if it's an index expression, add the new index to it.
                            if let o::VPExpression::Index {
                                base,
                                mut indexes,
                                position,
                            } = base_expr
                            {
                                indexes.push(index);
                                o::VPExpression::Index {
                                    base,
                                    indexes,
                                    position: pos,
                                }
                            // Otherwise, make an entirely new index expression.
                            } else {
                                o::VPExpression::Index {
                                    base: Box::new(base_expr),
                                    indexes: vec![index],
                                    position: pos,
                                }
                            },
                            etype,
                        )
                    }
                })
            }
            // Otherwise, if the index is only available as a run-time expression...
            ResolvedVPExpression::Modified(index_expr, ..) => {
                let pos = FilePosition::union(&[
                    &resolved_base.clone_position(),
                    &index_expr.clone_position(),
                ]);
                let expr = match resolved_base {
                    // If the base is a compile-time constant, make it a literal and return a new
                    // expression.
                    ResolvedVPExpression::Interpreted(base_data, base_pos, base_type) => {
                        let base_data = if let Ok(value) = Self::resolve_known_data(&base_data) {
                            value
                        } else {
                            return Err(problems::rt_indexes_on_ct_variable(
                                full_expression.clone(),
                                &base_type,
                            ));
                        };
                        let base_expr = o::VPExpression::Literal(base_data, base_pos);
                        o::VPExpression::Index {
                            base: Box::new(base_expr),
                            indexes: vec![index_expr],
                            position: pos,
                        }
                    }
                    ResolvedVPExpression::Modified(base_expr, base_type) => {
                        // Otherwise, if it's an index expression, add the new index to it.
                        if let o::VPExpression::Index {
                            base, mut indexes, ..
                        } = base_expr
                        {
                            indexes.push(index_expr);
                            o::VPExpression::Index {
                                base,
                                indexes,
                                position: pos,
                            }
                        // Otherwise, make an entirely new index expression.
                        } else {
                            o::VPExpression::Index {
                                base: Box::new(base_expr),
                                indexes: vec![index_expr],
                                position: pos,
                            }
                        }
                    }
                };
                Ok(ResolvedVPExpression::Modified(expr, etype))
            }
        }
    }

    fn resolve_property_access(
        &mut self,
        prop: i::Property,
        rhs: &i::VPExpression,
        position: &FilePosition,
    ) -> Result<ResolvedVPExpression, CompileProblem> {
        let res_rhs = self.resolve_vp_expression(rhs)?;
        Ok(match prop {
            i::Property::Type => ResolvedVPExpression::Interpreted(
                i::KnownData::DataType(res_rhs.borrow_data_type().clone()),
                position.clone(),
                i::SpecificDataType::DataType.into(),
            ),
            i::Property::Dims => {
                let dims =
                    if let ResolvedVPExpression::Interpreted(i::KnownData::DataType(typ), ..) =
                        res_rhs
                    {
                        // TODO: Nice error if typ is a type bound.
                        typ.actual_type.as_ref().unwrap().collect_dims()
                    } else {
                        // We don't have to worry about an uninterpreted data type because there's
                        // no such thing.
                        res_rhs.borrow_actual_data_type().collect_dims()
                    };
                let kdims: Vec<_> = dims
                    .into_iter()
                    .map(|dim| i::KnownData::Int(dim as i64))
                    .collect();
                let len = kdims.len();
                let (value, typ) = if len == 0 {
                    (i::KnownData::Int(1), i::SpecificDataType::Int)
                } else {
                    (
                        i::KnownData::Array(kdims),
                        i::SpecificDataType::Array(len, Box::new(i::SpecificDataType::Int)),
                    )
                };
                ResolvedVPExpression::Interpreted(value, position.clone(), typ.into())
            }
        })
    }

    fn resolve_unary_operation(
        &mut self,
        op: i::UnaryOperator,
        rhs: &i::VPExpression,
        position: &FilePosition,
    ) -> Result<ResolvedVPExpression, CompileProblem> {
        if let i::UnaryOperator::PropertyAccess(prop) = op {
            return self.resolve_property_access(prop, rhs, position);
        }
        let res_rhs = self.resolve_vp_expression(rhs)?;
        // TODO: Check that the operand has a data type compatible with the operator.
        let result_type = match op {
            i::UnaryOperator::Ftoi => res_rhs
                .borrow_data_type()
                .with_different_base(i::SpecificDataType::Int),
            i::UnaryOperator::Itof => res_rhs
                .borrow_data_type()
                .with_different_base(i::SpecificDataType::Float),
            i::UnaryOperator::PropertyAccess(..) => unreachable!("Handled earlier."),
            i::UnaryOperator::BNot
            | i::UnaryOperator::Negate
            | i::UnaryOperator::Not
            | i::UnaryOperator::Reciprocal
            | i::UnaryOperator::Sine
            | i::UnaryOperator::Cosine
            | i::UnaryOperator::SquareRoot
            | i::UnaryOperator::Exp
            | i::UnaryOperator::Exp2
            | i::UnaryOperator::Log
            | i::UnaryOperator::Log10
            | i::UnaryOperator::Log2
            | i::UnaryOperator::Absolute
            | i::UnaryOperator::Floor
            | i::UnaryOperator::Ceiling
            | i::UnaryOperator::Truncate => res_rhs.borrow_data_type().clone(),
        };
        Ok(
            if let ResolvedVPExpression::Interpreted(data, pos, ..) = res_rhs {
                ResolvedVPExpression::Interpreted(
                    Self::compute_unary_operation(op, &data),
                    pos,
                    result_type,
                )
            } else {
                let res_op = match op {
                    i::UnaryOperator::PropertyAccess(..) => unreachable!("Handled earlier."),
                    i::UnaryOperator::BNot => o::UnaryOperator::BNot,
                    i::UnaryOperator::Negate => o::UnaryOperator::Negate,
                    i::UnaryOperator::Not => o::UnaryOperator::Not,
                    i::UnaryOperator::Reciprocal => o::UnaryOperator::Reciprocal,
                    i::UnaryOperator::Sine => o::UnaryOperator::Sine,
                    i::UnaryOperator::Cosine => o::UnaryOperator::Cosine,
                    i::UnaryOperator::SquareRoot => o::UnaryOperator::SquareRoot,
                    i::UnaryOperator::Exp => o::UnaryOperator::Exp,
                    i::UnaryOperator::Exp2 => o::UnaryOperator::Exp2,
                    i::UnaryOperator::Log => o::UnaryOperator::Log,
                    i::UnaryOperator::Log10 => o::UnaryOperator::Log10,
                    i::UnaryOperator::Log2 => o::UnaryOperator::Log2,
                    i::UnaryOperator::Absolute => o::UnaryOperator::Absolute,
                    i::UnaryOperator::Floor => o::UnaryOperator::Floor,
                    i::UnaryOperator::Ceiling => o::UnaryOperator::Ceiling,
                    i::UnaryOperator::Truncate => o::UnaryOperator::Truncate,
                    i::UnaryOperator::Ftoi => o::UnaryOperator::Ftoi,
                    i::UnaryOperator::Itof => o::UnaryOperator::Itof,
                };
                ResolvedVPExpression::Modified(
                    o::VPExpression::UnaryOperation(
                        res_op,
                        Box::new(res_rhs.as_vp_expression()?),
                        position.clone(),
                    ),
                    result_type,
                )
            },
        )
    }

    fn resolve_binary_operation(
        &mut self,
        lhs: &i::VPExpression,
        operator: i::BinaryOperator,
        rhs: &i::VPExpression,
        position: &FilePosition,
    ) -> Result<ResolvedVPExpression, CompileProblem> {
        // TODO: Check that the operand has a data type compatible with the operator.
        let res_lhs = self.resolve_vp_expression(lhs)?;
        let res_rhs = self.resolve_vp_expression(rhs)?;
        // This operator works differently from all the otehrs. The RHS must be a data type and the
        // LHS can be anything.
        if operator == i::BinaryOperator::As {
            if res_rhs.borrow_actual_data_type() != &i::SpecificDataType::DataType {
                return Err(problems::bad_type_for_operator(
                    rhs.clone_position(),
                    "as operator",
                    "a DATA_TYPE",
                    res_rhs.borrow_actual_data_type(),
                ));
            }
            let as_type: i::DataType = if let ResolvedVPExpression::Interpreted(data, ..) = res_rhs
            {
                let type_bound = data.require_data_type();
                let value = if let Some(as_type) = type_bound.actual_type.as_ref() {
                    as_type.clone().into()
                } else {
                    return Err(problems::as_type_bound(rhs.clone_position()));
                };
                // If the LHS is already interpreted don't bother with the rest of this stuff just
                // interpret the value and return it.
                if let ResolvedVPExpression::Interpreted(lhsv, _, _) = res_lhs {
                    let inflated = Self::compute_binary_operation(&lhsv, operator, &data);
                    let typ = inflated.get_specific_data_type().into();
                    return Ok(ResolvedVPExpression::Interpreted(
                        inflated,
                        position.clone(),
                        typ,
                    ));
                }
                value
            } else {
                unreachable!("Handled above");
            };
            if Self::biggest_type(res_lhs.borrow_data_type(), &as_type).as_ref() != Ok(&as_type) {
                return Err(problems::cannot_inflate(
                    position.clone(),
                    res_lhs.borrow_data_type(),
                    &as_type.actual_type.unwrap(),
                ));
            }
            // The trivial phase handles the actual inflation logic.
            return Ok(match res_lhs {
                ResolvedVPExpression::Interpreted(..) => unreachable!("Handled earlier."),
                ResolvedVPExpression::Modified(e, _) => ResolvedVPExpression::Modified(e, as_type),
            });
        }
        let bct = if let Ok(bct) =
            Self::biggest_type(res_lhs.borrow_data_type(), res_rhs.borrow_data_type())
        {
            bct
        } else {
            return Err(problems::no_bct_binop(
                position.clone(),
                lhs.clone_position(),
                res_lhs.borrow_data_type(),
                rhs.clone_position(),
                res_rhs.borrow_data_type(),
            ));
        };
        let bct = match operator {
            i::BinaryOperator::LessThan
            | i::BinaryOperator::LessThanOrEqual
            | i::BinaryOperator::GreaterThan
            | i::BinaryOperator::GreaterThanOrEqual
            | i::BinaryOperator::Equal
            | i::BinaryOperator::NotEqual
            | i::BinaryOperator::In => bct.with_different_base(i::SpecificDataType::Bool),
            _ => bct,
        };
        if let (
            ResolvedVPExpression::Interpreted(lhs_data, ..),
            ResolvedVPExpression::Interpreted(rhs_data, ..),
        ) = (&res_lhs, &res_rhs)
        {
            let result = Self::compute_binary_operation(lhs_data, operator, rhs_data);
            debug_assert!(Some(result.get_specific_data_type()) == bct.actual_type);
            Ok(ResolvedVPExpression::Interpreted(
                result,
                position.clone(),
                bct,
            ))
        } else {
            Ok(ResolvedVPExpression::Modified(
                o::VPExpression::BinaryOperation {
                    lhs: Box::new(res_lhs.as_vp_expression()?),
                    op: Self::resolve_operator(operator),
                    rhs: Box::new(res_rhs.as_vp_expression()?),
                    typ: Self::resolve_data_type(bct.actual_type.as_ref().unwrap()).expect(
                        "Resolving lhs or rhs should have failed if the result is ct-only.",
                    ),
                    position: position.clone(),
                },
                bct,
            ))
        }
    }

    fn resolve_vp_index(
        &mut self,
        base: &i::VPExpression,
        indexes: &Vec<(i::VPExpression, bool)>, // The bool is whether or not the index is optional.
        position: &FilePosition,
    ) -> Result<ResolvedVPExpression, CompileProblem> {
        // Resolve the base expression that is being indexed.
        let resolved_base = self.resolve_vp_expression(base)?;
        let mut result = resolved_base;
        for (index, optional) in indexes {
            result = self.resolve_vp_index_impl(result, index, *optional, position)?;
        }
        Ok(result)
    }

    fn resolve_macro_call(
        &mut self,
        mcro: &i::VPExpression,
        inputs: &Vec<i::VPExpression>,
        outputs: &Vec<i::FuncCallOutput>,
        position: &FilePosition,
    ) -> Result<ResolvedVPExpression, CompileProblem> {
        // Find out what macro we are calling.
        let rmacro = self.resolve_vp_expression(mcro)?;
        if rmacro.borrow_actual_data_type() != &i::SpecificDataType::Macro {
            return Err(problems::not_macro(
                rmacro.clone_position(),
                rmacro.borrow_data_type(),
            ));
        }
        let macro_data = if let ResolvedVPExpression::Interpreted(data, ..) = rmacro {
            if let i::KnownData::Macro(data) = data {
                data
            } else {
                unreachable!("Already checked that data is a macro.");
            }
        } else {
            unreachable!("Data cannot be both a macro and run-time only.");
        };
        // Do this before we push the macro's scope so that it happens in the context of the macro
        // call, not the macro body.
        let mut rinputs = Vec::new();
        for input in inputs {
            rinputs.push(self.resolve_vp_expression(input)?);
        }
        let body_scope = macro_data.get_body();
        let rscope = self.target.create_scope();
        let old_scope = self.current_scope;
        self.current_scope = rscope;
        self.push_temp_table(macro_data.borrow_context().clone());

        // Copy each input value to a new variable. If we know what the input value is at compile
        // time, then just set its temporary value without creating an actual variable for it.
        let macro_inputs = self.source[body_scope].borrow_inputs().clone();
        if rinputs.len() != macro_inputs.len() {
            return Err(problems::wrong_number_of_inputs(
                position.clone(),
                macro_data.get_header().clone(),
                rinputs.len(),
                macro_inputs.len(),
            ));
        }
        for (index, rinput) in rinputs.into_iter().enumerate() {
            let input_id = macro_inputs[index];
            if let ResolvedVPExpression::Interpreted(data, _, dtype) = rinput {
                self.set_var_info(input_id, None, dtype);
                self.set_temporary_value(input_id, PossiblyKnownData::from_known_data(&data));
            } else if let ResolvedVPExpression::Modified(rinput, dtype) = rinput {
                let pos = rinput.clone_position();
                // It is impossible to get a dynamic expression that returns compile time only data.
                let odtype = Self::resolve_data_type(dtype.actual_type.as_ref().unwrap()).unwrap();
                let input_in_body = o::Variable::new(pos.clone(), odtype);
                let input_in_body_id = self.target.adopt_variable(input_in_body);
                self.set_var_info(macro_inputs[index], Some(input_in_body_id), dtype);
                self.reset_temporary_value(input_id);
                self.target[self.current_scope].add_statement(o::Statement::Assign {
                    target: Box::new(o::VCExpression::variable(input_in_body_id, pos.clone())),
                    value: Box::new(rinput),
                    position: pos.clone(),
                });
            }
        }

        // Resolve all the statements into the new body.
        for statement in self.source[body_scope].borrow_body().clone() {
            if let ResolvedStatement::Modified(statement) = self.resolve_statement(&statement)? {
                self.target[rscope].add_statement(statement);
            }
        }

        // This is necessary so that we can access information about both the variables in the macro
        // body and variables in the context that the macro call was in.
        self.fuse_top_table();
        // Copy all the output values to the VCEs given in the macro call.
        let mut inline_return = None;
        let macro_outputs = self.source[body_scope].borrow_outputs().clone();
        if outputs.len() != macro_outputs.len() {
            return Err(problems::wrong_number_of_outputs(
                position.clone(),
                macro_data.get_header().clone(),
                outputs.len(),
                macro_outputs.len(),
            ));
        }
        for index in 0..macro_outputs.len() {
            match &outputs[index] {
                i::FuncCallOutput::InlineReturn(..) => {
                    debug_assert!(inline_return.is_none());
                    inline_return = Some(macro_outputs[index]);
                }
                i::FuncCallOutput::VCExpression(vce) => {
                    let pos = vce.clone_position();
                    // This will handle all the icky optiization stuff for us.
                    let rs = self.resolve_assign_statement(
                        &vce,
                        &i::VPExpression::Variable(macro_outputs[index], pos.clone()),
                        &pos,
                    )?;
                    if let ResolvedStatement::Modified(news) = rs {
                        self.target[self.current_scope].add_statement(news);
                    }
                }
            }
        }

        let result = if let Some(output_var) = inline_return {
            // Undefined output should be caught by earlier phase.
            let dtype = self.get_var_info(output_var).unwrap().1.clone();
            let pkd = self.borrow_temporary_value(output_var);
            if let Ok(known_data) = pkd.to_known_data() {
                ResolvedVPExpression::Interpreted(known_data, position.clone(), dtype)
            } else {
                // A variable cannot carry an indeterminate value while being compile-time only.
                let var_id = self.get_var_info(output_var).unwrap().0.unwrap();
                ResolvedVPExpression::Modified(
                    o::VPExpression::Variable(var_id, position.clone()),
                    dtype,
                )
            }
        } else {
            ResolvedVPExpression::Interpreted(
                i::KnownData::Void,
                position.clone(),
                i::SpecificDataType::Void.into(),
            )
        };

        self.pop_table();
        self.current_scope = old_scope;
        // Add a statement to call the body we just made.
        self.target[self.current_scope].add_statement(o::Statement::MacroCall {
            mcro: rscope,
            position: position.clone(),
        });

        Ok(result)
    }

    pub(super) fn resolve_vp_expression(
        &mut self,
        input: &i::VPExpression,
    ) -> Result<ResolvedVPExpression, CompileProblem> {
        Ok(match input {
            i::VPExpression::Literal(value, pos) => ResolvedVPExpression::Interpreted(
                value.clone(),
                pos.clone(),
                value.get_specific_data_type().into(),
            ),
            i::VPExpression::Variable(id, position) => self.resolve_vp_variable(*id, position)?,
            i::VPExpression::Collect(items, position) => self.resolve_collect(items, position)?,
            i::VPExpression::UnboundedTypeBound(position) => ResolvedVPExpression::Interpreted(
                i::KnownData::DataType(i::DataType::unbounded()),
                position.clone(),
                i::SpecificDataType::DataType.into(),
            ),
            i::VPExpression::TypeBound {
                lower,
                upper,
                position,
            } => self.resolve_type_bound(lower, upper, position)?,
            i::VPExpression::BuildArrayType {
                dimensions,
                base,
                position,
            } => self.resolve_build_array_type(dimensions, base, position)?,

            i::VPExpression::UnaryOperation(op, a, position) => {
                self.resolve_unary_operation(*op, a, position)?
            }
            i::VPExpression::BinaryOperation(lhs, operator, rhs, position) => {
                self.resolve_binary_operation(lhs, *operator, rhs, position)?
            }
            i::VPExpression::Index {
                base,
                indexes,
                position,
            } => self.resolve_vp_index(base, indexes, position)?,
            i::VPExpression::MacroCall {
                mcro,
                inputs,
                outputs,
                position,
            } => self.resolve_macro_call(mcro, inputs, outputs, position)?,
        })
    }
}
