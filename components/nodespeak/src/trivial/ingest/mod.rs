use crate::high_level::compiler::SourceSet;
use crate::high_level::problem::{CompileProblem, FilePosition};
use crate::resolved::structure as i;
use crate::shared as s;
use crate::trivial::structure as o;
use std::borrow::Borrow;
use std::collections::{HashMap, HashSet};

mod problems;

pub fn ingest(program: &i::Program, sources: &SourceSet) -> Result<o::Program, CompileProblem> {
    let mut trivializer = Trivializer::new(program, sources);
    trivializer.entry_point()?;
    Result::Ok(trivializer.target)
}

struct Trivializer<'a> {
    source: &'a i::Program,
    source_set: &'a SourceSet,
    target: o::Program,
    variable_map: HashMap<i::VariableId, o::VariableId>,
    illegal_vars: HashSet<i::VariableId>,
    trivializing_static_init: bool,
}

impl<'a> Trivializer<'a> {
    fn new<'n>(source: &'n i::Program, source_set: &'n SourceSet) -> Trivializer<'n> {
        Trivializer {
            source,
            source_set,
            target: o::Program::new(),
            variable_map: HashMap::new(),
            illegal_vars: HashSet::new(),
            trivializing_static_init: true,
        }
    }

    fn entry_point(&mut self) -> Result<(), CompileProblem> {
        let source_static_vars = self.source.borrow_static_vars();
        for static_var in source_static_vars {
            self.trivialize_variable_custom_location(*static_var, o::StorageLocation::Static)?;
        }
        // We want to keep the static init and main body separated, they should not share any
        // variable IDs except for the static variables.
        let old_var_map = self.variable_map.clone();
        let source_inputs = self.source.borrow_inputs();
        for input in source_inputs {
            self.trivialize_variable_custom_location(*input, o::StorageLocation::Input)?;
        }
        let source_outputs = self.source.borrow_outputs();
        for output in source_outputs {
            self.trivialize_variable_custom_location(*output, o::StorageLocation::Output)?;
        }
        let source_entry_point = self.source.get_entry_point();
        self.trivializing_static_init = false;
        for statement in self.source[source_entry_point].borrow_body().clone() {
            self.trivialize_statement(&statement)?;
        }

        // Every var used in the main body, including static vars.
        let main_and_static_vars: HashSet<_> = self.variable_map.keys().cloned().collect();
        self.variable_map = old_var_map;
        // All static vars.
        let static_vars: HashSet<_> = self.variable_map.keys().cloned().collect();
        // If we encounter a variable in the static init body that is in this list, it is illegal
        // because it only exists in the main body.
        self.illegal_vars = main_and_static_vars
            .difference(&static_vars)
            .cloned()
            .collect();
        let source_static_init = self.source.get_static_init();
        self.trivializing_static_init = true;
        for statement in self.source[source_static_init].borrow_body().clone() {
            self.trivialize_statement(&statement)?;
        }
        Result::Ok(())
    }

    fn add_instruction(&mut self, instruction: o::Instruction) {
        if self.trivializing_static_init {
            self.target.add_static_init_instruction(instruction);
        } else {
            self.target.add_instruction(instruction);
        }
    }

    fn create_label(&mut self) -> o::LabelId {
        self.target.create_label(self.trivializing_static_init)
    }

    fn bct_dimensions(type1: &o::DataType, type2: &o::DataType) -> Vec<usize> {
        let t1dims = type1.collect_dimensions();
        let t2dims = type2.collect_dimensions();
        let mut bctdims = Vec::new();
        for index in 0..(t1dims.len().max(t2dims.len())) {
            let dim = if index >= t1dims.len() {
                t2dims[index]
            } else if index >= t2dims.len() {
                t1dims[index]
            } else if t1dims[index] == 1 {
                t2dims[index]
            } else if t2dims[index] == 1 {
                t1dims[index]
            } else if t1dims[index] == t2dims[index] {
                t1dims[index]
            } else {
                unreachable!("Invalid bct should have been caught earlier.")
            };
            bctdims.push(dim);
        }
        bctdims
    }

