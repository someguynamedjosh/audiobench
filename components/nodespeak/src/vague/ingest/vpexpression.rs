use super::problems;
use super::VagueIngester;
use crate::ast::structure as i;
use crate::high_level::problem::{CompileProblem, FilePosition};
use crate::vague::structure as o;

#[derive(Clone)]
enum Operator {
    Sentinel,
    Power,
    Multiply,
    Divide,
    Modulo,
    Add,
    Subtract,
    LeftShift,
    RightShift,
    Lte,
    Lt,
    Gte,
    Gt,
    Eq,
    Neq,
    Band,
    Bor,
    Bxor,
    And,
    Or,
    Xor,
}

impl Operator {
    fn precedence(&self) -> u32 {
        match self {
            Self::Power => 22,
            Self::Multiply => 21,
            Self::Divide => 21,
            Self::Modulo => 21,
            Self::Add => 20,
            Self::Subtract => 20,
            Self::LeftShift => 18,
            Self::RightShift => 18,
            Self::Band => 16,
            Self::Bxor => 15,
            Self::Bor => 14,
            Self::Lte => 13,
            Self::Lt => 13,
            Self::Gte => 13,
            Self::Gt => 13,
            Self::Eq => 13,
            Self::Neq => 13,
            Self::And => 12,
            Self::Xor => 11,
            Self::Or => 10,
            Self::Sentinel => 0,
        }
    }

    fn right_associative(&self) -> bool {
        match self {
            Self::Power => true,
            _ => false,
        }
    }

    fn bin_op(&self) -> o::BinaryOperator {
        match self {
            Self::Power => o::BinaryOperator::Power,
            Self::Multiply => o::BinaryOperator::Multiply,
            Self::Divide => o::BinaryOperator::Divide,
            Self::Modulo => o::BinaryOperator::Modulo,
            Self::Add => o::BinaryOperator::Add,
            Self::Subtract => o::BinaryOperator::Subtract,
            Self::LeftShift => o::BinaryOperator::LeftShift,
            Self::RightShift => o::BinaryOperator::RightShift,
            Self::Lte => o::BinaryOperator::LessThanOrEqual,
            Self::Lt => o::BinaryOperator::LessThan,
            Self::Gte => o::BinaryOperator::GreaterThanOrEqual,
            Self::Gt => o::BinaryOperator::GreaterThan,
            Self::Eq => o::BinaryOperator::Equal,
            Self::Neq => o::BinaryOperator::NotEqual,
            Self::Band => o::BinaryOperator::BAnd,
            Self::Bor => o::BinaryOperator::BOr,
            Self::Bxor => o::BinaryOperator::BXor,
            Self::And => o::BinaryOperator::And,
            Self::Or => o::BinaryOperator::Or,
            Self::Xor => o::BinaryOperator::Xor,
            Self::Sentinel => panic!(
                "Despite what Big Sentinel may have you think, Sentinel is not a real operator."
            ),
        }
    }
}

fn op_str_to_operator(op_str: &str) -> Operator {
    match op_str {
        "**" => Operator::Power,
        "+" => Operator::Add,
        "-" => Operator::Subtract,
        "*" => Operator::Multiply,
        "/" => Operator::Divide,
        "%" => Operator::Modulo,
        "<=" => Operator::Lte,
        "<" => Operator::Lt,
        ">=" => Operator::Gte,
        ">" => Operator::Gt,
        "==" => Operator::Eq,
        "!=" => Operator::Neq,
        "band" => Operator::Band,
        "bxor" => Operator::Bxor,
        "bor" => Operator::Bor,
        "<<" => Operator::LeftShift,
        ">>" => Operator::RightShift,
        "and" => Operator::And,
        "xor" => Operator::Xor,
        "or" => Operator::Or,
        "bnand" => unimplemented!(),
        "bxnor" => unimplemented!(),
        "bnor" => unimplemented!(),
        "nand" => unimplemented!(),
        "xnor" => unimplemented!(),
        "nor" => unimplemented!(),
        _ => unreachable!(),
    }
}

fn parse_float(input: &str) -> f64 {
    input
        .replace("_", "")
        .parse()
        .expect("Grammar requires valid float.")
}

fn parse_dec_int(input: &str) -> i64 {
    input
        .replace("_", "")
        .parse()
        .expect("Grammar requires valid int.")
}

fn parse_hex_int(input: &str) -> i64 {
    // Slice trims off 0x at beginning.
    i64::from_str_radix(&input.replace("_", "")[2..], 16)
        .expect("Grammar requires valid hexadecimal int.")
}

fn parse_oct_int(input: &str) -> i64 {
    // Slice trims off 0o at beginning.
    i64::from_str_radix(&input.replace("_", "")[2..], 8).expect("Grammar requires valid octal int.")
}

