use super::ScopeResolver;
use crate::high_level::problem::{CompileProblem, FilePosition};
use crate::resolved::structure as o;
use crate::vague::structure as i;

use std::convert::TryInto;

impl<'a> ScopeResolver<'a> {
    pub fn value_bound_error_helper(
        value_pos: FilePosition,
        usage_pos: FilePosition,
        value_type: &i::DataType,
        must_fit_in: &i::DataType,
    ) -> Result<(), CompileProblem> {
        let lower_bound = must_fit_in.min();
        let value_min = value_type.min();
        if let (Some(bound), Some(min)) = (&lower_bound, &value_min) {
            if Self::biggest_specific_type(bound, min).as_ref() != Ok(min) {
                return Err(super::problems::value_too_small(
                    value_pos, usage_pos, min, bound,
                ));
            }
        }
        let upper_bound = must_fit_in.max();
        let value_max = value_type.max();
        if let (Some(bound), Some(max)) = (upper_bound, value_max) {
            if Self::biggest_specific_type(bound, max).as_ref() != Ok(bound) {
                return Err(super::problems::value_too_big(
                    value_pos, usage_pos, max, bound,
                ));
            }
        }
        Ok(())
    }

    pub(super) fn resolve_data_type(dtype: &i::SpecificDataType) -> Option<o::DataType> {
        match dtype {
            i::SpecificDataType::Array(len, base) => {
                Self::resolve_data_type(base).map(|base| o::DataType::Array(*len, Box::new(base)))
            }
            i::SpecificDataType::Bool => Some(o::DataType::Bool),
            i::SpecificDataType::Int => Some(o::DataType::Int),
            i::SpecificDataType::Float => Some(o::DataType::Float),
            _ => None,
        }
    }

    pub(super) fn compute_unary_operation(
        operator: i::UnaryOperator,
        data: &i::KnownData,
    ) -> i::KnownData {
        if let i::KnownData::Array(items) = data {
            i::KnownData::Array(
                items
                    .iter()
                    .map(|item| Self::compute_unary_operation(operator, item))
                    .collect(),
            )
        } else {
            Self::compute_unary_operation_impl(operator, data)
        }
    }

    fn compute_unary_operation_impl(
        operator: i::UnaryOperator,
        data: &i::KnownData,
    ) -> i::KnownData {
        match operator {
            i::UnaryOperator::Not => match data {
                i::KnownData::Bool(value) => i::KnownData::Bool(!*value),
                _ => unreachable!(),
            },
            i::UnaryOperator::BNot => match data {
                i::KnownData::Int(value) => i::KnownData::Int(!*value),
                _ => unreachable!(),
            },
            i::UnaryOperator::Negate => match data {
                i::KnownData::Int(value) => i::KnownData::Int(-*value),
                i::KnownData::Float(value) => i::KnownData::Float(-*value),
                _ => unreachable!(),
            },
            i::UnaryOperator::Reciprocal => match data {
                i::KnownData::Float(value) => i::KnownData::Float(1.0 / *value),
                _ => unreachable!(),
            },
            i::UnaryOperator::Sine => i::KnownData::Float(data.require_float().sin()),
            i::UnaryOperator::Cosine => i::KnownData::Float(data.require_float().sin()),
            i::UnaryOperator::SquareRoot => i::KnownData::Float(data.require_float().sqrt()),
            i::UnaryOperator::Exp => i::KnownData::Float(data.require_float().exp()),
            i::UnaryOperator::Exp2 => i::KnownData::Float(data.require_float().exp2()),
            i::UnaryOperator::Log => i::KnownData::Float(data.require_float().ln()),
            i::UnaryOperator::Log10 => i::KnownData::Float(data.require_float().log10()),
            i::UnaryOperator::Log2 => i::KnownData::Float(data.require_float().log2()),
            i::UnaryOperator::Absolute => match data {
                i::KnownData::Int(value) => i::KnownData::Int(value.abs()),
                i::KnownData::Float(value) => i::KnownData::Float(value.abs()),
                _ => unreachable!(),
            },
            i::UnaryOperator::Floor => i::KnownData::Float(data.require_float().floor()),
            i::UnaryOperator::Ceiling => i::KnownData::Float(data.require_float().ceil()),
            i::UnaryOperator::Truncate => i::KnownData::Float(data.require_float().trunc()),

            i::UnaryOperator::Ftoi => i::KnownData::Int(data.require_float() as i64),
            i::UnaryOperator::Itof => i::KnownData::Float(data.require_int() as f64),
            i::UnaryOperator::PropertyAccess(..) => unreachable!("Should be handled elsewhere."),
        }
    }

