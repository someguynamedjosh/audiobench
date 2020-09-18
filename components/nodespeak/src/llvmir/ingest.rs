use crate::llvmir::structure as o;
use crate::shared::{self, ProxyMode};
use crate::trivial::structure as i;
use inkwell::{
    basic_block::BasicBlock,
    builder::Builder,
    context::Context,
    module::Module,
    types::{BasicTypeEnum, PointerType},
    values::{BasicValue, BasicValueEnum, FloatValue, FunctionValue, IntValue, PointerValue},
    AddressSpace, FloatPredicate, IntPredicate,
};
use std::collections::HashMap;

const UNNAMED: &str = "";

struct Intrinsics<'ctx> {
    sqrt_f32: FunctionValue<'ctx>,
    powi_i32: FunctionValue<'ctx>,
    sin_f32: FunctionValue<'ctx>,
    cos_f32: FunctionValue<'ctx>,
    pow_f32: FunctionValue<'ctx>,
    exp_f32: FunctionValue<'ctx>,
    exp2_f32: FunctionValue<'ctx>,
    log_f32: FunctionValue<'ctx>,
    log10_f32: FunctionValue<'ctx>,
    log2_f32: FunctionValue<'ctx>,
    fabs_f32: FunctionValue<'ctx>,
    floor_f32: FunctionValue<'ctx>,
    ceil_f32: FunctionValue<'ctx>,
    trunc_f32: FunctionValue<'ctx>,
}

impl<'ctx> Intrinsics<'ctx> {
    fn new(module: &Module<'ctx>, context: &'ctx Context) -> Self {
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

struct Converter<'i, 'ctx> {
    source: &'i i::Program,
    input_pointer_type: PointerType<'ctx>,
    output_pointer_type: PointerType<'ctx>,
    static_pointer_type: PointerType<'ctx>,

    context: &'ctx Context,
    module: &'i Module<'ctx>,
    builder: Builder<'ctx>,
    intrinsics: Intrinsics<'ctx>,

    value_pointers: HashMap<i::VariableId, PointerValue<'ctx>>,
    label_blocks: Vec<BasicBlock<'ctx>>,
    current_block_terminated: bool,
}

impl<'i, 'ctx> Converter<'i, 'ctx> {
    fn u32_const(&self, value: u32) -> IntValue<'ctx> {
        // TODO: This is not u32
        self.context.i32_type().const_int(value as _, false)
    }

    fn i32_const(&self, value: i32) -> IntValue<'ctx> {
        self.context.i32_type().const_int(value as _, false)
    }

