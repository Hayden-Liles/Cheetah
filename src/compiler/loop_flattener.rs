// loop_flattener.rs - Flattens nested loops to prevent stack overflow

use inkwell::basic_block::BasicBlock;
use inkwell::values::{BasicValueEnum, IntValue, FunctionValue};
use inkwell::IntPredicate;
use inkwell::builder::Builder;
use inkwell::context::Context;

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
        // Check if the loop contains another loop
        // This is a simple heuristic - we could make it more sophisticated
        let mut contains_loop = false;
        let mut current_inst = loop_block.get_first_instruction();
        
        while let Some(inst) = current_inst {
            // Check if this instruction is a branch to a loop header
            if inst.get_opcode() == inkwell::values::InstructionOpcode::Br {
                if let Some(target_block) = inst.get_operand(0) {
                    if let Some(block) = target_block.right() {
                        // Check if the block name contains "loop" or "for"
                        if let Some(name) = block.get_name().to_str() {
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
        
        // Create a new entry block for the flattened loop
        let entry_block = self.builder.get_insert_block().unwrap();
        
        // Create basic blocks for the flattened loop
        let loop_init = self.context.append_basic_block(function, "flat_loop_init");
        let loop_cond = self.context.append_basic_block(function, "flat_loop_cond");
        let loop_body = self.context.append_basic_block(function, "flat_loop_body");
        let loop_inc = self.context.append_basic_block(function, "flat_loop_inc");
        
        // Branch to the loop initialization block
        self.builder.build_unconditional_branch(loop_init).unwrap();
        
        // Loop initialization block
        self.builder.position_at_end(loop_init);
        
        // Create state variables for the flattened loop
        let outer_var_ptr = self.builder.build_alloca(i64_type, "outer_var").unwrap();
        let inner_var_ptr = self.builder.build_alloca(i64_type, "inner_var").unwrap();
        
        // Initialize the outer loop variable
        self.builder.build_store(outer_var_ptr, outer_start).unwrap();
        
        // Initialize the inner loop variable
        self.builder.build_store(inner_var_ptr, inner_start).unwrap();
        
        // Branch to the loop condition
        self.builder.build_unconditional_branch(loop_cond).unwrap();
        
        // Loop condition block
        self.builder.position_at_end(loop_cond);
        
        // Load the current values of the loop variables
        let outer_var = self.builder.build_load(i64_type, outer_var_ptr, "outer_var_val").unwrap().into_int_value();
        let inner_var = self.builder.build_load(i64_type, inner_var_ptr, "inner_var_val").unwrap().into_int_value();
        
        // Check if the outer loop is done
        let outer_cond = self.builder.build_int_compare(
            IntPredicate::SLT,
            outer_var,
            outer_end,
            "outer_cond"
        ).unwrap();
        
        // Check if the inner loop is done
        let inner_cond = self.builder.build_int_compare(
            IntPredicate::SLT,
            inner_var,
            inner_end,
            "inner_cond"
        ).unwrap();
        
        // Combine the conditions
        let combined_cond = self.builder.build_and(
            outer_cond,
            inner_cond,
            "combined_cond"
        ).unwrap();
        
        // Branch based on the combined condition
        self.builder.build_conditional_branch(combined_cond, loop_body, exit_block).unwrap();
        
        // Loop body block
        self.builder.position_at_end(loop_body);
        
        // Store the current values in the original loop variables
        self.builder.build_store(outer_loop_var.into_pointer_value(), outer_var).unwrap();
        self.builder.build_store(inner_loop_var.into_pointer_value(), inner_var).unwrap();
        
        // Branch to the original body block
        self.builder.build_unconditional_branch(body_block).unwrap();
        
        // Loop increment block
        self.builder.position_at_end(loop_inc);
        
        // Increment the inner loop variable
        let next_inner = self.builder.build_int_add(inner_var, inner_step, "next_inner").unwrap();
        self.builder.build_store(inner_var_ptr, next_inner).unwrap();
        
        // Check if the inner loop is done
        let inner_done = self.builder.build_int_compare(
            IntPredicate::SGE,
            next_inner,
            inner_end,
            "inner_done"
        ).unwrap();
        
        // If the inner loop is done, reset it and increment the outer loop
        let reset_block = self.context.append_basic_block(function, "reset_inner");
        let continue_block = self.context.append_basic_block(function, "continue_loop");
        
        self.builder.build_conditional_branch(inner_done, reset_block, continue_block).unwrap();
        
        // Reset block
        self.builder.position_at_end(reset_block);
        
        // Reset the inner loop variable
        self.builder.build_store(inner_var_ptr, inner_start).unwrap();
        
        // Increment the outer loop variable
        let next_outer = self.builder.build_int_add(outer_var, outer_step, "next_outer").unwrap();
        self.builder.build_store(outer_var_ptr, next_outer).unwrap();
        
        // Branch to the continue block
        self.builder.build_unconditional_branch(continue_block).unwrap();
        
        // Continue block
        self.builder.position_at_end(continue_block);
        
        // Branch back to the loop condition
        self.builder.build_unconditional_branch(loop_cond).unwrap();
        
        // Return the entry block
        entry_block
    }
}