    fn trivialize_known_data(data: &i::KnownData) -> Result<o::KnownData, CompileProblem> {
        Result::Ok(match data {
            i::KnownData::Int(value) => o::KnownData::Int(*value),
            i::KnownData::Float(value) => o::KnownData::Float(*value),
            i::KnownData::Bool(value) => o::KnownData::Bool(*value),
            i::KnownData::Array(items) => {
                let mut titems = Vec::with_capacity(items.len());
                for item in items {
                    titems.push(Self::trivialize_known_data(item)?);
                }
                o::KnownData::Array(titems)
            }
        })
    }

    fn trivialize_data_type(data_type: &i::DataType) -> o::DataType {
        match data_type {
            i::DataType::Float => o::DataType::F32,
            i::DataType::Int => o::DataType::I32,
            i::DataType::Bool => o::DataType::B1,
            i::DataType::Array(len, base) => {
                o::DataType::Array(*len, Box::new(Self::trivialize_data_type(base)))
            }
        }
    }

    fn default_storage_location(&self) -> o::StorageLocation {
        if self.trivializing_static_init {
            o::StorageLocation::StaticBody
        } else {
            o::StorageLocation::MainBody
        }
    }

    fn create_variable_custom_location(
        &mut self,
        typ: o::DataType,
        loc: o::StorageLocation,
    ) -> o::VariableId {
        let var = o::Variable::new(typ, loc);
        self.target.adopt_variable(var)
    }

    fn create_variable(&mut self, typ: o::DataType) -> o::VariableId {
        self.create_variable_custom_location(typ, self.default_storage_location())
    }

    fn trivialize_variable_custom_location(
        &mut self,
        variable: i::VariableId,
        location: o::StorageLocation,
    ) -> Result<o::VariableId, CompileProblem> {
        // We don't have to worry about checking for static body vars in the main body because
        // syntax doesn't allow accessing static body vars from the main body:
        // INT this_is_what_the_check_is_for = some_runtime_only_value;
        // static fine {
        //     INT not_a_problem = 12;
        //     INT fine = not_a_problem;
        //     assert this_is_what_the_check_is_for == 3333;
        // }
        // assert fine == 12;
        if self.trivializing_static_init && self.illegal_vars.contains(&variable) {
            panic!("TODO: Nice error, a variable from the main body was used in static init.");
        }
        Result::Ok(match self.variable_map.get(&variable) {
            Some(trivialized) => *trivialized,
            None => {
                let data_type = self.source[variable].borrow_data_type();
                let typ = Self::trivialize_data_type(data_type);
                let id = self.create_variable_custom_location(typ, location);
                self.variable_map.insert(variable, id);
                id
            }
        })
    }

    fn trivialize_variable(
        &mut self,
        variable: i::VariableId,
    ) -> Result<o::VariableId, CompileProblem> {
        self.trivialize_variable_custom_location(variable, self.default_storage_location())
    }

    fn trivialize_unary_expression(
        &mut self,
        operator: i::UnaryOperator,
        right: &i::VPExpression,
    ) -> Result<o::Value, CompileProblem> {
        let a = self.trivialize_vp_expression(right)?;
        let out_typ = a.get_type(&self.target);
        let mut base = out_typ.clone();
        while let o::DataType::Array(_, etype) = base {
            base = *etype;
        }
        let out_base = match operator {
            i::UnaryOperator::Itof => o::DataType::F32,
            i::UnaryOperator::Ftoi => o::DataType::I32,
            _ => base.clone(),
        };
        let out_typ = out_typ.with_different_base(out_base);
        let x_var = self.create_variable(out_typ);
        let x = o::Value::variable(x_var, &self.target);
        let toperator = match operator {
            i::UnaryOperator::Absolute => match base {
                o::DataType::F32 => o::UnaryOperator::FAbs,
                o::DataType::I32 => o::UnaryOperator::IAbs,
                _ => unreachable!(),
            },
            i::UnaryOperator::BNot => o::UnaryOperator::BNot,
            i::UnaryOperator::Ceiling => o::UnaryOperator::FCeil,
            i::UnaryOperator::Cosine => o::UnaryOperator::FCos,
            i::UnaryOperator::Exp => o::UnaryOperator::FExp,
            i::UnaryOperator::Exp2 => o::UnaryOperator::FExp2,
            i::UnaryOperator::Floor => o::UnaryOperator::FFloor,
            i::UnaryOperator::Log => o::UnaryOperator::FLog,
            i::UnaryOperator::Log10 => o::UnaryOperator::FLog10,
            i::UnaryOperator::Log2 => o::UnaryOperator::FLog2,
            i::UnaryOperator::Negate => match base {
                o::DataType::I32 => o::UnaryOperator::NegI,
                o::DataType::F32 => o::UnaryOperator::NegF,
                _ => unreachable!(),
            },
            i::UnaryOperator::Not => o::UnaryOperator::Not,
            i::UnaryOperator::Reciprocal => unimplemented!(),
            i::UnaryOperator::Sine => o::UnaryOperator::FSin,
            i::UnaryOperator::SquareRoot => o::UnaryOperator::FSqrt,
            i::UnaryOperator::Truncate => o::UnaryOperator::FTrunc,
            i::UnaryOperator::Ftoi => o::UnaryOperator::Ftoi,
            i::UnaryOperator::Itof => o::UnaryOperator::Itof,
        };
        self.add_instruction(o::Instruction::UnaryOperation {
            a,
            x: x.clone(),
            op: toperator,
        });
        Ok(x)
    }