    /// Expression must be a binary operator expression (add, equals, etc.) and A and B must be valid
    /// inputs for that expression. They cannot have different base types.
    pub(super) fn compute_binary_operation(
        a: &i::KnownData,
        operator: i::BinaryOperator,
        b: &i::KnownData,
    ) -> i::KnownData {
        if let i::KnownData::Array(array_a) = a {
            if let i::KnownData::Array(array_b) = b {
                let a_size = array_a.len();
                let b_size = array_b.len();

                let (inc_a, inc_b) = if a_size == b_size {
                    (true, true)
                } else if a_size == 1 {
                    (false, true)
                } else if b_size == 1 {
                    (true, false)
                } else {
                    unreachable!("Invalid inflation should be handled earlier.");
                };

                let result_size = a_size.max(b_size);
                let mut result_items = Vec::with_capacity(result_size);
                let mut a_index = 0;
                let mut b_index = 0;
                for _ in 0..result_size {
                    // Do some math, finally.
                    result_items.push(Self::compute_binary_operation(
                        &array_a[a_index],
                        operator,
                        &array_b[b_index],
                    ));

                    // Update the index for the next go-around.
                    if inc_a {
                        a_index += 1;
                    }
                    if inc_b {
                        b_index += 1;
                    }
                }
                i::KnownData::Array(result_items)
            } else {
                let a_size = array_a.len();
                let mut items = Vec::with_capacity(a_size);
                for a_item in array_a {
                    items.push(Self::compute_binary_operation_impl(a_item, operator, b));
                }
                i::KnownData::Array(items)
            }
        } else {
            if let i::KnownData::Array(array_b) = b {
                let b_size = array_b.len();
                let mut items = Vec::with_capacity(b_size);
                for b_item in array_b {
                    items.push(Self::compute_binary_operation_impl(a, operator, b_item));
                }
                i::KnownData::Array(items)
            } else {
                Self::compute_binary_operation_impl(a, operator, b)
            }
        }
    }

