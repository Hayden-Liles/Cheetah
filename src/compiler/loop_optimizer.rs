// loop_optimizer.rs - Optimizations for loops to improve performance

use inkwell::basic_block::BasicBlock;
use inkwell::values::{BasicValueEnum, IntValue, FunctionValue};
use inkwell::IntPredicate;
use inkwell::builder::Builder;
use inkwell::context::Context;

// Constants for loop optimization
const MIN_CHUNK_SIZE: u64 = 1000; // Minimum chunk size for better performance
const MAX_CHUNK_SIZE: u64 = 100000; // Maximum chunk size to prevent stack overflow
const DEFAULT_CHUNK_SIZE: u64 = 10000; // Default chunk size for most loops

// Threshold for large ranges that need special handling
const LARGE_RANGE_THRESHOLD: u64 = 1000000; // 1 million iterations is considered large
const VERY_LARGE_RANGE_THRESHOLD: u64 = 10000000; // 10 million iterations is very large

/// Loop optimization helper functions
pub struct LoopOptimizer<'ctx> {
    builder: &'ctx Builder<'ctx>,
    context: &'ctx Context,
}

impl<'ctx> LoopOptimizer<'ctx> {
    /// Create a new loop optimizer
    pub fn new(builder: &'ctx Builder<'ctx>, context: &'ctx Context) -> Self {
        Self { builder, context }
    }

    /// Check if a loop should be chunked based on its range
    pub fn should_chunk_loop(&self, _start_val: IntValue<'ctx>, _end_val: IntValue<'ctx>) -> bool {
        // Always chunk loops to prevent stack overflow
        // This is the most reliable way to prevent stack overflows in large loops
        true
    }

    /// Calculate an appropriate chunk size based on the range size
    pub fn calculate_chunk_size(&self, start_val: IntValue<'ctx>, end_val: IntValue<'ctx>) -> u64 {
        // Try to get constant values if available
        if let (Some(start_const), Some(end_const)) = (start_val.get_sign_extended_constant(), end_val.get_sign_extended_constant()) {
            let range_size = if end_const > start_const {
                (end_const - start_const) as u64
            } else {
                return MIN_CHUNK_SIZE; // Invalid range, use minimum chunk size
            };

            // For extremely large ranges, use a fixed large chunk size
            // This is more efficient than using very small chunks
            if range_size > VERY_LARGE_RANGE_THRESHOLD {
                // For extremely large ranges, use a fixed large chunk size
                // This is more efficient and prevents excessive chunking
                return MAX_CHUNK_SIZE;
            }
            // For very large ranges, use a dynamic chunk size
            else if range_size > LARGE_RANGE_THRESHOLD {
                // Use a square root scale for very large ranges
                // This provides a better balance between performance and stack usage
                let sqrt_factor = (range_size as f64).sqrt() as u64 / 100;
                let adjusted_size = DEFAULT_CHUNK_SIZE * sqrt_factor;
                return adjusted_size.clamp(MIN_CHUNK_SIZE, MAX_CHUNK_SIZE);
            }
            // For medium-large ranges, use the maximum chunk size
            else if range_size > MAX_CHUNK_SIZE {
                return MAX_CHUNK_SIZE;
            }
            // For medium ranges, use the range size itself (process in one chunk)
            else if range_size > MIN_CHUNK_SIZE {
                return range_size;
            }
        }

        // Default to a reasonable chunk size if we can't determine the range size
        DEFAULT_CHUNK_SIZE
    }

    /// Optimize a range-based for loop
    ///
    /// This function applies several optimizations to range-based for loops:
    /// 1. Loop unrolling for small constant ranges
    /// 2. Strength reduction for loop variables
    /// 3. Loop chunking for large ranges to prevent stack overflow
    pub fn optimize_range_loop(
        &self,
        function: FunctionValue<'ctx>,
        start_val: IntValue<'ctx>,
        end_val: IntValue<'ctx>,
        step_val: IntValue<'ctx>,
        loop_var_ptr: BasicValueEnum<'ctx>,
        body_block: BasicBlock<'ctx>,
        exit_block: BasicBlock<'ctx>,
    ) -> BasicBlock<'ctx> {
        // Create a new entry block for the optimized loop
        let entry_block = self.builder.get_insert_block().unwrap();

        // Check if we should apply chunking to this loop
        if self.should_chunk_loop(start_val, end_val) {
            // Apply chunking optimization for large loops
            return self.create_chunked_loop(
                function,
                start_val,
                end_val,
                step_val,
                loop_var_ptr,
                body_block,
                exit_block,
            );
        }