    fn trivialize_binary_expression(
        &mut self,
        left: &i::VPExpression,
        operator: i::BinaryOperator,
        right: &i::VPExpression,
        out_typ: &i::DataType,
    ) -> Result<o::Value, CompileProblem> {
        let mut a = self.trivialize_vp_expression(left)?;
        let mut b = self.trivialize_vp_expression(right)?;
        let bct_dims = Self::bct_dimensions(&a.get_type(&self.target), &b.get_type(&self.target));
        a.inflate(&bct_dims[..]);
        b.inflate(&bct_dims[..]);
        let out_typ = Self::trivialize_data_type(out_typ);
        let mut base = a.get_type(&self.target);
        while let o::DataType::Array(_, etype) = base {
            base = *etype;
        }
        let x_var = self.create_variable(out_typ);
        let x = o::Value::variable(x_var, &self.target);
        let x2 = x.clone();
        let toperator = match operator {
            i::BinaryOperator::Add => match base {
                o::DataType::F32 => o::BinaryOperator::AddF,
                o::DataType::I32 => o::BinaryOperator::AddI,
                o::DataType::B1 => unimplemented!(),
                o::DataType::Array(..) => unreachable!(),
            },
            i::BinaryOperator::Subtract => match base {
                o::DataType::F32 => o::BinaryOperator::SubF,
                o::DataType::I32 => o::BinaryOperator::SubI,
                o::DataType::B1 => unimplemented!(),
                o::DataType::Array(..) => unreachable!(),
            },
            i::BinaryOperator::Multiply => match base {
                o::DataType::F32 => o::BinaryOperator::MulF,
                o::DataType::I32 => o::BinaryOperator::MulI,
                o::DataType::B1 => unimplemented!(),
                o::DataType::Array(..) => unreachable!(),
            },
            i::BinaryOperator::Divide => match base {
                o::DataType::F32 => o::BinaryOperator::DivF,
                o::DataType::I32 => o::BinaryOperator::DivI,
                o::DataType::B1 => unimplemented!(),
                o::DataType::Array(..) => unreachable!(),
            },
            i::BinaryOperator::Modulo => match base {
                o::DataType::F32 => o::BinaryOperator::ModF,
                o::DataType::I32 => o::BinaryOperator::ModI,
                o::DataType::B1 => unimplemented!(),
                o::DataType::Array(..) => unreachable!(),
            },
            i::BinaryOperator::Power => {
                assert!(base == o::DataType::F32);
                o::BinaryOperator::PowF
            }

            i::BinaryOperator::Equal => match base {
                o::DataType::F32 => o::BinaryOperator::CompF(o::Condition::Equal),
                o::DataType::I32 => o::BinaryOperator::CompI(o::Condition::Equal),
                o::DataType::B1 => unimplemented!(),
                o::DataType::Array(..) => unreachable!(),
            },
            i::BinaryOperator::NotEqual => match base {
                o::DataType::F32 => o::BinaryOperator::CompF(o::Condition::NotEqual),
                o::DataType::I32 => o::BinaryOperator::CompI(o::Condition::NotEqual),
                o::DataType::B1 => unimplemented!(),
                o::DataType::Array(..) => unreachable!(),
            },
            i::BinaryOperator::GreaterThan => match base {
                o::DataType::F32 => o::BinaryOperator::CompF(o::Condition::GreaterThan),
                o::DataType::I32 => o::BinaryOperator::CompI(o::Condition::GreaterThan),
                o::DataType::B1 => unimplemented!(),
                o::DataType::Array(..) => unreachable!(),
            },
            i::BinaryOperator::GreaterThanOrEqual => match base {
                o::DataType::F32 => o::BinaryOperator::CompF(o::Condition::GreaterThanOrEqual),
                o::DataType::I32 => o::BinaryOperator::CompI(o::Condition::GreaterThanOrEqual),
                o::DataType::B1 => unimplemented!(),
                o::DataType::Array(..) => unreachable!(),
            },
            i::BinaryOperator::LessThan => match base {
                o::DataType::F32 => o::BinaryOperator::CompF(o::Condition::LessThan),
                o::DataType::I32 => o::BinaryOperator::CompI(o::Condition::LessThan),
                o::DataType::B1 => unimplemented!(),
                o::DataType::Array(..) => unreachable!(),
            },
            i::BinaryOperator::LessThanOrEqual => match base {
                o::DataType::F32 => o::BinaryOperator::CompF(o::Condition::LessThanOrEqual),
                o::DataType::I32 => o::BinaryOperator::CompI(o::Condition::LessThanOrEqual),
                o::DataType::B1 => unimplemented!(),
                o::DataType::Array(..) => unreachable!(),
            },

            i::BinaryOperator::BAnd => match base {
                o::DataType::F32 => unimplemented!(),
                o::DataType::I32 => o::BinaryOperator::BAnd,
                o::DataType::B1 => unimplemented!(),
                o::DataType::Array(..) => unreachable!(),
            },
            i::BinaryOperator::BOr => match base {
                o::DataType::F32 => unimplemented!(),
                o::DataType::I32 => o::BinaryOperator::BOr,
                o::DataType::B1 => unimplemented!(),
                o::DataType::Array(..) => unreachable!(),
            },
            i::BinaryOperator::BXor => match base {
                o::DataType::F32 => unimplemented!(),
                o::DataType::I32 => o::BinaryOperator::BXor,
                o::DataType::B1 => unimplemented!(),
                o::DataType::Array(..) => unreachable!(),
            },
            i::BinaryOperator::And => match base {
                o::DataType::F32 => unimplemented!(),
                o::DataType::I32 => unimplemented!(),
                o::DataType::B1 => o::BinaryOperator::And,
                o::DataType::Array(..) => unreachable!(),
            },
            i::BinaryOperator::Or => match base {
                o::DataType::F32 => unimplemented!(),
                o::DataType::I32 => unimplemented!(),
                o::DataType::B1 => o::BinaryOperator::Or,
                o::DataType::Array(..) => unreachable!(),
            },
            i::BinaryOperator::Xor => match base {
                o::DataType::F32 => unimplemented!(),
                o::DataType::I32 => unimplemented!(),
                o::DataType::B1 => o::BinaryOperator::Xor,
                o::DataType::Array(..) => unreachable!(),
            },
            i::BinaryOperator::LeftShift => match base {
                o::DataType::F32 => unimplemented!(),
                o::DataType::I32 => o::BinaryOperator::LeftShift,
                o::DataType::B1 => unimplemented!(),
                o::DataType::Array(..) => unreachable!(),
            },
            i::BinaryOperator::RightShift => match base {
                o::DataType::F32 => unimplemented!(),
                o::DataType::I32 => o::BinaryOperator::RightShift,
                o::DataType::B1 => unimplemented!(),
                o::DataType::Array(..) => unreachable!(),
            },
        };

        self.add_instruction(o::Instruction::BinaryOperation {
            op: toperator,
            a,
            b,
            x,
        });

        Result::Ok(x2)
    }