    fn f32_const(&self, value: f32) -> FloatValue<'ctx> {
        self.context.f32_type().const_float(value as _)
    }

    fn b1_const(&self, value: bool) -> IntValue<'ctx> {
        self.context.bool_type().const_int(value as _, false)
    }

    fn apply_proxy_to_const_indexes(proxy: &[(usize, ProxyMode)], indexes: &[u32]) -> Vec<u32> {
        debug_assert!(proxy.len() == indexes.len());
        let mut result = Vec::new();
        for position in 0..proxy.len() {
            match proxy[position].1 {
                ProxyMode::Keep => result.push(indexes[position]),
                ProxyMode::Collapse => result.push(0),
                ProxyMode::Discard => (),
            }
        }
        result
    }

    fn apply_proxy_to_dyn_indexes(
        &self,
        proxy: &[(usize, ProxyMode)],
        indexes: &[IntValue<'ctx>],
    ) -> Vec<IntValue<'ctx>> {
        if indexes.len() == 0 {
            assert!(proxy.len() == 0);
            return Vec::new();
        }
        debug_assert!(proxy.len() + 1 == indexes.len());
        let mut result = Vec::new();
        result.push(indexes[0]);
        for position in 0..proxy.len() {
            match proxy[position].1 {
                ProxyMode::Keep => result.push(indexes[position + 1]),
                ProxyMode::Collapse => result.push(self.u32_const(0)),
                ProxyMode::Discard => (),
            }
        }
        result
    }

    fn store_value<TYP: BasicValue<'ctx>>(
        &mut self,
        value: &i::Value,
        content: TYP,
        const_indexes: &[u32],
    ) {
        let mut indexes = Vec::new();
        if const_indexes.len() > 0 {
            indexes.push(self.u32_const(0));
            for index in const_indexes {
                indexes.push(self.u32_const(*index));
            }
        }
        self.store_value_dyn(value, content, &mut indexes[..]);
    }

    fn store_value_dyn<TYP: BasicValue<'ctx>>(
        &mut self,
        value: &i::Value,
        content: TYP,
        indexes: &mut [IntValue<'ctx>],
    ) {
        let indexes = self.apply_proxy_to_dyn_indexes(&value.dimensions, indexes);
        match &value.base {
            i::ValueBase::Variable(id) => {
                let mut ptr = *self
                    .value_pointers
                    .get(&id)
                    .expect("A variable was not given a pointer.");
                if indexes.len() > 0 {
                    ptr = unsafe { self.builder.build_gep(ptr, &indexes[..], UNNAMED) };
                } else {
                    assert!(value.dimensions.len() == 0);
                }
                self.builder.build_store(ptr, content);
            }
            i::ValueBase::Literal(..) => panic!("Cannot store to a constant."),
        }
    }

    fn load_value_dyn(
        &mut self,
        value: &i::Value,
        indexes: &mut [IntValue<'ctx>],
    ) -> BasicValueEnum<'ctx> {
        let indexes = self.apply_proxy_to_dyn_indexes(&value.dimensions, indexes);
        match &value.base {
            i::ValueBase::Variable(id) => {
                let mut ptr = *self
                    .value_pointers
                    .get(&id)
                    .expect("A variable was not given a pointer.");
                if indexes.len() > 0 {
                    ptr = unsafe { self.builder.build_gep(ptr, &indexes[..], UNNAMED) };
                }
                self.builder.build_load(ptr, UNNAMED)
            }
            i::ValueBase::Literal(data) => {
                // Last we left it, we were trying to figure out how to index stuff or something
                // like basically figure out what to do to return the requested value of the
                // known data.
                if let i::KnownData::Array(..) = &data {
                    let runtime_value = self.create_temp_value_holding_data(&data);
                    let required_dims = value.get_type(&self.source).collect_dimensions();
                    // +1 for pointer dereference.
                    assert!(required_dims.len() + 1 == indexes.len());
                    let value_ptr =
                        unsafe { self.builder.build_gep(runtime_value, &indexes[..], UNNAMED) };
                    self.builder.build_load(value_ptr, UNNAMED)
                } else {
                    assert!(indexes.len() == 0, "Cannot index scalar data.");
                    match data {
                        i::KnownData::Array(..) => unreachable!("Handled above."),
                        i::KnownData::Bool(value) => self.b1_const(*value).into(),
                        i::KnownData::Int(value) => self.i32_const(*value as i32).into(),
                        i::KnownData::Float(value) => self.f32_const(*value as f32).into(),
                    }
                }
            }
        }
    }

    fn load_value(&mut self, value: &i::Value, const_indexes: &[u32]) -> BasicValueEnum<'ctx> {
        let const_indexes = Self::apply_proxy_to_const_indexes(&value.dimensions, const_indexes);
        match &value.base {
            i::ValueBase::Variable(id) => {
                let mut ptr = *self
                    .value_pointers
                    .get(&id)
                    .expect("A variable was not given a pointer.");
                if const_indexes.len() > 0 {
                    let mut indices = Vec::new();
                    indices.push(self.u32_const(0));
                    for (index, (_, proxy_mode)) in value.dimensions.iter().enumerate() {
                        match proxy_mode {
                            ProxyMode::Keep => indices.push(self.u32_const(const_indexes[index])),
                            ProxyMode::Collapse => indices.push(self.u32_const(0)),
                            ProxyMode::Discard => (),
                        }
                    }
                    ptr = unsafe { self.builder.build_gep(ptr, &indices[..], UNNAMED) };
                }
                self.builder.build_load(ptr, UNNAMED)
            }
            i::ValueBase::Literal(data) => {
                let mut data = data.clone();
                for index in const_indexes {
                    if let i::KnownData::Array(mut values) = data {
                        data = values.remove(index as usize);
                    } else {
                        unreachable!("Illegal indexes should be caught earlier.");
                    }
                }
                match data {
                    i::KnownData::Array(..) => unimplemented!(),
                    i::KnownData::Bool(value) => self.b1_const(value).into(),
                    i::KnownData::Int(value) => self.i32_const(value as i32).into(),
                    i::KnownData::Float(value) => self.f32_const(value as f32).into(),
                }
            }
        }
    }

    fn store_data_in_ptr(
        &self,
        ptr: PointerValue<'ctx>,
        data: &i::KnownData,
        current_indexes: &[usize],
    ) {
        if let i::KnownData::Array(items) = data {
            debug_assert!(items.len() > 0);
            let mut new_indexes = Vec::with_capacity(current_indexes.len() + 1);
            for ci in current_indexes {
                new_indexes.push(*ci);
            }
            new_indexes.push(0);
            for item in items {
                self.store_data_in_ptr(ptr, item, &new_indexes[..]);
                let last = new_indexes.len() - 1;
                new_indexes[last] += 1;
            }
        } else {
            let mut ptr = ptr;
            if current_indexes.len() > 0 {
                let mut literal_indexes: Vec<_> = current_indexes
                    .iter()
                    .map(|i| self.u32_const(*i as u32))
                    .collect();
                literal_indexes.insert(0, self.u32_const(0));
                ptr = unsafe { self.builder.build_gep(ptr, &literal_indexes[..], UNNAMED) };
            }
            let value: BasicValueEnum<'ctx> = match data {
                i::KnownData::Bool(value) => self.b1_const(*value).into(),
                i::KnownData::Int(value) => self.i32_const(*value as i32).into(),
                i::KnownData::Float(value) => self.f32_const(*value as f32).into(),
                i::KnownData::Array(..) => unreachable!("Handled above."),
            };
            self.builder.build_store(ptr, value);
        }
    }

    fn create_temp_value_holding_data(&self, data: &i::KnownData) -> PointerValue<'ctx> {
        let dtype = data.get_type();
        let vtype = llvm_type(self.context, &dtype);
        let value_ptr = self.builder.build_alloca(vtype, UNNAMED);
        self.store_data_in_ptr(value_ptr, data, &[]);
        value_ptr
    }

    fn get_block_for_label(&self, id: &i::LabelId) -> BasicBlock<'ctx> {
        self.label_blocks[id.raw()]
    }

    fn usize_vec_to_u32(vec: Vec<usize>) -> Vec<u32> {
        vec.into_iter().map(|i| i as u32).collect()
    }

    fn convert_unary_expression(&mut self, op: &i::UnaryOperator, a: &i::Value, x: &i::Value) {
        let dimensions = x.dimensions.iter().map(|(len, _)| *len).collect();
        for position in shared::NDIndexIter::new(dimensions) {
            let coord = Self::usize_vec_to_u32(position);
            let ar = self.load_value(a, &coord[..]);

            let xr = self.do_unary_op(op, ar);
            self.store_value(x, xr, &coord[..]);
        }
    }

    fn build_call(
        &mut self,
        fn_ref: FunctionValue<'ctx>,
        args: &mut [BasicValueEnum<'ctx>],
    ) -> BasicValueEnum<'ctx> {
        self.builder
            .build_call(fn_ref, &args[..], UNNAMED)
            .try_as_basic_value()
            .left()
            .unwrap()
    }

    fn do_unary_op(
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

    fn convert_binary_expression(
        &mut self,
        op: &i::BinaryOperator,
        a: &i::Value,
        b: &i::Value,
        x: &i::Value,
    ) {
        let dimensions = x.dimensions.iter().map(|(len, _)| *len).collect();
        for position in shared::NDIndexIter::new(dimensions) {
            let coord = Self::usize_vec_to_u32(position);
            let ar = self.load_value(a, &coord[..]);
            let br = self.load_value(b, &coord[..]);

            let xr = self.do_binary_op(op, ar, br);
            self.store_value(x, xr, &coord[..]);
        }
    }

    fn do_binary_op(
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

    fn convert_move(&mut self, from: &i::Value, to: &i::Value) {
        let dimensions = to.dimensions.iter().map(|(len, _)| *len).collect();
        for position in shared::NDIndexIter::new(dimensions) {
            let coord = Self::usize_vec_to_u32(position);
            let from = self.load_value(from, &coord[..]);
            self.store_value(to, from, &coord[..]);
        }
    }

    fn convert_store(&mut self, from: &i::Value, to: &i::Value, to_indexes: &Vec<i::Value>) {
        let dimensions = from.dimensions.iter().map(|(len, _)| *len).collect();
        let mut dyn_indexes: Vec<_> = to_indexes
            .iter()
            .map(|value| self.load_value(value, &[]).into_int_value())
            .collect();
        dyn_indexes.insert(0, self.u32_const(0));
        for position in shared::NDIndexIter::new(dimensions) {
            let coord = Self::usize_vec_to_u32(position);
            let mut to_indexes = dyn_indexes.clone();
            for static_index in &coord {
                to_indexes.push(self.u32_const(*static_index));
            }
            let item = self.load_value(from, &coord[..]);
            self.store_value_dyn(to, item, &mut to_indexes[..]);
        }
    }

    fn convert_load(&mut self, from: &i::Value, from_indexes: &Vec<i::Value>, to: &i::Value) {
        let dimensions = to.dimensions.iter().map(|(len, _)| *len).collect();
        let mut dyn_indexes: Vec<_> = from_indexes
            .iter()
            .map(|value| self.load_value(value, &[]).into_int_value())
            .collect();
        dyn_indexes.insert(0, self.u32_const(0));
        for position in shared::NDIndexIter::new(dimensions) {
            let coord = Self::usize_vec_to_u32(position);
            let mut from_indexes = dyn_indexes.clone();
            for static_index in &coord {
                from_indexes.push(self.u32_const(*static_index));
            }
            let item = self.load_value_dyn(from, &mut from_indexes[..]);
            self.store_value(to, item, &coord[..]);
        }
    }

    fn convert_label(&mut self, id: &i::LabelId) {
        if !self.current_block_terminated {
            self.builder
                .build_unconditional_branch(self.get_block_for_label(id));
        }
        self.builder.position_at_end(self.get_block_for_label(id));
        self.current_block_terminated = false;
    }

    fn convert_branch(
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

    fn convert_abort(&mut self, error_code: u32) {
        self.builder.build_return(Some(&self.u32_const(error_code)));
        self.current_block_terminated = true;
    }

    fn convert_jump(&mut self, label: &i::LabelId) {
        self.builder
            .build_unconditional_branch(self.get_block_for_label(label));
        self.current_block_terminated = true;
    }

    fn create_variable_pointers_for_main_body(
        &mut self,
        input_pointer: PointerValue<'ctx>,
        static_pointer: PointerValue<'ctx>,
        output_pointer: PointerValue<'ctx>,
    ) {
        let mut input_index = 0;
        let mut output_index = 0;
        let mut static_index = 0;
        for var_id in self.source.iterate_all_variables() {
            let llvmt = llvm_type(self.context, self.source[var_id].borrow_type());
            let ptr = match self.source[var_id].get_location() {
                i::StorageLocation::Input => {
                    let indices = [self.u32_const(0), self.u32_const(input_index as u32)];
                    input_index += 1;
                    unsafe { self.builder.build_gep(input_pointer, &indices[..], UNNAMED) }
                }
                i::StorageLocation::Output => {
                    let indices = [self.u32_const(0), self.u32_const(output_index as u32)];
                    output_index += 1;
                    unsafe {
                        self.builder
                            .build_gep(output_pointer, &indices[..], UNNAMED)
                    }
                }
                i::StorageLocation::Static => {
                    let indices = [self.u32_const(0), self.u32_const(static_index as u32)];
                    static_index += 1;
                    unsafe {
                        self.builder
                            .build_gep(static_pointer, &indices[..], UNNAMED)
                    }
                }
                i::StorageLocation::StaticBody => {
                    continue;
                }
                i::StorageLocation::MainBody => self.builder.build_alloca(llvmt, UNNAMED),
            };
            self.value_pointers.insert(var_id, ptr);
        }
    }

    fn create_variable_pointers_for_static_body(&mut self, static_pointer: PointerValue<'ctx>) {
        let mut static_index = 0;
        for var_id in self.source.iterate_all_variables() {
            let llvmt = llvm_type(self.context, self.source[var_id].borrow_type());
            let ptr = match self.source[var_id].get_location() {
                i::StorageLocation::Input => {
                    continue;
                }
                i::StorageLocation::Output => {
                    continue;
                }
                i::StorageLocation::Static => {
                    let indices = [self.u32_const(0), self.u32_const(static_index as u32)];
                    static_index += 1;
                    unsafe {
                        self.builder
                            .build_gep(static_pointer, &indices[..], UNNAMED)
                    }
                }
                i::StorageLocation::StaticBody => self.builder.build_alloca(llvmt, UNNAMED),
                i::StorageLocation::MainBody => {
                    continue;
                }
            };
            self.value_pointers.insert(var_id, ptr);
        }
    }

    fn create_blocks_for_labels(&mut self, function: FunctionValue<'ctx>, static_body: bool) {
        for label in self.source.iterate_all_labels() {
            if self.source.is_label_in_static_body(label) != static_body {
                continue;
            }
            self.label_blocks.push(
                self.context
                    .append_basic_block(function, &format!("{:?}", label)),
            );
        }
    }

    fn create_blocks_for_main_body_labels(&mut self, function: FunctionValue<'ctx>) {
        self.create_blocks_for_labels(function, false);
    }

    fn create_blocks_for_static_body_labels(&mut self, function: FunctionValue<'ctx>) {
        self.create_blocks_for_labels(function, true);
    }

    fn reset(&mut self) {
        self.label_blocks.clear();
        self.value_pointers.clear();
        self.current_block_terminated = false;
    }

    fn convert_instruction(&mut self, instruction: &i::Instruction) {
        match instruction {
            i::Instruction::Abort(error_code) => self.convert_abort(*error_code),
            i::Instruction::BinaryOperation { op, a, b, x } => {
                self.convert_binary_expression(op, a, b, x)
            }
            i::Instruction::UnaryOperation { op, a, x } => self.convert_unary_expression(op, a, x),
            i::Instruction::Move { from, to } => self.convert_move(from, to),
            i::Instruction::Label(id) => self.convert_label(id),
            i::Instruction::Branch {
                condition,
                true_target,
                false_target,
            } => self.convert_branch(condition, true_target, false_target),
            i::Instruction::Jump { label } => self.convert_jump(label),
            i::Instruction::Store {
                from,
                to,
                to_indexes,
            } => self.convert_store(from, to, to_indexes),
            i::Instruction::Load {
                from,
                from_indexes,
                to,
            } => self.convert_load(from, from_indexes, to),
        }
    }

    fn convert(&mut self) {
        // LLVM related setup for main function.
        let i32t = self.context.i32_type();
        let argts = [
            self.input_pointer_type.into(),
            self.static_pointer_type.into(),
            self.output_pointer_type.into(),
        ];
        let function_type = i32t.fn_type(&argts[..], false);
        let main_fn = self.module.add_function("main", function_type, None);
        let entry_block = self.context.append_basic_block(main_fn, "entry");
        self.builder.position_at_end(entry_block);
        let input_pointer = main_fn.get_params()[0].into_pointer_value();
        let static_pointer = main_fn.get_params()[1].into_pointer_value();
        let output_pointer = main_fn.get_params()[2].into_pointer_value();

        // Self-related setup for main function.
        self.reset();
        self.create_variable_pointers_for_main_body(input_pointer, static_pointer, output_pointer);
        self.create_blocks_for_main_body_labels(main_fn);

        // Convert instructions.
        for instruction in self.source.borrow_instructions().clone() {
            self.convert_instruction(instruction);
        }

        // Add OK return if missing.
        if !self.current_block_terminated {
            self.builder.build_return(Some(&self.u32_const(0)));
        }

        // LLVM related setup for static init function.
        let argts = [self.static_pointer_type.into()];
        let function_type = i32t.fn_type(&argts[..], false);
        let static_init_fn = self.module.add_function("static_init", function_type, None);
        let entry_block = self.context.append_basic_block(static_init_fn, "entry");
        self.builder.position_at_end(entry_block);
        let static_pointer = static_init_fn.get_params()[0].into_pointer_value();

        // Self-related setup for main function.
        self.reset();
        self.create_variable_pointers_for_static_body(static_pointer);
        self.create_blocks_for_static_body_labels(static_init_fn);

        // Convert instructions.
        for instruction in self.source.borrow_static_init_instructions().clone() {
            self.convert_instruction(instruction);
        }

        // Add OK return if missing.
        if !self.current_block_terminated {
            self.builder.build_return(Some(&self.u32_const(0)));
        }

        // Dump human-readable IR to stdout
        println!("\nUnoptimized:");
        self.module.print_to_stderr();
    }
}

fn llvm_type<'ctx>(context: &'ctx Context, trivial_type: &i::DataType) -> BasicTypeEnum<'ctx> {
    match trivial_type {
        i::DataType::B1 => context.bool_type().into(),
        i::DataType::I32 => context.i32_type().into(),
        i::DataType::F32 => context.f32_type().into(),
        i::DataType::Array(len, etype) => match llvm_type(context, etype) {
            BasicTypeEnum::ArrayType(typ) => typ.array_type(*len as _).into(),
            BasicTypeEnum::FloatType(typ) => typ.array_type(*len as _).into(),
            BasicTypeEnum::IntType(typ) => typ.array_type(*len as _).into(),
            _ => unreachable!(),
        },
    }
}

