// loop_flattener.rs - Flattens nested loops to prevent stack overflow

use inkwell::basic_block::BasicBlock;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::values::{BasicValueEnum, FunctionValue, IntValue};
use inkwell::IntPredicate;

/// Loop flattening helper functions
pub struct LoopFlattener<'ctx> {
    builder: &'ctx Builder<'ctx>,
    context: &'ctx Context,
}

impl<'ctx> LoopFlattener<'ctx> {
    /// Create a new loop flattener
    pub fn new(builder: &'ctx Builder<'ctx>, context: &'ctx Context) -> Self {
        Self { builder, context }
    }

    /// Check if a loop should be flattened
    pub fn should_flatten_loop(&self, loop_block: BasicBlock<'ctx>) -> bool {
        let mut contains_loop = false;
        let mut current_inst = loop_block.get_first_instruction();

        while let Some(inst) = current_inst {
            if inst.get_opcode() == inkwell::values::InstructionOpcode::Br {
                if let Some(target_block) = inst.get_operand(0) {
                    if let Some(block) = target_block.right() {
                        if let Ok(name) = block.get_name().to_str() {
                            if name.contains("loop") || name.contains("for") {
                                contains_loop = true;
                                break;
                            }
                        }
                    }
                }
            }

            current_inst = inst.get_next_instruction();
        }

        contains_loop
    }

    /// Flatten a nested loop
    /// This converts a nested loop into a single loop with additional state variables
    pub fn flatten_nested_loop(
        &self,
        function: FunctionValue<'ctx>,
        outer_loop_var: BasicValueEnum<'ctx>,
        outer_start: IntValue<'ctx>,
        outer_end: IntValue<'ctx>,
        outer_step: IntValue<'ctx>,
        inner_loop_var: BasicValueEnum<'ctx>,
        inner_start: IntValue<'ctx>,
        inner_end: IntValue<'ctx>,
        inner_step: IntValue<'ctx>,
        body_block: BasicBlock<'ctx>,
        exit_block: BasicBlock<'ctx>,
    ) -> BasicBlock<'ctx> {
        let i64_type = self.context.i64_type();

        let entry_block = self.builder.get_insert_block().unwrap();

        let loop_init = self.context.append_basic_block(function, "flat_loop_init");
        let loop_cond = self.context.append_basic_block(function, "flat_loop_cond");
        let loop_body = self.context.append_basic_block(function, "flat_loop_body");
        let loop_inc = self.context.append_basic_block(function, "flat_loop_inc");

        self.builder.build_unconditional_branch(loop_init).unwrap();

        self.builder.position_at_end(loop_init);

        let outer_var_ptr = self.builder.build_alloca(i64_type, "outer_var").unwrap();
        let inner_var_ptr = self.builder.build_alloca(i64_type, "inner_var").unwrap();

        self.builder
            .build_store(outer_var_ptr, outer_start)
            .unwrap();

        self.builder
            .build_store(inner_var_ptr, inner_start)
            .unwrap();

        self.builder.build_unconditional_branch(loop_cond).unwrap();

        self.builder.position_at_end(loop_cond);

        let outer_var = self
            .builder
            .build_load(i64_type, outer_var_ptr, "outer_var_val")
            .unwrap()
            .into_int_value();
        let inner_var = self
            .builder
            .build_load(i64_type, inner_var_ptr, "inner_var_val")
            .unwrap()
            .into_int_value();

        let outer_cond = self
            .builder
            .build_int_compare(IntPredicate::SLT, outer_var, outer_end, "outer_cond")
            .unwrap();

        let inner_cond = self
            .builder
            .build_int_compare(IntPredicate::SLT, inner_var, inner_end, "inner_cond")
            .unwrap();

        let combined_cond = self
            .builder
            .build_and(outer_cond, inner_cond, "combined_cond")
            .unwrap();

        self.builder
            .build_conditional_branch(combined_cond, loop_body, exit_block)
            .unwrap();

        self.builder.position_at_end(loop_body);

        self.builder
            .build_store(outer_loop_var.into_pointer_value(), outer_var)
            .unwrap();
        self.builder
            .build_store(inner_loop_var.into_pointer_value(), inner_var)
            .unwrap();

        self.builder.build_unconditional_branch(body_block).unwrap();

        self.builder.position_at_end(loop_inc);

        let next_inner = self
            .builder
            .build_int_add(inner_var, inner_step, "next_inner")
            .unwrap();
        self.builder.build_store(inner_var_ptr, next_inner).unwrap();

        let inner_done = self
            .builder
            .build_int_compare(IntPredicate::SGE, next_inner, inner_end, "inner_done")
            .unwrap();

        let reset_block = self.context.append_basic_block(function, "reset_inner");
        let continue_block = self.context.append_basic_block(function, "continue_loop");

        self.builder
            .build_conditional_branch(inner_done, reset_block, continue_block)
            .unwrap();

        self.builder.position_at_end(reset_block);

        self.builder
            .build_store(inner_var_ptr, inner_start)
            .unwrap();

        let next_outer = self
            .builder
            .build_int_add(outer_var, outer_step, "next_outer")
            .unwrap();
        self.builder.build_store(outer_var_ptr, next_outer).unwrap();

        self.builder
            .build_unconditional_branch(continue_block)
            .unwrap();

        self.builder.position_at_end(continue_block);

        self.builder.build_unconditional_branch(loop_cond).unwrap();

        entry_block
    }
}