    fn trivialize_collect(
        &mut self,
        items: &Vec<i::VPExpression>,
    ) -> Result<o::Value, CompileProblem> {
        let mut titems = Vec::new();
        for item in items {
            titems.push(self.trivialize_vp_expression(item)?);
        }
        assert!(titems.len() > 0);
        let item_type = titems[0].get_type(&self.target);
        let array_type = o::DataType::Array(titems.len(), Box::new(item_type));
        let target_var = self.create_variable(array_type);
        let target_value = o::Value::variable(target_var, &self.target);
        for (index, item) in titems.into_iter().enumerate() {
            self.add_instruction(o::Instruction::Store {
                from: item,
                to: target_value.clone(),
                to_indexes: vec![o::Value::literal(o::KnownData::Int(index as _))],
            })
        }
        Ok(target_value)
    }

    fn trivialize_assert(
        &mut self,
        condition: &i::VPExpression,
        position: &FilePosition,
    ) -> Result<(), CompileProblem> {
        let tcondition = self.trivialize_vp_expression(condition)?;
        let abort_label = self.create_label();
        let skip_label = self.create_label();
        self.add_instruction(o::Instruction::Branch {
            condition: tcondition,
            true_target: skip_label,
            false_target: abort_label,
        });
        self.add_instruction(o::Instruction::Label(abort_label));
        let location = position.create_line_column_ref(self.source_set);
        let error_code = self
            .target
            .add_error(format!("Assert failed at {}", location));
        self.add_instruction(o::Instruction::Abort(error_code));
        self.add_instruction(o::Instruction::Label(skip_label));
        Ok(())
    }

