//! Functions for converting each kind of trivial instruction.

use super::Converter;
use crate::trivial::structure as i;
use inkwell::{values::BasicValueEnum, FloatPredicate, IntPredicate};
use shared_util::prelude::*;

const UNNAMED: &str = "";

impl<'i, 'ctx> Converter<'i, 'ctx> {
    pub fn convert_unary_expression(&mut self, op: &i::UnaryOperator, a: &i::Value, x: &i::Value) {
        let dimensions: Vec<_> = x.dimensions.iter().map(|(len, _)| *len).collect();
        if dimensions.len() == 0 {
            let ar = self.load_value(a, &[]);
            let xr = self.do_unary_op(op, ar);
            self.store_value(x, xr, &[]);
        } else {
            let loops: Vec<_> = dimensions.iter().map(|_| self.start_loop()).collect();
            let mut loop_counters: Vec<_> = loops.iter().map(|(_, _, counter)| *counter).collect();
            loop_counters.insert(0, self.i32_const(0));
            let ar = self.load_value_dyn(a, &loop_counters[..]);
            let xr = self.do_unary_op(op, ar);
            self.store_value_dyn(x, xr, &loop_counters[..]);
            for (size, loop_params) in dimensions.into_iter().zip(loops.into_iter()).rev() {
                self.end_loop(size as i32, loop_params);
            }
        }
    }

