// tail_call_optimizer.rs - Optimizations for tail calls to prevent stack overflow

use inkwell::basic_block::BasicBlock;
use inkwell::values::{BasicValueEnum, FunctionValue};
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;

/// Tail call optimization helper functions
pub struct TailCallOptimizer<'ctx> {
    builder: &'ctx Builder<'ctx>,
    context: &'ctx Context,
    module: &'ctx Module<'ctx>,
}

impl<'ctx> TailCallOptimizer<'ctx> {
    /// Create a new tail call optimizer
    pub fn new(builder: &'ctx Builder<'ctx>, context: &'ctx Context, module: &'ctx Module<'ctx>) -> Self {
        Self { builder, context, module }
    }

    /// Apply tail call optimization to a function
    /// This converts recursive calls at the end of a function into loops
    pub fn optimize_function(&self, function: FunctionValue<'ctx>) -> bool {
        // Check if the function has any tail calls
        let mut has_tail_calls = false;

        // Iterate through all basic blocks in the function
        for block in function.get_basic_blocks() {
            // Check if the block ends with a return instruction
            if let Some(terminator) = block.get_terminator() {
                // Check if the terminator is a return instruction
                if terminator.get_opcode() == inkwell::values::InstructionOpcode::Return {
                    // Check if the return value is a call instruction
                    if let Some(return_value) = terminator.get_operand(0) {
                        if let Some(call_inst) = return_value.left() {
                            if call_inst.get_opcode() == inkwell::values::InstructionOpcode::Call {
                                // This is a tail call - convert it to a loop
                                has_tail_calls = true;
                                self.convert_tail_call_to_loop(block, call_inst.into_instruction_value());
                            }
                        }
                    }
                }
            }
        }

        has_tail_calls
    }

    /// Convert a tail call to a loop
    fn convert_tail_call_to_loop(&self, block: BasicBlock<'ctx>, call_inst: inkwell::values::InstructionValue<'ctx>) {
        // Get the function being called
        if let Some(called_fn) = call_inst.get_called_function_value() {
            // Only optimize tail calls to the same function (recursive calls)
            if called_fn == block.get_parent().unwrap() {
                // Create a new entry block for the function
                let function = block.get_parent().unwrap();
                let entry_block = function.get_first_basic_block().unwrap();
                
                // Create a new block for the loop header
                let loop_header = self.context.append_basic_block(function, "tail_call_loop");
                
                // Move all instructions from the entry block to the loop header
                // except for the parameter declarations
                let mut instructions = Vec::new();
                let mut current_inst = entry_block.get_first_instruction();
                
                while let Some(inst) = current_inst {
                    // Skip parameter declarations
                    if inst.get_opcode() != inkwell::values::InstructionOpcode::Alloca {
                        instructions.push(inst);
                    }
                    current_inst = inst.get_next_instruction();
                }
                
                // Remove the instructions from the entry block
                for inst in &instructions {
                    inst.remove_from_basic_block();
                }
                
                // Add a branch from the entry block to the loop header
                self.builder.position_at_end(entry_block);
                self.builder.build_unconditional_branch(loop_header).unwrap();
                
                // Add the instructions to the loop header
                self.builder.position_at_end(loop_header);
                for inst in instructions {
                    self.builder.insert_instruction(inst).unwrap();
                }
                
                // Replace the tail call with assignments to the parameters
                // and a branch back to the loop header
                self.builder.position_at_end(block);
                
                // Get the parameters of the function
                let params = function.get_params();
                
                // Get the arguments of the call
                let mut args = Vec::new();
                for i in 0..call_inst.get_num_operands() - 1 {
                    if let Some(arg) = call_inst.get_operand(i) {
                        args.push(arg.left().unwrap());
                    }
                }
                
                // Assign the arguments to the parameters
                for (param, arg) in params.iter().zip(args.iter()) {
                    // Create a temporary variable to hold the argument value
                    let temp_var = self.builder.build_alloca(param.get_type(), "temp").unwrap();
                    self.builder.build_store(temp_var, *arg).unwrap();
                    
                    // Store the argument value in the parameter
                    let param_ptr = function.get_nth_param(param.get_param_index()).unwrap();
                    let param_val = self.builder.build_load(param.get_type(), temp_var, "param_val").unwrap();
                    self.builder.build_store(param_ptr.into_pointer_value(), param_val).unwrap();
                }
                
                // Remove the call instruction and the return instruction
                call_inst.remove_from_basic_block();
                block.get_terminator().unwrap().remove_from_basic_block();
                
                // Add a branch back to the loop header
                self.builder.build_unconditional_branch(loop_header).unwrap();
            }
        }
    }
}