    fn trivialize_assignment(
        &mut self,
        statement: &i::Statement,
        target: &i::VCExpression,
        value: &i::VPExpression,
    ) -> Result<(), CompileProblem> {
        let base = self.trivialize_variable(target.base)?;
        let base_type = self.target[base].borrow_type().clone();
        let (new_indexes, base_type) = self.trivialize_indexes(&target.indexes, base_type)?;
        let mut tvalue = self.trivialize_vp_expression(value.borrow())?;
        tvalue.inflate(&base_type.collect_dimensions());
        let base = o::Value::variable(base, &self.target);

        if target.indexes.len() > 0 {
            self.add_instruction(o::Instruction::Store {
                from: tvalue,
                to: base,
                to_indexes: new_indexes,
            });
        } else {
            self.add_instruction(o::Instruction::Move {
                from: tvalue,
                to: base,
            });
        }
        Ok(())
    }

    fn trivialize_branch(
        &mut self,
        clauses: &Vec<(i::VPExpression, i::ScopeId)>,
        else_clause: &Option<i::ScopeId>,
    ) -> Result<(), CompileProblem> {
        debug_assert!(clauses.len() > 0);
        let end_label = self.create_label();
        for (condition_expr, body) in clauses.iter() {
            let condition = self.trivialize_vp_expression(condition_expr)?;
            let body_label = self.create_label();
            let next_condition_label = self.create_label();
            self.add_instruction(o::Instruction::Branch {
                condition,
                true_target: body_label,
                false_target: next_condition_label,
            });
            self.add_instruction(o::Instruction::Label(body_label));
            for statement in self.source[*body].borrow_body().clone() {
                self.trivialize_statement(&statement)?;
            }
            self.add_instruction(o::Instruction::Jump { label: end_label });
            self.add_instruction(o::Instruction::Label(next_condition_label));
        }
        if let Some(body) = /* once told me */ else_clause {
            for statement in self.source[*body].borrow_body().clone() {
                self.trivialize_statement(&statement)?;
            }
        }
        self.add_instruction(o::Instruction::Label(end_label));
        Ok(())
    }

    fn trivialize_for_loop(
        &mut self,
        counter: i::VariableId,
        start: &i::VPExpression,
        end: &i::VPExpression,
        body: i::ScopeId,
    ) -> Result<(), CompileProblem> {
        let (start_label, end_label) = (self.create_label(), self.create_label());
        let tcount = o::Value::variable(self.trivialize_variable(counter)?, &self.target);
        let tstart = self.trivialize_vp_expression(start)?;
        let tend = self.trivialize_vp_expression(end)?;
        self.add_instruction(o::Instruction::Move {
            from: tstart,
            to: tcount.clone(),
        });
        let condition_var = self.create_variable(o::DataType::B1);
        let condition_var = o::Value::variable(condition_var, &self.target);
        self.add_instruction(o::Instruction::BinaryOperation {
            a: tcount.clone(),
            b: tend.clone(),
            x: condition_var.clone(),
            op: o::BinaryOperator::CompI(o::Condition::LessThan),
        });
        self.add_instruction(o::Instruction::Branch {
            condition: condition_var.clone(),
            true_target: start_label,
            false_target: end_label,
        });

        self.add_instruction(o::Instruction::Label(start_label));
        for statement in self.source[body].borrow_body().clone() {
            self.trivialize_statement(&statement)?;
        }

        self.add_instruction(o::Instruction::BinaryOperation {
            a: tcount.clone(),
            b: o::Value::literal(o::KnownData::Int(1)),
            x: tcount.clone(),
            op: o::BinaryOperator::AddI,
        });
        self.add_instruction(o::Instruction::BinaryOperation {
            a: tcount.clone(),
            b: tend,
            x: condition_var.clone(),
            op: o::BinaryOperator::CompI(o::Condition::LessThan),
        });
        self.add_instruction(o::Instruction::Branch {
            condition: condition_var.clone(),
            true_target: start_label,
            false_target: end_label,
        });
        self.add_instruction(o::Instruction::Label(end_label));
        Ok(())
    }