use rental::rental;
rental! {
    mod rent_impl {
        use super::*;

        #[rental]
        pub struct TestRental {
            context: Box<Context>,
            module: Module<'context>,
        }
    }
}

fn create_module<'ctx>(source: &i::Program, context: &'ctx Context) -> Module<'ctx> {
    let module = context.create_module("nsprog");
    let builder = context.create_builder();

    let mut input_types = Vec::new();
    let mut output_types = Vec::new();
    let mut static_types = Vec::new();
    for var in source.iterate_all_variables() {
        let ltype = llvm_type(&*context, source[var].borrow_type());
        match source[var].get_location() {
            i::StorageLocation::Input => input_types.push(ltype),
            i::StorageLocation::Output => output_types.push(ltype),
            i::StorageLocation::Static => static_types.push(ltype),
            _ => (),
        }
    }

    // The input and output data types should be packed so that their layout can be more
    // easily predicted by the host program.
    let input_data_type = context.struct_type(&input_types[..], true);
    let input_pointer_type = input_data_type.ptr_type(AddressSpace::Generic);
    let output_data_type = context.struct_type(&output_types[..], true);
    let output_pointer_type = output_data_type.ptr_type(AddressSpace::Generic);
    let static_data_type = context.struct_type(&static_types[..], true);
    let static_pointer_type = static_data_type.ptr_type(AddressSpace::Generic);

    let intrinsics = Intrinsics::new(&module, &*context);

    let mut converter = Converter {
        source,
        input_pointer_type,
        output_pointer_type,
        static_pointer_type,

        context,
        module: &module,
        builder,
        intrinsics,

        value_pointers: HashMap::new(),
        label_blocks: Vec::with_capacity(source.iterate_all_labels().count()),
        current_block_terminated: false,
    };

    converter.convert();

    module
}

pub fn ingest(source: &i::Program) -> o::Program {
    let mut input_len = 0;
    let mut output_len = 0;
    let mut static_len = 0;
    for var in source.iterate_all_variables() {
        let typ = source[var].borrow_type();
        match source[var].get_location() {
            i::StorageLocation::Input => input_len += typ.size(),
            i::StorageLocation::Output => output_len += typ.size(),
            i::StorageLocation::Static => static_len += typ.size(),
            _ => (),
        }
    }

    let context = Context::create();
    o::Program::new(
        context,
        |context| Box::new(create_module(source, context)),
        input_len,
        output_len,
        static_len,
        source.borrow_error_descriptions().clone(),
    )
}
