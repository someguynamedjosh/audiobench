//! Types used by all other files.

use crate::trivial::structure as i;
use inkwell::{
    basic_block::BasicBlock,
    builder::Builder,
    context::Context,
    module::Module,
    values::{FunctionValue, PointerValue},
};
use std::collections::HashMap;

pub struct Intrinsics<'ctx> {
    pub sqrt_f32: FunctionValue<'ctx>,
    pub powi_i32: FunctionValue<'ctx>,
    pub sin_f32: FunctionValue<'ctx>,
    pub cos_f32: FunctionValue<'ctx>,
    pub pow_f32: FunctionValue<'ctx>,
    pub exp_f32: FunctionValue<'ctx>,
    pub exp2_f32: FunctionValue<'ctx>,
    pub log_f32: FunctionValue<'ctx>,
    pub log10_f32: FunctionValue<'ctx>,
    pub log2_f32: FunctionValue<'ctx>,
    pub fabs_f32: FunctionValue<'ctx>,
    pub floor_f32: FunctionValue<'ctx>,
    pub ceil_f32: FunctionValue<'ctx>,
    pub trunc_f32: FunctionValue<'ctx>,
}

impl<'ctx> Intrinsics<'ctx> {
    pub fn new(module: &Module<'ctx>, context: &'ctx Context) -> Self {
        let make = |name: &str| -> FunctionValue<'ctx> {
            let float_type = context.f32_type();
            let arg_types = [float_type.into()];
            let fn_type = float_type.fn_type(&arg_types[..], false);
            module.add_function(name, fn_type, None)
        };
        let make_f32_f32_f32 = |name: &str| -> FunctionValue<'ctx> {
            let float_type = context.f32_type();
            let arg_types = [float_type.into(), float_type.into()];
            let fn_type = float_type.fn_type(&arg_types[..], false);
            module.add_function(name, fn_type, None)
        };
        let make_i32_i32_i32 = |name: &str| -> FunctionValue<'ctx> {
            let int_type = context.i32_type();
            let arg_types = [int_type.into(), int_type.into()];
            let fn_type = int_type.fn_type(&arg_types[..], false);
            module.add_function(name, fn_type, None)
        };

        Self {
            sqrt_f32: make("llvm.sqrt.f32"),
            sin_f32: make("llvm.sin.f32"),
            cos_f32: make("llvm.cos.f32"),
            pow_f32: make_f32_f32_f32("llvm.pow.f32"),
            powi_i32: make_i32_i32_i32("llvm.powi.i32"),
            exp_f32: make("llvm.exp.f32"),
            exp2_f32: make("llvm.exp2.f32"),
            log_f32: make("llvm.log.f32"),
            log10_f32: make("llvm.log10.f32"),
            log2_f32: make("llvm.log2.f32"),
            fabs_f32: make("llvm.fabs.f32"),
            floor_f32: make("llvm.floor.f32"),
            ceil_f32: make("llvm.ceil.f32"),
            trunc_f32: make("llvm.trunc.f32"),
        }
    }
}

pub struct Converter<'i, 'ctx> {
    pub source: &'i i::Program,
    pub main_fn: FunctionValue<'ctx>,
    pub static_init_fn: FunctionValue<'ctx>,
    pub current_fn: FunctionValue<'ctx>,

    pub context: &'ctx Context,
    pub module: &'i Module<'ctx>,
    pub builder: Builder<'ctx>,
    pub intrinsics: Intrinsics<'ctx>,

    pub value_pointers: HashMap<i::VariableId, PointerValue<'ctx>>,
    pub label_blocks: Vec<BasicBlock<'ctx>>,
    pub current_block_terminated: bool,
}