    pub fn do_unary_op(
        &mut self,
        op: &i::UnaryOperator,
        ar: BasicValueEnum<'ctx>,
    ) -> BasicValueEnum<'ctx> {
        match op {
            i::UnaryOperator::BNot => self
                .builder
                .build_xor(ar.into_int_value(), self.u32_const(0xFFFFFFFF), UNNAMED)
                .into(),
            i::UnaryOperator::FAbs => self.build_call(self.intrinsics.fabs_f32, &mut [ar]),
            i::UnaryOperator::FCeil => self.build_call(self.intrinsics.ceil_f32, &mut [ar]),
            i::UnaryOperator::FCos => self.build_call(self.intrinsics.cos_f32, &mut [ar]),
            i::UnaryOperator::FExp => self.build_call(self.intrinsics.exp_f32, &mut [ar]),
            i::UnaryOperator::FExp2 => self.build_call(self.intrinsics.exp2_f32, &mut [ar]),
            i::UnaryOperator::FFloor => self.build_call(self.intrinsics.floor_f32, &mut [ar]),
            i::UnaryOperator::FLog => self.build_call(self.intrinsics.log_f32, &mut [ar]),
            i::UnaryOperator::FLog10 => self.build_call(self.intrinsics.log10_f32, &mut [ar]),
            i::UnaryOperator::FLog2 => self.build_call(self.intrinsics.log2_f32, &mut [ar]),
            i::UnaryOperator::FSin => self.build_call(self.intrinsics.sin_f32, &mut [ar]),
            i::UnaryOperator::FSqrt => self.build_call(self.intrinsics.sqrt_f32, &mut [ar]),
            i::UnaryOperator::FTrunc => self.build_call(self.intrinsics.trunc_f32, &mut [ar]),
            i::UnaryOperator::IAbs => unimplemented!(),
            i::UnaryOperator::NegF => self
                .builder
                .build_float_sub(self.f32_const(0.0), ar.into_float_value(), UNNAMED)
                .into(),
            i::UnaryOperator::NegI => self
                .builder
                .build_int_sub(self.i32_const(0), ar.into_int_value(), UNNAMED)
                .into(),
            i::UnaryOperator::Not => self
                .builder
                .build_xor(ar.into_int_value(), self.b1_const(true), UNNAMED)
                .into(),
            i::UnaryOperator::Ftoi => self
                .builder
                .build_float_to_signed_int(ar.into_float_value(), self.context.i32_type(), UNNAMED)
                .into(),
            i::UnaryOperator::Itof => self
                .builder
                .build_signed_int_to_float(ar.into_int_value(), self.context.f32_type(), UNNAMED)
                .into(),
        }
    }

    pub fn convert_binary_expression(
        &mut self,
        op: &i::BinaryOperator,
        a: &i::Value,
        b: &i::Value,
        x: &i::Value,
    ) {
        let dimensions: Vec<_> = x.dimensions.iter().map(|(len, _)| *len).collect();
        if dimensions.len() == 0 {
            let ar = self.load_value(a, &[]);
            let br = self.load_value(b, &[]);
            let xr = self.do_binary_op(op, ar, br);
            self.store_value(x, xr, &[]);
        } else {
            let loops: Vec<_> = dimensions.iter().map(|_| self.start_loop()).collect();
            let mut loop_counters: Vec<_> = loops.iter().map(|(_, _, counter)| *counter).collect();
            loop_counters.insert(0, self.i32_const(0));
            let ar = self.load_value_dyn(a, &loop_counters[..]);
            let br = self.load_value_dyn(b, &loop_counters[..]);
            let xr = self.do_binary_op(op, ar, br);
            self.store_value_dyn(x, xr, &loop_counters[..]);
            for (size, loop_params) in dimensions.into_iter().zip(loops.into_iter()).rev() {
                self.end_loop(size as i32, loop_params);
            }
        }
    }

    pub fn do_binary_op(
        &mut self,
        op: &i::BinaryOperator,
        ar: BasicValueEnum<'ctx>,
        br: BasicValueEnum<'ctx>,
    ) -> BasicValueEnum<'ctx> {
        match op {
            i::BinaryOperator::AddI => self
                .builder
                .build_int_add(ar.into_int_value(), br.into_int_value(), UNNAMED)
                .into(),
            i::BinaryOperator::SubI => self
                .builder
                .build_int_sub(ar.into_int_value(), br.into_int_value(), UNNAMED)
                .into(),
            i::BinaryOperator::MulI => self
                .builder
                .build_int_mul(ar.into_int_value(), br.into_int_value(), UNNAMED)
                .into(),
            i::BinaryOperator::DivI => self
                .builder
                .build_int_signed_div(ar.into_int_value(), br.into_int_value(), UNNAMED)
                .into(),
            i::BinaryOperator::ModI => self
                .builder
                .build_int_signed_rem(ar.into_int_value(), br.into_int_value(), UNNAMED)
                .into(),
            i::BinaryOperator::AddF => self
                .builder
                .build_float_add(ar.into_float_value(), br.into_float_value(), UNNAMED)
                .into(),
            i::BinaryOperator::SubF => self
                .builder
                .build_float_sub(ar.into_float_value(), br.into_float_value(), UNNAMED)
                .into(),
            i::BinaryOperator::MulF => self
                .builder
                .build_float_mul(ar.into_float_value(), br.into_float_value(), UNNAMED)
                .into(),
            i::BinaryOperator::DivF => self
                .builder
                .build_float_div(ar.into_float_value(), br.into_float_value(), UNNAMED)
                .into(),
            i::BinaryOperator::ModF => self
                .builder
                .build_float_rem(ar.into_float_value(), br.into_float_value(), UNNAMED)
                .into(),
            i::BinaryOperator::PowF => self.build_call(self.intrinsics.pow_f32, &mut [ar, br]),
            i::BinaryOperator::CompI(condition) => {
                let predicate = match condition {
                    i::Condition::Equal => IntPredicate::EQ,
                    i::Condition::NotEqual => IntPredicate::NE,
                    i::Condition::GreaterThan => IntPredicate::SGT,
                    i::Condition::GreaterThanOrEqual => IntPredicate::SGE,
                    i::Condition::LessThan => IntPredicate::SLT,
                    i::Condition::LessThanOrEqual => IntPredicate::SLE,
                };
                self.builder
                    .build_int_compare(predicate, ar.into_int_value(), br.into_int_value(), UNNAMED)
                    .into()
            }
            i::BinaryOperator::CompF(condition) => {
                let predicate = match condition {
                    i::Condition::Equal => FloatPredicate::OEQ,
                    i::Condition::NotEqual => FloatPredicate::ONE,
                    i::Condition::GreaterThan => FloatPredicate::OGT,
                    i::Condition::GreaterThanOrEqual => FloatPredicate::OGE,
                    i::Condition::LessThan => FloatPredicate::OLT,
                    i::Condition::LessThanOrEqual => FloatPredicate::OLE,
                };
                self.builder
                    .build_float_compare(
                        predicate,
                        ar.into_float_value(),
                        br.into_float_value(),
                        UNNAMED,
                    )
                    .into()
            }
            i::BinaryOperator::BAnd => self
                .builder
                .build_and(ar.into_int_value(), br.into_int_value(), UNNAMED)
                .into(),
            i::BinaryOperator::BOr => self
                .builder
                .build_or(ar.into_int_value(), br.into_int_value(), UNNAMED)
                .into(),
            i::BinaryOperator::BXor => self
                .builder
                .build_xor(ar.into_int_value(), br.into_int_value(), UNNAMED)
                .into(),
            i::BinaryOperator::And => self
                .builder
                .build_and(ar.into_int_value(), br.into_int_value(), UNNAMED)
                .into(),
            i::BinaryOperator::Or => self
                .builder
                .build_or(ar.into_int_value(), br.into_int_value(), UNNAMED)
                .into(),
            i::BinaryOperator::Xor => self
                .builder
                .build_xor(ar.into_int_value(), br.into_int_value(), UNNAMED)
                .into(),
            i::BinaryOperator::LeftShift => self
                .builder
                .build_left_shift(ar.into_int_value(), br.into_int_value(), UNNAMED)
                .into(),
            i::BinaryOperator::RightShift => self
                .builder
                .build_right_shift(ar.into_int_value(), br.into_int_value(), true, UNNAMED)
                .into(),
        }
    }

    pub fn convert_move(&mut self, from: &i::Value, to: &i::Value) {
        let dimensions: Vec<_> = to.dimensions.iter().map(|(len, _)| *len).collect();
        if dimensions.len() == 0 {
            let value = self.load_value(from, &[]);
            self.store_value(to, value, &[]);
        } else {
            let loops: Vec<_> = dimensions.iter().map(|_| self.start_loop()).collect();
            let mut loop_counters: Vec<_> = loops.iter().map(|(_, _, counter)| *counter).collect();
            loop_counters.insert(0, self.i32_const(0));
            let from = self.load_value_dyn(from, &loop_counters[..]);
            self.store_value_dyn(to, from, &loop_counters[..]);
            for (size, loop_params) in dimensions.into_iter().zip(loops.into_iter()).rev() {
                self.end_loop(size as i32, loop_params);
            }
        }
    }

    pub fn convert_store(&mut self, from: &i::Value, to: &i::Value, to_indexes: &Vec<i::Value>) {
        let dimensions: Vec<_> = from.dimensions.iter().map(|(len, _)| *len).collect();
        // Contains all of to_indexes followed by all the indexes that are iterated in the loop.
        let mut to_indexes = to_indexes.imc(|value| self.load_value(value, &[]).into_int_value());
        to_indexes.insert(0, self.i32_const(0));
        if dimensions.len() == 0 {
            let value = self.load_value(from, &[]);
            self.store_value_dyn(to, value, &to_indexes[..]);
        } else {
            let loops: Vec<_> = dimensions.iter().map(|_| self.start_loop()).collect();
            // Contains all the indexes that are iterated during the loop.
            let mut from_indexes = loops.imc(|(_, _, counter)| *counter);
            to_indexes.append(&mut from_indexes.clone());
            from_indexes.insert(0, self.i32_const(0));
            let value = self.load_value_dyn(from, &from_indexes[..]);
            self.store_value_dyn(to, value, &to_indexes[..]);
            for (size, loop_params) in dimensions.into_iter().zip(loops.into_iter()).rev() {
                self.end_loop(size as i32, loop_params);
            }
        }
    }

    pub fn convert_load(&mut self, from: &i::Value, from_indexes: &Vec<i::Value>, to: &i::Value) {
        let dimensions: Vec<_> = to.dimensions.iter().map(|(len, _)| *len).collect();
        // Contains all of to_indexes followed by all the indexes that are iterated in the loop.
        let mut from_indexes =
            from_indexes.imc(|value| self.load_value(value, &[]).into_int_value());
        from_indexes.insert(0, self.i32_const(0));
        if dimensions.len() == 0 {
            let value = self.load_value_dyn(from, &from_indexes[..]);
            self.store_value(to, value, &[]);
        } else {
            let loops: Vec<_> = dimensions.iter().map(|_| self.start_loop()).collect();
            // Contains all the indexes that are iterated during the loop.
            let mut to_indexes = loops.imc(|(_, _, counter)| *counter);
            from_indexes.append(&mut to_indexes.clone());
            to_indexes.insert(0, self.i32_const(0));
            let value = self.load_value_dyn(from, &from_indexes[..]);
            self.store_value_dyn(to, value, &to_indexes[..]);
            for (size, loop_params) in dimensions.into_iter().zip(loops.into_iter()).rev() {
                self.end_loop(size as i32, loop_params);
            }
        }
    }

    pub fn convert_label(&mut self, id: &i::LabelId) {
        if !self.current_block_terminated {
            self.builder
                .build_unconditional_branch(self.get_block_for_label(id));
        }
        self.builder.position_at_end(self.get_block_for_label(id));
        self.current_block_terminated = false;
    }

    pub fn convert_branch(
        &mut self,
        condition: &i::Value,
        true_target: &i::LabelId,
        false_target: &i::LabelId,
    ) {
        let condition = self.load_value(condition, &[]).into_int_value();
        self.builder.build_conditional_branch(
            condition,
            self.get_block_for_label(true_target),
            self.get_block_for_label(false_target),
        );
        self.current_block_terminated = true;
    }

    pub fn convert_abort(&mut self, error_code: u32) {
        self.builder.build_return(Some(&self.u32_const(error_code)));
        self.current_block_terminated = true;
    }

    pub fn convert_jump(&mut self, label: &i::LabelId) {
        self.builder
            .build_unconditional_branch(self.get_block_for_label(label));
        self.current_block_terminated = true;
    }
}