fn parse_legacy_oct_int(input: &str) -> i64 {
    // Slice trims off 0 at beginning.
    i64::from_str_radix(&input.replace("_", "")[1..], 8).expect("Grammar requires valid octal int.")
}

fn parse_bin_int(input: &str) -> i64 {
    // Slice trims off 0b at beginning.
    i64::from_str_radix(&input.replace("_", "")[2..], 2)
        .expect("Grammar requires valid binary int.")
}

impl<'a> VagueIngester<'a> {
    pub(super) fn convert_literal(
        &mut self,
        node: i::Node,
    ) -> Result<o::VPExpression, CompileProblem> {
        debug_assert!(node.as_rule() == i::Rule::literal);
        let position = self.make_position(&node);
        let child = node.into_inner().next().expect("bad AST");

        let value = match child.as_rule() {
            i::Rule::bin_int => o::KnownData::Int(parse_bin_int(child.as_str())),
            i::Rule::oct_int => o::KnownData::Int(parse_oct_int(child.as_str())),
            i::Rule::dec_int => o::KnownData::Int(parse_dec_int(child.as_str())),
            i::Rule::hex_int => o::KnownData::Int(parse_hex_int(child.as_str())),
            i::Rule::legacy_oct_int => o::KnownData::Int(parse_legacy_oct_int(child.as_str())),
            i::Rule::float => o::KnownData::Float(parse_float(child.as_str())),
            _ => unreachable!("bad AST"),
        };
        Ok(o::VPExpression::Literal(value, position))
    }

    pub(super) fn convert_macro_call_input_list(
        &mut self,
        node: i::Node,
    ) -> Result<Vec<o::VPExpression>, CompileProblem> {
        debug_assert!(node.as_rule() == i::Rule::macro_call_input_list);
        let mut inputs = Vec::new();
        for child in node.into_inner() {
            inputs.push(self.convert_vpe(child)?);
        }
        Ok(inputs)
    }

    pub(super) fn convert_macro_call_output_list(
        &mut self,
        node: i::Node,
    ) -> Result<Vec<o::FuncCallOutput>, CompileProblem> {
        debug_assert!(node.as_rule() == i::Rule::macro_call_output_list);
        let mut outputs = Vec::new();
        for child in node.into_inner() {
            let output = match child.as_rule() {
                i::Rule::vce => o::FuncCallOutput::VCExpression(Box::new(self.convert_vce(child)?)),
                i::Rule::inline_output => {
                    o::FuncCallOutput::InlineReturn(self.make_position(&child))
                }
                _ => unreachable!("bad AST"),
            };
            outputs.push(output);
        }
        Ok(outputs)
    }

    pub(super) fn convert_macro_call(
        &mut self,
        node: i::Node,
        require_inline_output: bool,
    ) -> Result<o::VPExpression, CompileProblem> {
        debug_assert!(node.as_rule() == i::Rule::macro_call);
        let position = self.make_position(&node);
        let mut children = node.into_inner();
        let name_node = children.next().expect("bad AST");
        let input_list_node = children.next().expect("bad AST");
        let maybe_output_list_node = children.next();

        let input_list = self.convert_macro_call_input_list(input_list_node)?;
        let output_list = if let Some(output_list_node) = maybe_output_list_node {
            let output_list_position = self.make_position(&output_list_node);
            let output_list = self.convert_macro_call_output_list(output_list_node)?;
            let num_inline_outputs = output_list.iter().fold(0, |counter, item| {
                if let o::FuncCallOutput::InlineReturn(..) = item {
                    counter + 1
                } else {
                    counter
                }
            });
            if num_inline_outputs > 1 {
                return Err(problems::too_many_inline_returns(
                    position,
                    output_list_position,
                    num_inline_outputs,
                ));
            }
            if require_inline_output && num_inline_outputs != 1 {
                return Err(problems::missing_inline_return(
                    position,
                    output_list_position,
                ));
            }
            output_list
        } else if require_inline_output {
            vec![o::FuncCallOutput::InlineReturn(position.clone())]
        } else {
            vec![]
        };

        Ok(o::VPExpression::MacroCall {
            mcro: Box::new(o::VPExpression::Variable(
                self.lookup_identifier(&name_node)?,
                self.make_position(&name_node),
            )),
            inputs: input_list,
            outputs: output_list,
            position,
        })
    }

    pub(super) fn convert_vp_var(
        &mut self,
        node: i::Node,
    ) -> Result<o::VPExpression, CompileProblem> {
        debug_assert!(node.as_rule() == i::Rule::vp_var);
        let position = self.make_position(&node);
        let child = node.into_inner().next().expect("bad AST");
        let var_id = self.lookup_identifier(&child)?;
        Ok(o::VPExpression::Variable(var_id, position))
    }