    fn trivialize_indexes(
        &mut self,
        indexes: &Vec<i::VPExpression>,
        data_type: o::DataType,
    ) -> Result<(Vec<o::Value>, o::DataType), CompileProblem> {
        let mut tindexes = Vec::with_capacity(indexes.len());
        let mut element_type = data_type;
        for index in indexes {
            let index_value = self.trivialize_vp_expression(index)?;
            let indext = index_value.get_type(&self.target);
            if indext != o::DataType::I32 {
                unreachable!("Ilegal index type should be handled by previous phase.");
            }
            tindexes.push(index_value);
            element_type = if let o::DataType::Array(_, etype) = element_type {
                *etype
            } else {
                unreachable!("Illegal array access should be handled by previous phase.")
            };
        }
        Ok((tindexes, element_type))
    }

    fn trivialize_index(
        &mut self,
        base: &i::VPExpression,
        indexes: &Vec<i::VPExpression>,
    ) -> Result<o::Value, CompileProblem> {
        let base_value = self.trivialize_vp_expression(base)?;
        let (new_indexes, result_type) =
            self.trivialize_indexes(indexes, base_value.get_type(&self.target))?;

        let output_holder = self.create_variable(result_type.clone());
        let mut output_value = o::Value::variable(output_holder, &self.target);
        output_value.dimensions = result_type
            .collect_dimensions()
            .iter()
            .map(|dim| (*dim, s::ProxyMode::Keep))
            .collect();
        self.add_instruction(o::Instruction::Load {
            from: base_value,
            to: output_value.clone(),
            from_indexes: new_indexes,
        });
        Ok(output_value)
    }

    fn trivialize_vp_expression(
        &mut self,
        expression: &i::VPExpression,
    ) -> Result<o::Value, CompileProblem> {
        Ok(match expression {
            i::VPExpression::Literal(data, ..) => {
                o::Value::literal(Self::trivialize_known_data(data)?)
            }
            i::VPExpression::Variable(id, ..) => {
                o::Value::variable(self.trivialize_variable(*id)?, &self.target)
            }
            i::VPExpression::Index { base, indexes, .. } => self.trivialize_index(base, indexes)?,

            i::VPExpression::UnaryOperation(op, rhs, ..) => {
                self.trivialize_unary_expression(*op, rhs)?
            }
            i::VPExpression::BinaryOperation {
                lhs, op, rhs, typ, ..
            } => self.trivialize_binary_expression(lhs, *op, rhs, typ)?,

            i::VPExpression::Collect(items, ..) => self.trivialize_collect(items)?,
        })
    }

    fn trivialize_statement(&mut self, statement: &i::Statement) -> Result<(), CompileProblem> {
        Ok(match statement {
            i::Statement::Assert(condition, position) => {
                self.trivialize_assert(condition, position)?
            }
            i::Statement::Assign { target, value, .. } => {
                self.trivialize_assignment(statement, target, value)?;
            }
            i::Statement::Return(..) => unimplemented!(),
            i::Statement::Branch {
                clauses,
                else_clause,
                ..
            } => {
                self.trivialize_branch(clauses, else_clause)?;
            }
            i::Statement::ForLoop {
                counter,
                start,
                end,
                body,
                ..
            } => {
                self.trivialize_for_loop(*counter, start, end, *body)?;
            }
            i::Statement::MacroCall { mcro, .. } => {
                for statement in self.source[*mcro].borrow_body().clone() {
                    self.trivialize_statement(&statement)?;
                }
            }
        })
    }
}
