// optimized_loop_optimizer.rs - High-performance loop optimizations

use inkwell::basic_block::BasicBlock;
use inkwell::values::{BasicValueEnum, IntValue, FunctionValue};
use inkwell::IntPredicate;
use inkwell::builder::Builder;
use inkwell::context::Context;

// Import memory profiler for dynamic chunk sizing
use crate::compiler::runtime::memory_profiler;

// Constants for loop optimization
const MIN_CHUNK_SIZE: u64 = 5000; // Minimum chunk size (increased from 1000)
const MAX_CHUNK_SIZE: u64 = 200000; // Maximum chunk size (increased from 100000)
const DEFAULT_CHUNK_SIZE: u64 = 50000; // Default chunk size (increased from 10000)

// Threshold for large ranges that need special handling
const LARGE_RANGE_THRESHOLD: u64 = 1000000; // 1 million iterations
const VERY_LARGE_RANGE_THRESHOLD: u64 = 10000000; // 10 million iterations

// Constants for adaptive chunk sizing
const ADAPTIVE_CHUNK_FACTOR: f64 = 0.8; // Factor to adjust chunk size (increased from 0.75)
const MEMORY_SCALING_FACTOR: f64 = 0.6; // Factor to scale chunk size (increased from 0.5)
const SYSTEM_MEMORY_THRESHOLD: f64 = 0.8; // Memory usage threshold (80%)