    pub(super) fn convert_build_array(
        &mut self,
        node: i::Node,
    ) -> Result<o::VPExpression, CompileProblem> {
        debug_assert!(node.as_rule() == i::Rule::build_array);
        let position = self.make_position(&node);
        let mut values = Vec::new();
        for child in node.into_inner() {
            values.push(self.convert_vpe(child)?);
        }
        Ok(o::VPExpression::Collect(values, position))
    }

    pub(super) fn convert_build_type_bound(
        &mut self,
        node: i::Node,
    ) -> Result<o::VPExpression, CompileProblem> {
        debug_assert!(node.as_rule() == i::Rule::build_type_bound);
        let position = self.make_position(&node);
        let mut children = node.into_inner();
        let first_value = children.next();
        let second_value = children.next();
        debug_assert!(children.next().is_none());
        Ok(match (first_value, second_value) {
            (None, None) => o::VPExpression::UnboundedTypeBound(position),
            (Some(upper_bound), None) => o::VPExpression::TypeBound {
                lower: None,
                upper: Box::new(self.convert_vpe_part_3(upper_bound)?),
                position,
            },
            (Some(lower_bound), Some(upper_bound)) => o::VPExpression::TypeBound {
                lower: Some(Box::new(self.convert_vpe_part_3(lower_bound)?)),
                upper: Box::new(self.convert_vpe_part_3(upper_bound)?),
                position,
            },
            (None, Some(..)) => unreachable!("Encountered item after iterator ended."),
        })
    }

    pub(super) fn convert_vpe_part_1(
        &mut self,
        node: i::Node,
    ) -> Result<o::VPExpression, CompileProblem> {
        debug_assert!(node.as_rule() == i::Rule::vpe_part_1);
        let child = node.into_inner().next().expect("bad AST");
        match child.as_rule() {
            i::Rule::literal => self.convert_literal(child),
            i::Rule::macro_call => self.convert_macro_call(child, true),
            i::Rule::vp_var => self.convert_vp_var(child),
            i::Rule::vpe => self.convert_vpe(child),
            i::Rule::build_array => self.convert_build_array(child),
            i::Rule::build_type_bound => self.convert_build_type_bound(child),
            _ => unreachable!("bad AST"),
        }
    }

    pub(super) fn convert_negate(
        &mut self,
        node: i::Node,
    ) -> Result<o::VPExpression, CompileProblem> {
        debug_assert!(node.as_rule() == i::Rule::negate);
        let position = self.make_position(&node);
        let child = node.into_inner().next().expect("bad AST");
        let base = self.convert_vpe_part_2(child)?;
        Ok(o::VPExpression::UnaryOperation(
            o::UnaryOperator::Negate,
            Box::new(base),
            position,
        ))
    }

    pub(super) fn convert_not(&mut self, node: i::Node) -> Result<o::VPExpression, CompileProblem> {
        debug_assert!(node.as_rule() == i::Rule::not);
        let position = self.make_position(&node);
        let child = node.into_inner().next().expect("bad AST");
        let base = self.convert_vpe_part_2(child)?;
        Ok(o::VPExpression::UnaryOperation(
            o::UnaryOperator::Not,
            Box::new(base),
            position,
        ))
    }

    pub(super) fn convert_build_array_type(
        &mut self,
        node: i::Node,
    ) -> Result<o::VPExpression, CompileProblem> {
        debug_assert!(node.as_rule() == i::Rule::build_array_type);
        let position = self.make_position(&node);

        let mut dimensions = Vec::new();
        for child in node.into_inner() {
            match child.as_rule() {
                i::Rule::vpe => dimensions.push(self.convert_vpe(child)?),
                i::Rule::vpe_part_1 => {
                    let base = self.convert_vpe_part_1(child)?;
                    return Ok(o::VPExpression::BuildArrayType {
                        dimensions,
                        base: Box::new(base),
                        position,
                    });
                }
                _ => unreachable!("bad AST"),
            }
        }
        unreachable!("bad AST")
    }

    pub(super) fn convert_vp_index(
        &mut self,
        node: i::Node,
    ) -> Result<o::VPExpression, CompileProblem> {
        debug_assert!(node.as_rule() == i::Rule::vp_index);
        let position = self.make_position(&node);
        let mut children = node.into_inner();

        let base_node = children.next().expect("bad AST");
        let base = self.convert_vpe_part_1(base_node)?;
        let mut indexes = Vec::new();
        for child in children {
            if child.as_rule() == i::Rule::vpe {
                indexes.push((self.convert_vpe(child)?, false));
            } else if child.as_rule() == i::Rule::optional_index_indicator {
                // Turns out the previous index is actually optional.
                let last = indexes.len() - 1;
                indexes[last].1 = true;
            } else {
                unreachable!("bad AST");
            }
        }

        Ok(o::VPExpression::Index {
            base: Box::new(base),
            indexes,
            position,
        })
    }

