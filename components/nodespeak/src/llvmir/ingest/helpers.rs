//! Helper functions that are used in several different situations.

use super::Converter;
use crate::shared::ProxyMode;
use crate::trivial::structure as i;
use inkwell::{
    basic_block::BasicBlock,
    context::Context,
    types::BasicTypeEnum,
    values::{
        BasicValue, BasicValueEnum, FloatValue, FunctionValue, IntValue, PhiValue, PointerValue,
    },
    IntPredicate,
};

const UNNAMED: &str = "";

pub fn llvm_type<'ctx>(context: &'ctx Context, trivial_type: &i::DataType) -> BasicTypeEnum<'ctx> {
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

impl<'i, 'ctx> Converter<'i, 'ctx> {
    pub fn u32_const(&self, value: u32) -> IntValue<'ctx> {
        // TODO: This is not u32
        self.context.i32_type().const_int(value as _, false)
    }

    pub fn i32_const(&self, value: i32) -> IntValue<'ctx> {
        self.context.i32_type().const_int(value as _, false)
    }

    pub fn f32_const(&self, value: f32) -> FloatValue<'ctx> {
        self.context.f32_type().const_float(value as _)
    }

    pub fn b1_const(&self, value: bool) -> IntValue<'ctx> {
        self.context.bool_type().const_int(value as _, false)
    }

    pub fn apply_proxy_to_const_indexes(proxy: &[(usize, ProxyMode)], indexes: &[u32]) -> Vec<u32> {
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

    pub fn apply_proxy_to_dyn_indexes(
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

    pub fn store_value<TYP: BasicValue<'ctx>>(
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

    pub fn store_value_dyn<TYP: BasicValue<'ctx>>(
        &mut self,
        value: &i::Value,
        content: TYP,
        indexes: &[IntValue<'ctx>],
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

    pub fn load_value_dyn(
        &mut self,
        value: &i::Value,
        indexes: &[IntValue<'ctx>],
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
                    // LLVM requires the first index to be zero and that the actual indexes be
                    // placed after that. Having just that first noop index means we are not
                    // actually trying to index anything.
                    assert!(
                        indexes.len() == 0 || indexes.len() == 1,
                        "Cannot index scalar data."
                    );
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

    pub fn load_value(&mut self, value: &i::Value, const_indexes: &[u32]) -> BasicValueEnum<'ctx> {
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

    pub fn store_data_in_ptr(
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

    pub fn create_temp_value_holding_data(&self, data: &i::KnownData) -> PointerValue<'ctx> {
        let dtype = data.get_type();
        let vtype = llvm_type(self.context, &dtype);
        let value_ptr = self.builder.build_alloca(vtype, UNNAMED);
        self.store_data_in_ptr(value_ptr, data, &[]);
        value_ptr
    }

    pub fn get_block_for_label(&self, id: &i::LabelId) -> BasicBlock<'ctx> {
        self.label_blocks[id.raw()]
    }

    pub fn start_loop(&mut self) -> (BasicBlock<'ctx>, PhiValue<'ctx>, IntValue<'ctx>) {
        assert!(!self.current_block_terminated);
        let body_block = self
            .context
            .append_basic_block(self.current_fn, "loop_body");
        let entry_block = self.builder.get_insert_block().unwrap();
        self.builder.build_unconditional_branch(body_block);
        self.builder.position_at_end(body_block);
        let phi = self.builder.build_phi(self.context.i32_type(), UNNAMED);
        phi.add_incoming(&[(&self.i32_const(0), entry_block)]);
        self.current_block_terminated = false;
        let int_value = phi.as_basic_value().into_int_value();
        (body_block, phi, int_value)
    }

    pub fn end_loop(
        &mut self,
        num_iterations: i32,
        loop_params: (BasicBlock<'ctx>, PhiValue<'ctx>, IntValue<'ctx>),
    ) {
        assert!(!self.current_block_terminated);
        let loop_exit_block = self
            .context
            .append_basic_block(self.current_fn, "loop_exit");
        let incremented_counter =
            self.builder
                .build_int_add(self.i32_const(1), loop_params.2, UNNAMED);
        let current_block = self.builder.get_insert_block().unwrap();
        loop_params
            .1
            .add_incoming(&[(&incremented_counter, current_block)]);
        let repeat = self.builder.build_int_compare(
            IntPredicate::ULT,
            incremented_counter,
            self.i32_const(num_iterations),
            UNNAMED,
        );
        self.builder
            .build_conditional_branch(repeat, loop_params.0, loop_exit_block);
        self.builder.position_at_end(loop_exit_block);
        self.current_block_terminated = false;
    }

    pub fn build_call(
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
}