// Constants for loop unrolling
const UNROLL_THRESHOLD: u64 = 16; // Maximum number of iterations to fully unroll (increased from 0)
const PARTIAL_UNROLL_FACTOR: u64 = 4; // Unroll factor for partial unrolling (increased from 1)

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
    pub fn should_chunk_loop(&self, start_val: IntValue<'ctx>, end_val: IntValue<'ctx>) -> bool {
        // Get the range size if possible
        if let (Some(start_const), Some(end_const)) = (start_val.get_sign_extended_constant(), end_val.get_sign_extended_constant()) {
            if end_const > start_const {
                let range_size = (end_const - start_const) as u64;
                // Only chunk loops larger than MIN_CHUNK_SIZE
                return range_size > MIN_CHUNK_SIZE;
            }
        }
        
        // By default, don't chunk small loops for better performance
        false
    }

    /// Calculate an appropriate chunk size based on the range size and system conditions
    pub fn calculate_chunk_size(&self, start_val: IntValue<'ctx>, end_val: IntValue<'ctx>) -> u64 {
        // Try to get constant values if available
        let range_size = if let (Some(start_const), Some(end_const)) = (start_val.get_sign_extended_constant(), end_val.get_sign_extended_constant()) {
            if end_const > start_const {
                (end_const - start_const) as u64
            } else {
                return MIN_CHUNK_SIZE; // Invalid range, use minimum chunk size
            }
        } else {
            // If we can't determine the range size statically, use a default size
            return self.adjust_chunk_size_for_memory(DEFAULT_CHUNK_SIZE);
        };

        // Get the base chunk size based on range size
        let base_chunk_size = self.get_base_chunk_size(range_size);

        // Adjust the chunk size based on current memory usage
        let memory_adjusted_size = self.adjust_chunk_size_for_memory(base_chunk_size);

        // Apply complexity adjustment if we can estimate it
        let final_chunk_size = self.adjust_for_complexity(memory_adjusted_size);

        // Log the chunk size calculation for debugging
        if cfg!(debug_assertions) {
            println!("[CHUNK SIZE] Range: {}, Base: {}, Memory-adjusted: {}, Final: {}",
                     range_size, base_chunk_size, memory_adjusted_size, final_chunk_size);
        }

        final_chunk_size
    }

    /// Get the base chunk size based on the range size
    fn get_base_chunk_size(&self, range_size: u64) -> u64 {
        // For extremely large ranges, use a fixed large chunk size
        if range_size > VERY_LARGE_RANGE_THRESHOLD {
            MAX_CHUNK_SIZE
        }
        // For large ranges, use a dynamic chunk size
        else if range_size > LARGE_RANGE_THRESHOLD {
            // Use a square root scale for large ranges
            // Increased multiplier for better performance
            let sqrt_factor = (range_size as f64).sqrt() as u64 / 50;
            let adjusted_size = DEFAULT_CHUNK_SIZE * sqrt_factor.max(1);
            adjusted_size.clamp(MIN_CHUNK_SIZE, MAX_CHUNK_SIZE)
        }
        // For medium-large ranges, use optimal chunk size
        else if range_size > MAX_CHUNK_SIZE {
            // Use a chunk size that's a power of 2 for better performance
            let power_of_two = (range_size as f64).log2().floor();
            (2.0f64.powf(power_of_two) as u64).clamp(MIN_CHUNK_SIZE, MAX_CHUNK_SIZE)
        }
        // For medium ranges, use the range size itself
        else if range_size > MIN_CHUNK_SIZE {
            range_size
        }
        // For small ranges, use minimum chunk size
        else {
            MIN_CHUNK_SIZE
        }
    }

    /// Adjust chunk size based on current memory usage
    fn adjust_chunk_size_for_memory(&self, chunk_size: u64) -> u64 {
        // Get current memory usage from the memory profiler
        let current_memory = memory_profiler::get_current_memory_usage() as f64;
        let peak_memory = memory_profiler::get_peak_memory_usage() as f64;

        // If we don't have meaningful memory data, return the original size
        if current_memory == 0.0 || peak_memory == 0.0 {
            return chunk_size;
        }

        // Calculate memory usage ratio
        let memory_ratio = current_memory / peak_memory;

        // If memory usage is high, reduce chunk size to prevent excessive memory usage
        if memory_ratio > SYSTEM_MEMORY_THRESHOLD {
            // Scale down chunk size based on how close we are to peak memory
            let scale_factor = 1.0 - ((memory_ratio - SYSTEM_MEMORY_THRESHOLD) / (1.0 - SYSTEM_MEMORY_THRESHOLD));
            let scaled_size = (chunk_size as f64 * scale_factor * MEMORY_SCALING_FACTOR) as u64;

            // Ensure we don't go below the minimum chunk size
            scaled_size.max(MIN_CHUNK_SIZE)
        } else {
            // Memory usage is acceptable, use the original chunk size
            chunk_size
        }
    }

    /// Adjust chunk size based on estimated loop complexity
    fn adjust_for_complexity(&self, chunk_size: u64) -> u64 {
        // Apply a conservative adjustment using the adaptive chunk factor
        let adjusted_size = (chunk_size as f64 * ADAPTIVE_CHUNK_FACTOR) as u64;

        // Ensure we don't go below the minimum chunk size
        // And make sure the chunk size is a multiple of PARTIAL_UNROLL_FACTOR for better performance
        let remainder = adjusted_size % PARTIAL_UNROLL_FACTOR;
        let size = if remainder == 0 {
            adjusted_size
        } else {
            adjusted_size + (PARTIAL_UNROLL_FACTOR - remainder)
        };
        
        size.max(MIN_CHUNK_SIZE)
    }

    /// Optimize a range-based for loop
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

        // For smaller loops, apply basic optimizations like strength reduction
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
            // Get range size for logging if possible
            let range_size_str = if let (Some(start_const), Some(end_const)) = (start_val.get_sign_extended_constant(), end_val.get_sign_extended_constant()) {
                format!("{}", (end_const - start_const) as u64)
            } else {
                "unknown".to_string()
            };

            println!("[LOOP CHUNKING] Using dynamic chunk size: {} for loop with range size: {}",
                     dynamic_chunk_size, range_size_str);

            // Log memory usage if available
            let current_memory = memory_profiler::get_current_memory_usage();
            let peak_memory = memory_profiler::get_peak_memory_usage();
            if current_memory > 0 && peak_memory > 0 {
                println!("[LOOP MEMORY] Current memory: {:.2} MB, Peak: {:.2} MB, Usage ratio: {:.2}%",
                         current_memory as f64 / (1024.0 * 1024.0),
                         peak_memory as f64 / (1024.0 * 1024.0),
                         (current_memory as f64 / peak_memory as f64) * 100.0);
            }
        }

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

        // Increment the loop variable - use step size
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
            
            // Only fully unroll very small loops - this prevents code bloat
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

            // For medium-sized loops that are multiples of the unroll factor, use partial unrolling
            if num_iterations_u64 > UNROLL_THRESHOLD && num_iterations_u64 <= 500 && num_iterations_u64 % PARTIAL_UNROLL_FACTOR == 0 {
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
        let entry_block = self.builder.get_insert_block().unwrap();

        // Create a sequence of blocks for each iteration
        let mut current_val = start_val;
        let mut current_block = entry_block;

        // Loop through each iteration and create the unrolled IR
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

            // If the body block already has a terminator, we need to handle it specially
            if body_terminator.is_some() {
                // Create a new body block for this iteration
                let new_body_block = self.context.append_basic_block(function, &format!("unrolled_body_{}", i));

                // Position at the start of the new body block
                self.builder.position_at_end(new_body_block);
                
                // Branch to the continuation block
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

        // Create blocks for the main loop
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

        // Compute the step size for the unrolled iterations
        let unrolled_step = self.builder.build_int_mul(
            step_val,
            i64_type.const_int(unroll_factor, false),
            "unrolled_step"
        ).unwrap();

        // Create unrolled iterations - this is the key to performance
        let mut current_block = body_start_block;
        
        for i in 0..unroll_factor {
            // For each unrolled iteration, compute the current value
            let current_index = if i == 0 {
                current_val
            } else {
                let offset = self.builder.build_int_mul(
                    step_val, 
                    i64_type.const_int(i, false),
                    &format!("offset_{}", i)
                ).unwrap();
                
                self.builder.build_int_add(
                    current_val,
                    offset,
                    &format!("unrolled_val_{}", i)
                ).unwrap()
            };
            
            // Store the current iteration's value
            self.builder.build_store(loop_var_ptr.into_pointer_value(), current_index).unwrap();
            
            // Branch to the body block
            let body_cont_block = self.context.append_basic_block(function, &format!("body_cont_{}", i));
            
            self.builder.build_unconditional_branch(body_block).unwrap();
            
            // Setup for continuation after the body
            self.builder.position_at_end(body_cont_block);
            
            // If this is the last iteration, branch to increment block
            if i == unroll_factor - 1 {
                self.builder.build_unconditional_branch(inc_block).unwrap();
            }
            
            // Update the current block for the next iteration
            current_block = body_cont_block;
        }

        // Increment block - increment by (unroll_factor * step) and branch back to header
        self.builder.position_at_end(inc_block);

        // Load the current value of the loop variable
        let current_val = self.builder.build_load(i64_type, loop_var_ptr.into_pointer_value(), "current_at_end").unwrap().into_int_value();
        
        // Add the unrolled step to the loop variable
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