    pub(super) fn convert_vpe_part_2(
        &mut self,
        node: i::Node,
    ) -> Result<o::VPExpression, CompileProblem> {
        debug_assert!(node.as_rule() == i::Rule::vpe_part_2);
        let child = node.into_inner().next().expect("bad AST");
        match child.as_rule() {
            i::Rule::build_array_type => self.convert_build_array_type(child),
            i::Rule::vp_index => self.convert_vp_index(child),
            i::Rule::vpe_part_1 => self.convert_vpe_part_1(child),
            _ => unreachable!("bad AST"),
        }
    }

    fn convert_get_property(&mut self, node: i::Node) -> Result<o::VPExpression, CompileProblem> {
        debug_assert!(node.as_rule() == i::Rule::get_property);
        let pos = FilePosition::from_pair(&node, self.current_file_id);
        let mut children = node.into_inner();
        let vpe = self.convert_vpe_part_2(children.next().expect("bad AST"))?;
        let prop_node = children.next().expect("bad AST");
        let prop = o::Property::from_str(prop_node.as_str()).map_err(|_| {
            problems::bad_property_name(FilePosition::from_pair(&prop_node, self.current_file_id))
        })?;
        Ok(o::VPExpression::UnaryOperation(
            o::UnaryOperator::PropertyAccess(prop),
            Box::new(vpe),
            pos,
        ))
    }

    pub(super) fn convert_vpe_part_3(
        &mut self,
        node: i::Node,
    ) -> Result<o::VPExpression, CompileProblem> {
        debug_assert!(node.as_rule() == i::Rule::vpe_part_3);
        let child = node.into_inner().next().expect("bad AST");
        match child.as_rule() {
            i::Rule::negate => self.convert_negate(child),
            i::Rule::not => self.convert_not(child),
            i::Rule::get_property => self.convert_get_property(child),
            i::Rule::vpe_part_2 => self.convert_vpe_part_2(child),
            _ => unreachable!("bad AST"),
        }
    }

    pub(super) fn convert_vpe(&mut self, node: i::Node) -> Result<o::VPExpression, CompileProblem> {
        debug_assert!(node.as_rule() == i::Rule::vpe);
        let mut operator_stack = Vec::with_capacity(64);
        let mut operand_stack = Vec::with_capacity(64);
        operator_stack.push(Operator::Sentinel);

        for child in node.into_inner() {
            match child.as_rule() {
                i::Rule::vpe_part_3 => {
                    let result = self.convert_vpe_part_3(child)?;
                    operand_stack.push(result);
                }
                i::Rule::operator => {
                    let op_str = child.as_str();
                    let new_operator = op_str_to_operator(op_str);
                    // Shunting yard algorithm.
                    loop {
                        let top_op_prec = operator_stack.last().unwrap().precedence();
                        if new_operator.precedence() > top_op_prec
                            || (new_operator.precedence() == top_op_prec
                                && new_operator.right_associative())
                        {
                            operator_stack.push(new_operator);
                            break;
                        } else {
                            let top_operator = operator_stack.pop().unwrap();
                            let rhs = operand_stack.pop().unwrap();
                            let lhs = operand_stack.pop().unwrap();
                            let mut position = lhs.clone_position();
                            position.include_other(&rhs.clone_position());
                            operand_stack.push(o::VPExpression::BinaryOperation(
                                Box::new(lhs),
                                top_operator.bin_op(),
                                Box::new(rhs),
                                position,
                            ));
                        }
                    }
                }
                _ => unreachable!("bad AST"),
            }
        }

        // If we have leftover operators, we need to do some more looping to get rid of them. The
        // shunting yard algorithm used above guarantees that we can just loop through and compose
        // them in order because they are already in the correct order of precedence.
        // We start from 1 not 0 because we don't want to pop the sentinel.
        for _ in 1..operator_stack.len() {
            let top_operator = operator_stack.pop().unwrap();
            let rhs = operand_stack.pop().unwrap();
            let lhs = operand_stack.pop().unwrap();
            let mut position = lhs.clone_position();
            position.include_other(&rhs.clone_position());
            operand_stack.push(o::VPExpression::BinaryOperation(
                Box::new(lhs),
                top_operator.bin_op(),
                Box::new(rhs),
                position,
            ));
        }

        Ok(operand_stack.pop().unwrap())
    }
}
