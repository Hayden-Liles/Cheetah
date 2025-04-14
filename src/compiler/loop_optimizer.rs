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

// Constants for loop unrolling
const UNROLL_THRESHOLD: u64 = 0; // Maximum number of iterations to fully unroll
const PARTIAL_UNROLL_FACTOR: u64 = 1; // Unroll factor for partial unrolling

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

        // Check if we can unroll this loop
        if let Some(unrolled_entry) = self.try_unroll_loop(
            function,
            start_val,
            end_val,
            step_val,
            loop_var_ptr,
            body_block,
            exit_block,
        ) {
            return unrolled_entry;
        }

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

    /// Try to unroll a loop if it has constant bounds and a small iteration count
    /// Returns Some(entry_block) if unrolling was successful, None otherwise
    fn try_unroll_loop(
        &self,
        function: FunctionValue<'ctx>,
        start_val: IntValue<'ctx>,
        end_val: IntValue<'ctx>,
        step_val: IntValue<'ctx>,
        loop_var_ptr: BasicValueEnum<'ctx>,
        body_block: BasicBlock<'ctx>,
        exit_block: BasicBlock<'ctx>,
    ) -> Option<BasicBlock<'ctx>> {
        // Only unroll loops with constant bounds and step
        if let (Some(start_const), Some(end_const), Some(step_const)) = (
            start_val.get_sign_extended_constant(),
            end_val.get_sign_extended_constant(),
            step_val.get_sign_extended_constant(),
        ) {
            // Make sure step is not zero to avoid infinite loops
            if step_const == 0 {
                eprintln!("[LOOP UNROLL] Skipping loop with zero step");
                return None;
            }

            // Calculate the number of iterations
            let num_iterations = if step_const > 0 && end_const > start_const {
                (end_const - start_const + step_const - 1) / step_const
            } else if step_const < 0 && start_const > end_const {
                (start_const - end_const - step_const - 1) / (-step_const)
            } else {
                // Invalid loop bounds or step direction
                eprintln!("[LOOP UNROLL] Skipping loop with invalid bounds or step direction: start={}, end={}, step={}",
                         start_const, end_const, step_const);
                return None;
            };

            // Convert to u64 for comparison with threshold
            let num_iterations_u64 = num_iterations as u64;
            eprintln!("[LOOP UNROLL] Loop has {} iterations (threshold for full unrolling: {})",
                     num_iterations_u64, UNROLL_THRESHOLD);

            // Only fully unroll small loops
            if num_iterations_u64 <= UNROLL_THRESHOLD && num_iterations_u64 > 0 {
                eprintln!("[LOOP UNROLL] Fully unrolling loop with {} iterations", num_iterations_u64);
                return Some(self.fully_unroll_loop(
                    function,
                    start_val,
                    step_val,
                    loop_var_ptr,
                    body_block,
                    exit_block,
                    num_iterations_u64,
                ));
            }

            // For larger loops, consider partial unrolling
            if num_iterations_u64 > UNROLL_THRESHOLD && num_iterations_u64 % PARTIAL_UNROLL_FACTOR == 0 {
                eprintln!("[LOOP UNROLL] Partially unrolling loop with {} iterations (factor: {})",
                         num_iterations_u64, PARTIAL_UNROLL_FACTOR);
                return Some(self.partially_unroll_loop(
                    function,
                    start_val,
                    end_val,
                    step_val,
                    loop_var_ptr,
                    body_block,
                    exit_block,
                    PARTIAL_UNROLL_FACTOR,
                ));
            }

            eprintln!("[LOOP UNROLL] Not unrolling loop with {} iterations", num_iterations_u64);
        }

        // Cannot unroll this loop
        None
    }

    /// Fully unroll a loop with a known iteration count
    fn fully_unroll_loop(
        &self,
        function: FunctionValue<'ctx>,
        start_val: IntValue<'ctx>,
        step_val: IntValue<'ctx>,
        loop_var_ptr: BasicValueEnum<'ctx>,
        body_block: BasicBlock<'ctx>,
        exit_block: BasicBlock<'ctx>,
        num_iterations: u64,
    ) -> BasicBlock<'ctx> {
        // We don't need i64_type for fully unrolled loops
        let entry_block = self.builder.get_insert_block().unwrap();

        // Create a sequence of blocks for each iteration
        let mut current_val = start_val;
        let mut current_block = entry_block;

        eprintln!("[LOOP UNROLL] Fully unrolling loop with {} iterations", num_iterations);

        for i in 0..num_iterations {
            // Create a new block for this iteration
            let iter_block = self.context.append_basic_block(function, &format!("unrolled_iter_{}", i));

            // Branch from the current block to this iteration's block
            self.builder.position_at_end(current_block);
            self.builder.build_unconditional_branch(iter_block).unwrap();

            // Position at the start of this iteration's block
            self.builder.position_at_end(iter_block);

            // Set the loop variable to the current value
            self.builder.build_store(loop_var_ptr.into_pointer_value(), current_val).unwrap();

            // Branch to the body block
            self.builder.build_unconditional_branch(body_block).unwrap();

            // Create a continuation block for after the body
            let cont_block = self.context.append_basic_block(function, &format!("unrolled_cont_{}", i));

            // Position at the end of the body block
            // We need to save the current terminator if it exists
            let body_terminator = body_block.get_terminator();

            // If the body block already has a terminator, we need to create a new block
            if body_terminator.is_some() {
                // Create a new body block for this iteration
                let new_body_block = self.context.append_basic_block(function, &format!("unrolled_body_{}", i));

                // Copy the instructions from the original body block to the new one
                // This is a simplified approach - in a real implementation, you'd need to
                // properly clone all instructions and update references

                // For now, we'll just branch to the original body and then to our continuation
                self.builder.position_at_end(iter_block);
                self.builder.build_unconditional_branch(new_body_block).unwrap();

                self.builder.position_at_end(new_body_block);
                self.builder.build_unconditional_branch(cont_block).unwrap();
            } else {
                // Position at the end of the body block
                self.builder.position_at_end(body_block);

                // Branch to the continuation block
                self.builder.build_unconditional_branch(cont_block).unwrap();
            }

            // Position at the continuation block
            self.builder.position_at_end(cont_block);

            // Update the current value for the next iteration
            current_val = self.builder.build_int_add(current_val, step_val, &format!("unrolled_next_{}", i)).unwrap();

            // Update the current block for the next iteration
            current_block = cont_block;
        }

        // After all iterations, branch to the exit block
        self.builder.position_at_end(current_block);
        self.builder.build_unconditional_branch(exit_block).unwrap();

        // Return the entry block
        entry_block
    }

    /// Partially unroll a loop by a factor
    fn partially_unroll_loop(
        &self,
        function: FunctionValue<'ctx>,
        start_val: IntValue<'ctx>,
        end_val: IntValue<'ctx>,
        step_val: IntValue<'ctx>,
        loop_var_ptr: BasicValueEnum<'ctx>,
        body_block: BasicBlock<'ctx>,
        exit_block: BasicBlock<'ctx>,
        unroll_factor: u64,
    ) -> BasicBlock<'ctx> {
        let i64_type = self.context.i64_type();
        let entry_block = self.builder.get_insert_block().unwrap();

        eprintln!("[LOOP UNROLL] Partially unrolling loop with factor {}", unroll_factor);

        // Create blocks for the unrolled loop
        let header_block = self.context.append_basic_block(function, "unroll_header");
        let body_start_block = self.context.append_basic_block(function, "unroll_body_start");
        let inc_block = self.context.append_basic_block(function, "unroll_inc");

        // Branch to the header block
        self.builder.position_at_end(entry_block);
        self.builder.build_unconditional_branch(header_block).unwrap();

        // Header block - initialize loop variable and check condition
        self.builder.position_at_end(header_block);
        self.builder.build_store(loop_var_ptr.into_pointer_value(), start_val).unwrap();

        // Load the current value and check if we should enter the loop
        let current_val = self.builder.build_load(i64_type, loop_var_ptr.into_pointer_value(), "current").unwrap().into_int_value();

        // Build the loop condition
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

        // Branch to the body or exit based on the condition
        self.builder.build_conditional_branch(condition, body_start_block, exit_block).unwrap();

        // Body start block
        self.builder.position_at_end(body_start_block);

        // Create blocks for each unrolled iteration
        let mut current_block = body_start_block;

        // Create unrolled iterations
        for i in 0..unroll_factor {
            // Branch to the body block
            self.builder.position_at_end(current_block);

            // Check if we should execute this iteration
            if i > 0 {
                // For iterations after the first, we need to check if we're still in bounds
                let iter_val = self.builder.build_load(i64_type, loop_var_ptr.into_pointer_value(), &format!("iter_val_{}", i)).unwrap().into_int_value();

                let iter_cond_pos = self.builder.build_int_compare(
                    IntPredicate::SLT,
                    iter_val,
                    end_val,
                    &format!("iter_cond_pos_{}", i)
                ).unwrap();

                let iter_cond_neg = self.builder.build_int_compare(
                    IntPredicate::SGT,
                    iter_val,
                    end_val,
                    &format!("iter_cond_neg_{}", i)
                ).unwrap();

                let iter_condition = self.builder.build_select(
                    step_positive,
                    iter_cond_pos,
                    iter_cond_neg,
                    &format!("iter_condition_{}", i)
                ).unwrap().into_int_value();

                // Create a block for this iteration's body
                let iter_body_block = self.context.append_basic_block(function, &format!("unroll_iter_body_{}", i));
                let iter_skip_block = self.context.append_basic_block(function, &format!("unroll_iter_skip_{}", i));

                // Branch to the body or skip based on the condition
                self.builder.build_conditional_branch(iter_condition, iter_body_block, iter_skip_block).unwrap();

                // Body block for this iteration
                self.builder.position_at_end(iter_body_block);

                // Branch to the original body block
                self.builder.build_unconditional_branch(body_block).unwrap();

                // Create a continuation block for after the body
                let iter_cont_block = self.context.append_basic_block(function, &format!("unroll_iter_cont_{}", i));

                // Position at the end of the body block
                // We need to save the current terminator if it exists
                let body_terminator = body_block.get_terminator();

                if body_terminator.is_some() {
                    // Create a new body block for this iteration
                    let new_body_block = self.context.append_basic_block(function, &format!("unroll_iter_new_body_{}", i));

                    // Branch to the new body block
                    self.builder.position_at_end(iter_body_block);
                    self.builder.build_unconditional_branch(new_body_block).unwrap();

                    // Position at the new body block
                    self.builder.position_at_end(new_body_block);

                    // Branch to the continuation block
                    self.builder.build_unconditional_branch(iter_cont_block).unwrap();
                } else {
                    // Position at the end of the body block
                    self.builder.position_at_end(body_block);

                    // Branch to the continuation block
                    self.builder.build_unconditional_branch(iter_cont_block).unwrap();
                }

                // Position at the continuation block
                self.builder.position_at_end(iter_cont_block);

                // Increment the loop variable
                let iter_current_val = self.builder.build_load(i64_type, loop_var_ptr.into_pointer_value(), &format!("iter_current_{}", i)).unwrap().into_int_value();
                let iter_next_val = self.builder.build_int_add(iter_current_val, step_val, &format!("iter_next_{}", i)).unwrap();
                self.builder.build_store(loop_var_ptr.into_pointer_value(), iter_next_val).unwrap();

                // Branch to the skip block
                self.builder.build_unconditional_branch(iter_skip_block).unwrap();

                // Skip block for this iteration
                self.builder.position_at_end(iter_skip_block);

                // Update the current block for the next iteration
                current_block = iter_skip_block;
            } else {
                // For the first iteration, we already know we're in bounds
                // Branch to the original body block
                self.builder.build_unconditional_branch(body_block).unwrap();

                // Create a continuation block for after the body
                let iter_cont_block = self.context.append_basic_block(function, "unroll_iter_cont_0");

                // Position at the end of the body block
                // We need to save the current terminator if it exists
                let body_terminator = body_block.get_terminator();

                if body_terminator.is_some() {
                    // Create a new body block for this iteration
                    let new_body_block = self.context.append_basic_block(function, "unroll_iter_new_body_0");

                    // Branch to the new body block
                    self.builder.position_at_end(current_block);
                    self.builder.build_unconditional_branch(new_body_block).unwrap();

                    // Position at the new body block
                    self.builder.position_at_end(new_body_block);

                    // Branch to the continuation block
                    self.builder.build_unconditional_branch(iter_cont_block).unwrap();
                } else {
                    // Position at the end of the body block
                    self.builder.position_at_end(body_block);

                    // Branch to the continuation block
                    self.builder.build_unconditional_branch(iter_cont_block).unwrap();
                }

                // Position at the continuation block
                self.builder.position_at_end(iter_cont_block);

                // Increment the loop variable
                let iter_current_val = self.builder.build_load(i64_type, loop_var_ptr.into_pointer_value(), "iter_current_0").unwrap().into_int_value();
                let iter_next_val = self.builder.build_int_add(iter_current_val, step_val, "iter_next_0").unwrap();
                self.builder.build_store(loop_var_ptr.into_pointer_value(), iter_next_val).unwrap();

                // Update the current block for the next iteration
                current_block = iter_cont_block;
            }
        }

        // After all unrolled iterations, branch to the increment block
        self.builder.position_at_end(current_block);
        self.builder.build_unconditional_branch(inc_block).unwrap();

        // Increment block - increment by (unroll_factor * step) and branch back to header
        self.builder.position_at_end(inc_block);

        // Calculate the step for the unrolled loop and update the loop variable
        let current_val = self.builder.build_load(i64_type, loop_var_ptr.into_pointer_value(), "current_unrolled").unwrap().into_int_value();
        let unrolled_step = self.builder.build_int_mul(
            step_val,
            i64_type.const_int(unroll_factor, false),
            "unrolled_step"
        ).unwrap();
        let next_val = self.builder.build_int_add(current_val, unrolled_step, "next_unrolled").unwrap();
        self.builder.build_store(loop_var_ptr.into_pointer_value(), next_val).unwrap();

        // Branch back to the header block
        self.builder.build_unconditional_branch(header_block).unwrap();

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
