//! Top-level functions for converting Trivial to LLVMIR.

use super::base::{Converter, Intrinsics};
use super::helpers::llvm_type;
use crate::llvmir::structure as o;
use crate::trivial::structure as i;
use inkwell::{
    context::Context,
    module::Module,
    values::{FunctionValue, PointerValue},
    AddressSpace,
};
use std::collections::HashMap;

const UNNAMED: &str = "";

impl<'i, 'ctx> Converter<'i, 'ctx> {
    pub fn create_variable_pointers_for_main_body(
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

    pub fn create_variable_pointers_for_static_body(&mut self, static_pointer: PointerValue<'ctx>) {
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

    pub fn create_blocks_for_labels(&mut self, function: FunctionValue<'ctx>, static_body: bool) {
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

    pub fn create_blocks_for_main_body_labels(&mut self, function: FunctionValue<'ctx>) {
        self.create_blocks_for_labels(function, false);
    }

    pub fn create_blocks_for_static_body_labels(&mut self, function: FunctionValue<'ctx>) {
        self.create_blocks_for_labels(function, true);
    }

    pub fn reset(&mut self) {
        self.label_blocks.clear();
        self.value_pointers.clear();
        self.current_block_terminated = false;
    }

    pub fn convert_instruction(&mut self, instruction: &i::Instruction) {
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

    pub fn convert(&mut self) {
        // LLVM related setup for main function.
        let entry_block = self.context.append_basic_block(self.main_fn, "entry");
        self.builder.position_at_end(entry_block);
        let input_pointer = self.main_fn.get_params()[0].into_pointer_value();
        let static_pointer = self.main_fn.get_params()[1].into_pointer_value();
        let output_pointer = self.main_fn.get_params()[2].into_pointer_value();

        // Self-related setup for main function.
        self.reset();
        self.create_variable_pointers_for_main_body(input_pointer, static_pointer, output_pointer);
        self.create_blocks_for_main_body_labels(self.main_fn);
        self.current_fn = self.main_fn;

        // Convert instructions.
        for instruction in self.source.borrow_instructions().clone() {
            self.convert_instruction(instruction);
        }

        // Add OK return if missing.
        if !self.current_block_terminated {
            self.builder.build_return(Some(&self.u32_const(0)));
        }

        // LLVM related setup for static init function.
        let entry_block = self
            .context
            .append_basic_block(self.static_init_fn, "entry");
        self.builder.position_at_end(entry_block);
        let static_pointer = self.static_init_fn.get_params()[0].into_pointer_value();

        // Self-related setup for main function.
        self.reset();
        self.create_variable_pointers_for_static_body(static_pointer);
        self.create_blocks_for_static_body_labels(self.static_init_fn);
        self.current_fn = self.static_init_fn;

        // Convert instructions.
        for instruction in self.source.borrow_static_init_instructions().clone() {
            self.convert_instruction(instruction);
        }

        // Add OK return if missing.
        if !self.current_block_terminated {
            self.builder.build_return(Some(&self.u32_const(0)));
        }

        // Dump human-readable IR to stdout
        if cfg!(feature = "dump-llvmir") {
            println!("\nUnoptimized:");
            self.module.print_to_stderr();
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

    let i32t = context.i32_type();
    let argts = [
        input_pointer_type.into(),
        static_pointer_type.into(),
        output_pointer_type.into(),
    ];
    let function_type = i32t.fn_type(&argts[..], false);
    let main_fn = module.add_function("main", function_type, None);

    let argts = [static_pointer_type.into()];
    let function_type = i32t.fn_type(&argts[..], false);
    let static_init_fn = module.add_function("static_init", function_type, None);

    let mut converter = Converter {
        source,
        main_fn,
        static_init_fn,
        current_fn: main_fn,

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