    fn compute_binary_operation_impl(
        a: &i::KnownData,
        operator: i::BinaryOperator,
        b: &i::KnownData,
    ) -> i::KnownData {
        match operator {
            i::BinaryOperator::Add => match a {
                i::KnownData::Bool(..) => unimplemented!(),
                i::KnownData::Int(value) => i::KnownData::Int(value + b.require_int()),
                i::KnownData::Float(value) => i::KnownData::Float(value + b.require_float()),
                i::KnownData::DataType(dta) => i::KnownData::DataType(
                    Self::biggest_type(&dta, b.require_data_type()).expect("TODO: Nice error."),
                ),
                _ => unreachable!(),
            },
            i::BinaryOperator::Subtract => match a {
                i::KnownData::Bool(..) => unimplemented!(),
                i::KnownData::Int(value) => i::KnownData::Int(value - b.require_int()),
                i::KnownData::Float(value) => i::KnownData::Float(value - b.require_float()),
                _ => unreachable!(),
            },
            i::BinaryOperator::Multiply => match a {
                i::KnownData::Bool(..) => unimplemented!(),
                i::KnownData::Int(value) => i::KnownData::Int(value * b.require_int()),
                i::KnownData::Float(value) => i::KnownData::Float(value * b.require_float()),
                _ => unreachable!(),
            },
            i::BinaryOperator::Divide => match a {
                i::KnownData::Float(value) => i::KnownData::Float(value / b.require_float()),
                i::KnownData::Int(value) => i::KnownData::Int(value / b.require_int()),
                _ => unreachable!(),
            },
            i::BinaryOperator::Modulo => match a {
                i::KnownData::Bool(..) => unimplemented!(),
                i::KnownData::Int(value) => i::KnownData::Int(value % b.require_int()),
                i::KnownData::Float(value) => i::KnownData::Float(value % b.require_float()),
                _ => unreachable!(),
            },
            i::BinaryOperator::Power => match a {
                i::KnownData::Bool(..) => unimplemented!(),
                i::KnownData::Int(value) => {
                    i::KnownData::Int(i64::pow(*value, b.require_int().try_into().unwrap()))
                }
                i::KnownData::Float(value) => i::KnownData::Float(value.powf(b.require_float())),
                _ => unreachable!(),
            },
            i::BinaryOperator::And => i::KnownData::Bool(a.require_bool() && b.require_bool()),
            i::BinaryOperator::Or => i::KnownData::Bool(a.require_bool() || b.require_bool()),
            i::BinaryOperator::Xor => i::KnownData::Bool(a.require_bool() != b.require_bool()),
            i::BinaryOperator::BAnd => i::KnownData::Int(a.require_int() & b.require_int()),
            i::BinaryOperator::BOr => i::KnownData::Int(a.require_int() | b.require_int()),
            i::BinaryOperator::BXor => i::KnownData::Int(a.require_int() ^ b.require_int()),
            i::BinaryOperator::LeftShift => i::KnownData::Int(a.require_int() << b.require_int()),
            i::BinaryOperator::RightShift => i::KnownData::Int(a.require_int() >> b.require_int()),
            i::BinaryOperator::Equal => match a {
                i::KnownData::Bool(value) => i::KnownData::Bool(*value == b.require_bool()),
                i::KnownData::Int(value) => i::KnownData::Bool(*value == b.require_int()),
                i::KnownData::Float(value) => i::KnownData::Bool(*value == b.require_float()),
                i::KnownData::DataType(value) => i::KnownData::Bool(value == b.require_data_type()),
                i::KnownData::Macro(value) => i::KnownData::Bool(value == b.require_macro()),
                i::KnownData::Array(value) => i::KnownData::Bool(value == b.require_array()),
                _ => unreachable!(),
            },
            i::BinaryOperator::NotEqual => match a {
                i::KnownData::Bool(value) => i::KnownData::Bool(*value != b.require_bool()),
                i::KnownData::Int(value) => i::KnownData::Bool(*value != b.require_int()),
                i::KnownData::Float(value) => i::KnownData::Bool(*value != b.require_float()),
                i::KnownData::DataType(value) => i::KnownData::Bool(value != b.require_data_type()),
                i::KnownData::Macro(value) => i::KnownData::Bool(value != b.require_macro()),
                i::KnownData::Array(value) => i::KnownData::Bool(value != b.require_array()),
                _ => unreachable!(),
            },
            i::BinaryOperator::LessThan => match a {
                i::KnownData::Int(value) => i::KnownData::Bool(*value < b.require_int()),
                i::KnownData::Float(value) => i::KnownData::Bool(*value < b.require_float()),
                _ => unreachable!(),
            },
            i::BinaryOperator::GreaterThan => match a {
                i::KnownData::Int(value) => i::KnownData::Bool(*value > b.require_int()),
                i::KnownData::Float(value) => i::KnownData::Bool(*value > b.require_float()),
                _ => unreachable!(),
            },
            i::BinaryOperator::LessThanOrEqual => match a {
                i::KnownData::Int(value) => i::KnownData::Bool(*value <= b.require_int()),
                i::KnownData::Float(value) => i::KnownData::Bool(*value <= b.require_float()),
                i::KnownData::DataType(smaller) => {
                    let bigger = b.require_data_type().clone();
                    i::KnownData::Bool(Self::biggest_type(smaller, &bigger) == Ok(bigger))
                }
                _ => unreachable!(),
            },
            i::BinaryOperator::GreaterThanOrEqual => match a {
                i::KnownData::Int(value) => i::KnownData::Bool(*value >= b.require_int()),
                i::KnownData::Float(value) => i::KnownData::Bool(*value >= b.require_float()),
                i::KnownData::DataType(typ) => i::KnownData::Bool(
                    Self::biggest_type(typ, b.require_data_type()) == Ok(typ.clone()),
                ),
                _ => unreachable!(),
            },
        }
    }

    /// Returns Result::Err if there is no biggest type.
    pub(super) fn biggest_type(a: &i::DataType, b: &i::DataType) -> Result<i::DataType, ()> {
        fn biggest_option_type(
            a: Option<&i::SpecificDataType>,
            b: Option<&i::SpecificDataType>,
        ) -> Result<Option<i::SpecificDataType>, ()> {
            if a.is_none() {
                Ok(b.cloned())
            } else if b.is_none() {
                Ok(a.cloned())
            } else {
                ScopeResolver::biggest_specific_type(a.unwrap(), b.unwrap()).map(|sdt| Some(sdt))
            }
        }
        let (au, al) = a.bounds.as_tuple();
        let (bu, bl) = b.bounds.as_tuple();
        let u = biggest_option_type(au, bu)?;
        let l = biggest_option_type(al, bl)?;
        let actual_type = biggest_option_type(a.actual_type.as_ref(), b.actual_type.as_ref())?;
        Ok(i::DataType {
            actual_type,
            bounds: i::Bounds::from_tuple((l, u)),
        })
    }