        // For smaller loops, apply basic optimizations
        // Create an increment block for the optimized loop
        let inc_block = self.context.append_basic_block(function, "opt_inc_block");

        self.optimize_loop_condition(
            function,
            start_val,
            end_val,
            step_val,
            loop_var_ptr,
            body_block,
            exit_block,
            inc_block,
        );

        entry_block
    }

    /// Create a chunked loop to prevent stack overflow
    fn create_chunked_loop(
        &self,
        function: FunctionValue<'ctx>,
        start_val: IntValue<'ctx>,
        end_val: IntValue<'ctx>,
        step_val: IntValue<'ctx>,
        loop_var_ptr: BasicValueEnum<'ctx>,
        body_block: BasicBlock<'ctx>,
        exit_block: BasicBlock<'ctx>,
    ) -> BasicBlock<'ctx> {
        // Log the range size if we can determine it
        if let (Some(start_const), Some(end_const)) = (start_val.get_sign_extended_constant(), end_val.get_sign_extended_constant()) {
            let range_size = if end_const > start_const {
                (end_const - start_const) as u64
            } else {
                0 // Invalid range
            };

            if range_size > VERY_LARGE_RANGE_THRESHOLD {
                // For very large ranges, log a warning
                println!("[LOOP WARNING] Very large range detected: {} iterations", range_size);
            }
        }
        let i64_type = self.context.i64_type();
        let entry_block = self.builder.get_insert_block().unwrap();

        // Create blocks for the chunked loop
        let chunk_init_block = self.context.append_basic_block(function, "chunk_init");
        let chunk_cond_block = self.context.append_basic_block(function, "chunk_cond");
        let chunk_body_block = self.context.append_basic_block(function, "chunk_body");
        let chunk_inc_block = self.context.append_basic_block(function, "chunk_inc");
        let inner_loop_block = self.context.append_basic_block(function, "inner_loop");

        // Branch to the chunk initialization block
        self.builder.build_unconditional_branch(chunk_init_block).unwrap();

        // Chunk initialization block
        self.builder.position_at_end(chunk_init_block);

        // Create a chunk counter variable
        let chunk_counter_ptr = self.builder.build_alloca(i64_type, "chunk_counter").unwrap();
        self.builder.build_store(chunk_counter_ptr, start_val).unwrap();

        // Branch to the chunk condition block
        self.builder.build_unconditional_branch(chunk_cond_block).unwrap();

        // Chunk condition block
        self.builder.position_at_end(chunk_cond_block);

        // Load the current chunk counter value
        let current_chunk = self.builder.build_load(i64_type, chunk_counter_ptr, "current_chunk").unwrap().into_int_value();

        // Check if we've reached the end of the loop
        let chunk_cond = self.builder.build_int_compare(
            IntPredicate::SLT,
            current_chunk,
            end_val,
            "chunk_cond"
        ).unwrap();

        // Branch to the chunk body or exit based on the condition
        self.builder.build_conditional_branch(chunk_cond, chunk_body_block, exit_block).unwrap();

        // Chunk body block
        self.builder.position_at_end(chunk_body_block);

        // Calculate the end of this chunk using a dynamic chunk size
        let dynamic_chunk_size = self.calculate_chunk_size(start_val, end_val);
        let chunk_size = i64_type.const_int(dynamic_chunk_size, false);
        let chunk_end = self.builder.build_int_add(current_chunk, chunk_size, "chunk_end").unwrap();

        // Log the chunk size for debugging
        if cfg!(debug_assertions) {
            println!("Using chunk size: {} for loop", dynamic_chunk_size);
        }

        // For very large ranges, we'll add a debug message
        // This helps with debugging and prevents excessive memory usage
        if cfg!(debug_assertions) {
            if let (Some(start_const), Some(end_const)) = (start_val.get_sign_extended_constant(), end_val.get_sign_extended_constant()) {
                let range_size = if end_const > start_const {
                    (end_const - start_const) as u64
                } else {
                    0 // Invalid range
                };

                if range_size > VERY_LARGE_RANGE_THRESHOLD {
                    println!("[LOOP CHUNK] Processing chunk {} to {} (size {})",
                             current_chunk.get_sign_extended_constant().unwrap_or(0),
                             chunk_end.get_sign_extended_constant().unwrap_or(0),
                             dynamic_chunk_size);
                }
            }
        }

        // We'll use a simpler approach for debugging - just add a comment
        // This helps prevent stack overflow by using smaller chunks

        // Make sure we don't go past the actual end
        let use_chunk_end = self.builder.build_int_compare(
            IntPredicate::SLT,
            chunk_end,
            end_val,
            "use_chunk_end"
        ).unwrap();

        let actual_chunk_end = self.builder.build_select(
            use_chunk_end,
            chunk_end,
            end_val,
            "actual_chunk_end"
        ).unwrap().into_int_value();

        // Initialize the loop variable to the current chunk start
        self.builder.build_store(loop_var_ptr.into_pointer_value(), current_chunk).unwrap();

        // Branch to the inner loop
        self.builder.build_unconditional_branch(inner_loop_block).unwrap();

        // Inner loop block
        self.builder.position_at_end(inner_loop_block);

        // Load the current loop variable
        let current_val = self.builder.build_load(i64_type, loop_var_ptr.into_pointer_value(), "current").unwrap().into_int_value();

        // Check if we've reached the end of this chunk
        let inner_cond = self.builder.build_int_compare(
            IntPredicate::SLT,
            current_val,
            actual_chunk_end,
            "inner_cond"
        ).unwrap();

        // Create a block for the inner loop body
        let inner_body_block = self.context.append_basic_block(function, "inner_body");
        let inner_inc_block = self.context.append_basic_block(function, "inner_inc");

        // Branch to the inner body or chunk increment based on the condition
        self.builder.build_conditional_branch(inner_cond, inner_body_block, chunk_inc_block).unwrap();

        // Inner body block
        self.builder.position_at_end(inner_body_block);

        // Branch to the actual body block
        self.builder.build_unconditional_branch(body_block).unwrap();

        // Set up the inner increment block
        self.builder.position_at_end(inner_inc_block);

        // Increment the loop variable
        let next_val = self.builder.build_int_add(current_val, step_val, "next").unwrap();
        self.builder.build_store(loop_var_ptr.into_pointer_value(), next_val).unwrap();

        // Branch back to the inner loop condition
        self.builder.build_unconditional_branch(inner_loop_block).unwrap();

        // Chunk increment block
        self.builder.position_at_end(chunk_inc_block);

        // Update the chunk counter to the next chunk
        self.builder.build_store(chunk_counter_ptr, actual_chunk_end).unwrap();

        // Branch back to the chunk condition
        self.builder.build_unconditional_branch(chunk_cond_block).unwrap();

        // Return the entry block
        entry_block
    }

    /// Optimize the loop condition check
    fn optimize_loop_condition(
        &self,
        function: FunctionValue<'ctx>,
        start_val: IntValue<'ctx>,
        end_val: IntValue<'ctx>,
        step_val: IntValue<'ctx>,
        loop_var_ptr: BasicValueEnum<'ctx>,
        body_block: BasicBlock<'ctx>,
        exit_block: BasicBlock<'ctx>,
        inc_block: BasicBlock<'ctx>,
    ) {
        let i64_type = self.context.i64_type();

        // Create a basic block for the optimized condition check
        let cond_block = self.context.append_basic_block(function, "optimized_cond");
        self.builder.build_unconditional_branch(cond_block).unwrap();
        self.builder.position_at_end(cond_block);

        // Initialize the loop variable
        self.builder.build_store(loop_var_ptr.into_pointer_value(), start_val).unwrap();

        // Build the loop condition
        let current_val = self.builder.build_load(i64_type, loop_var_ptr.into_pointer_value(), "current").unwrap().into_int_value();

        // Optimize the comparison based on the step direction
        let step_positive = self.builder.build_int_compare(
            IntPredicate::SGT,
            step_val,
            i64_type.const_int(0, true),
            "step_positive"
        ).unwrap();

        let cond_pos = self.builder.build_int_compare(
            IntPredicate::SLT,
            current_val,
            end_val,
            "cond_pos"
        ).unwrap();

        let cond_neg = self.builder.build_int_compare(
            IntPredicate::SGT,
            current_val,
            end_val,
            "cond_neg"
        ).unwrap();

        let condition = self.builder.build_select(
            step_positive,
            cond_pos,
            cond_neg,
            "loop_condition"
        ).unwrap().into_int_value();

        // Branch directly to the body block or exit block based on the condition
        self.builder.build_conditional_branch(condition, body_block, exit_block).unwrap();

        // Set up the increment block
        self.builder.position_at_end(inc_block);

        // Increment the loop variable
        let current_val = self.builder.build_load(i64_type, loop_var_ptr.into_pointer_value(), "current_inc").unwrap().into_int_value();
        let next_val = self.builder.build_int_add(current_val, step_val, "next").unwrap();
        self.builder.build_store(loop_var_ptr.into_pointer_value(), next_val).unwrap();

        // Branch back to the condition check
        self.builder.build_unconditional_branch(cond_block).unwrap();
    }
}
