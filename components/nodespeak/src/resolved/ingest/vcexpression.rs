use super::{problems, ResolvedVCExpression, ResolvedVPExpression, ScopeResolver};
use crate::high_level::problem::{CompileProblem, FilePosition};
use crate::resolved::structure as o;
use crate::vague::structure as i;

impl<'a> ScopeResolver<'a> {
    fn resolve_vc_variable(
        &mut self,
        var_id: i::VariableId,
        position: &FilePosition,
    ) -> Result<ResolvedVCExpression, CompileProblem> {
        let (_, var_type) = self
            .get_var_info(var_id)
            .expect("Variable used before declaration, vague step should have caught this.");
        Ok(ResolvedVCExpression::Specific {
            var: var_id,
            indexes: Vec::new(),
            typ: var_type.clone(),
            pos: position.clone(),
        })
    }

    fn resolve_vc_index(
        &mut self,
        base: &i::VCExpression,
        indexes: &Vec<(i::VPExpression, bool)>,
        position: &FilePosition,
    ) -> Result<ResolvedVCExpression, CompileProblem> {
        let rbase = self.resolve_vc_expression(base)?;

        let mut known_indexes = Vec::new();
        let mut all_indexes = Vec::new();
        let mut etype = rbase.borrow_data_type().clone();
        for (index, optional) in indexes {
            let arr_len;
            if let Some(eetype) = etype.indexed(*optional) {
                if let Some(i::SpecificDataType::Array(len, _)) = &etype.actual_type {
                    arr_len = *len;
                    etype = eetype;
                } else if *optional {
                    etype = eetype;
                    continue;
                } else {
                    unreachable!()
                }
            } else {
                return Err(problems::cannot_index(
                    position.clone(),
                    index.clone_position(),
                    &etype.min().unwrap(),
                ));
            }
            let rindex = self.resolve_vp_expression(index)?;
            if rindex.borrow_actual_data_type() != &i::SpecificDataType::Int {
                return Err(problems::array_index_not_int(
                    rindex.clone_position(),
                    rindex.borrow_data_type(),
                    position.clone(),
                ));
            }
            if let ResolvedVPExpression::Interpreted(data, pos, ..) = &rindex {
                let val = data.require_int(); // We already checked that it should be an int.
                if val < 0 {
                    return Err(problems::array_index_less_than_zero(
                        pos.clone(),
                        val,
                        position.clone(),
                    ));
                }
                let val = val as usize;
                if val >= arr_len {
                    return Err(problems::array_index_too_big(
                        pos.clone(),
                        val,
                        arr_len,
                        position.clone(),
                    ));
                }
                // If they are unequal, that means at some point we didn't know what one of the
                // earlier indexes was, so we should not add on any more known indexes because it's
                // not really useful in this phase. LLVM will still be able to do optimizations on
                // the literal values that will take their place.
                if known_indexes.len() == all_indexes.len() {
                    known_indexes.push(val);
                }
            }
            all_indexes.push(rindex.as_vp_expression()?);
        }

        let etype = etype.clone();
        Ok(match rbase {
            ResolvedVCExpression::Modified {
                mut vce,
                base,
                indexes,
                ..
            } => {
                // We can't add on our known indices. If all the previous indices were known, we
                // would have gotten a Specific result. Instead, since it is Modified, we cannot
                // add on our indexes to the end because the previous set of indexes is not
                // complete.
                vce.indexes.append(&mut all_indexes);
                ResolvedVCExpression::Modified {
                    vce,
                    typ: etype,
                    base,
                    indexes,
                }
            }
            ResolvedVCExpression::Specific {
                var,
                mut indexes,
                pos,
                typ,
            } => {
                let unknown_indexes = &all_indexes[known_indexes.len()..];
                indexes.append(&mut known_indexes);
                if unknown_indexes.len() == 0 {
                    ResolvedVCExpression::Specific {
                        var,
                        indexes,
                        pos,
                        typ: etype,
                    }
                } else {
                    let mut all_indexes = Vec::new();
                    for literal_index in &indexes {
                        all_indexes.push(o::VPExpression::Literal(
                            o::KnownData::Int(*literal_index as i64),
                            FilePosition::placeholder(),
                        ));
                    }
                    for index in unknown_indexes {
                        all_indexes.push(index.clone());
                    }
                    let resolved_var = if let Some((Some(id), _)) = self.get_var_info(var) {
                        *id
                    } else {
                        return Err(problems::rt_indexes_on_ct_variable(position.clone(), &typ));
                    };
                    ResolvedVCExpression::Modified {
                        vce: o::VCExpression::index(resolved_var, all_indexes, position.clone()),
                        typ: etype,
                        base: var,
                        indexes,
                    }
                }
            }
        })
    }

    pub(super) fn resolve_vc_expression(
        &mut self,
        input: &i::VCExpression,
    ) -> Result<ResolvedVCExpression, CompileProblem> {
        match input {
            i::VCExpression::Variable(id, position) => self.resolve_vc_variable(*id, position),
            i::VCExpression::Index {
                base,
                indexes,
                position,
            } => self.resolve_vc_index(base, indexes, position),
        }
    }
}