    /// Returns Result::Err if there is no biggest type.
    pub(super) fn biggest_specific_type(
        a: &i::SpecificDataType,
        b: &i::SpecificDataType,
    ) -> Result<i::SpecificDataType, ()> {
        // BCT rule 2
        if a == b {
            Ok(a.clone())
        // BCT rules 3 & 4
        } else if let (
            i::SpecificDataType::Array(alen, abase),
            i::SpecificDataType::Array(blen, bbase),
        ) = (a, b)
        {
            // BCT rule 3
            if alen == blen {
                Ok(i::SpecificDataType::Array(
                    *alen,
                    Box::new(Self::biggest_specific_type(abase, bbase)?),
                ))
            // BCT rule 4
            } else if *alen == 1 {
                Ok(i::SpecificDataType::Array(
                    *blen,
                    Box::new(Self::biggest_specific_type(abase, bbase)?),
                ))
            } else if *blen == 1 {
                Ok(i::SpecificDataType::Array(
                    *alen,
                    Box::new(Self::biggest_specific_type(abase, bbase)?),
                ))
            } else {
                Err(())
            }
        // BCT rule 5
        } else if let i::SpecificDataType::Array(alen, abase) = a {
            Ok(i::SpecificDataType::Array(
                *alen,
                Box::new(Self::biggest_specific_type(abase, b)?),
            ))
        } else if let i::SpecificDataType::Array(blen, bbase) = b {
            Ok(i::SpecificDataType::Array(
                *blen,
                Box::new(Self::biggest_specific_type(a, bbase)?),
            ))
        } else {
            Err(())
        }
    }

    pub(super) fn resolve_known_data(input: &i::KnownData) -> Result<o::KnownData, ()> {
        Result::Ok(match input {
            i::KnownData::Bool(value) => o::KnownData::Bool(*value),
            i::KnownData::Int(value) => o::KnownData::Int(*value),
            i::KnownData::Float(value) => o::KnownData::Float(*value),
            i::KnownData::Array(old_data) => {
                let mut items = Vec::with_capacity(old_data.len());
                for old_item in old_data {
                    items.push(Self::resolve_known_data(old_item)?);
                }
                o::KnownData::Array(items)
            }
            i::KnownData::DataType(..) | i::KnownData::Macro(..) | i::KnownData::Void => {
                return Result::Err(())
            }
        })
    }

    pub(super) fn resolve_operator(operator: i::BinaryOperator) -> o::BinaryOperator {
        match operator {
            i::BinaryOperator::Add => o::BinaryOperator::Add,
            i::BinaryOperator::And => o::BinaryOperator::And,
            i::BinaryOperator::BAnd => o::BinaryOperator::BAnd,
            i::BinaryOperator::BOr => o::BinaryOperator::BOr,
            i::BinaryOperator::BXor => o::BinaryOperator::BXor,
            i::BinaryOperator::LeftShift => o::BinaryOperator::LeftShift,
            i::BinaryOperator::RightShift => o::BinaryOperator::RightShift,
            i::BinaryOperator::Divide => o::BinaryOperator::Divide,
            i::BinaryOperator::Equal => o::BinaryOperator::Equal,
            i::BinaryOperator::GreaterThan => o::BinaryOperator::GreaterThan,
            i::BinaryOperator::GreaterThanOrEqual => o::BinaryOperator::GreaterThanOrEqual,
            i::BinaryOperator::LessThan => o::BinaryOperator::LessThan,
            i::BinaryOperator::LessThanOrEqual => o::BinaryOperator::LessThanOrEqual,
            i::BinaryOperator::Modulo => o::BinaryOperator::Modulo,
            i::BinaryOperator::Multiply => o::BinaryOperator::Multiply,
            i::BinaryOperator::NotEqual => o::BinaryOperator::NotEqual,
            i::BinaryOperator::Or => o::BinaryOperator::Or,
            i::BinaryOperator::Power => o::BinaryOperator::Power,
            i::BinaryOperator::Subtract => o::BinaryOperator::Subtract,
            i::BinaryOperator::Xor => o::BinaryOperator::Xor,
        }
    }
}
