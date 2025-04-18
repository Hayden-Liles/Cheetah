// parallel_loop_optimizer.rs - Optimizations for parallel loops using Rayon
// This file implements parallel loop optimizations for better performance

use inkwell::basic_block::BasicBlock;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::values::{BasicValueEnum, FunctionValue, IntValue};

// Constants for parallel loop optimization
const MIN_PARALLEL_SIZE: u64 = 1000;
// Removed unused constant LARGE_PARALLEL_THRESHOLD

/// Parallel loop optimization helper functions
pub struct ParallelLoopOptimizer<'ctx> {
    builder: &'ctx Builder<'ctx>,
    context: &'ctx Context,
    module: &'ctx inkwell::module::Module<'ctx>,
}

impl<'ctx> ParallelLoopOptimizer<'ctx> {
    /// Create a new parallel loop optimizer
    pub fn new(
        builder: &'ctx Builder<'ctx>,
        context: &'ctx Context,
        module: &'ctx inkwell::module::Module<'ctx>,
    ) -> Self {
        Self {
            builder,
            context,
            module,
        }
    }

    /// Check if a loop should be parallelized based on its range
    pub fn should_parallelize(&self, start_val: IntValue<'ctx>, end_val: IntValue<'ctx>) -> bool {
        if let (Some(start_const), Some(end_const)) = (
            start_val.get_sign_extended_constant(),
            end_val.get_sign_extended_constant(),
        ) {
            let range_size = if end_const > start_const {
                (end_const - start_const) as u64
            } else {
                0
            };

            range_size >= MIN_PARALLEL_SIZE
        } else {
            false
        }
    }

    /// Create a parallelized loop using Rayon
    pub fn create_parallel_loop(
        &self,
        function: FunctionValue<'ctx>,
        start_val: IntValue<'ctx>,
        end_val: IntValue<'ctx>,
        step_val: IntValue<'ctx>,
        _loop_var_ptr: BasicValueEnum<'ctx>,
        body_block: BasicBlock<'ctx>,
        exit_block: BasicBlock<'ctx>,
    ) -> BasicBlock<'ctx> {
        let i64_type = self.context.i64_type();
        let entry_block = self.builder.get_insert_block().unwrap();

        let parallel_block = self.context.append_basic_block(function, "parallel_loop");

        self.builder
            .build_unconditional_branch(parallel_block)
            .unwrap();

        self.builder.position_at_end(parallel_block);

        if cfg!(debug_assertions) {
            if let (Some(start_const), Some(end_const)) = (
                start_val.get_sign_extended_constant(),
                end_val.get_sign_extended_constant(),
            ) {
                let range_size = if end_const > start_const {
                    (end_const - start_const) as u64
                } else {
                    0
                };

                println!(
                    "[PARALLEL LOOP] Using parallel loop for range size: {}",
                    range_size
                );
            } else {
                println!("[PARALLEL LOOP] Using parallel loop for dynamic range");
            }
        }

        let loop_body_fn_type = self.context.void_type().fn_type(&[i64_type.into()], false);

        let loop_body_fn = self.module.add_function(
            &format!(
                "parallel_loop_body_{}",
                function.get_name().to_str().unwrap_or("unknown")
            ),
            loop_body_fn_type,
            None,
        );

        let loop_body_entry = self.context.append_basic_block(loop_body_fn, "entry");

        let current_block = self.builder.get_insert_block().unwrap();

        self.builder.position_at_end(loop_body_entry);

        let loop_var = loop_body_fn.get_first_param().unwrap().into_int_value();

        let global_loop_var_ptr = self.module.add_global(
            i64_type,
            None,
            &format!(
                "global_loop_var_{}",
                function.get_name().to_str().unwrap_or("unknown")
            ),
        );
        global_loop_var_ptr.set_initializer(&i64_type.const_zero());

        self.builder
            .build_store(global_loop_var_ptr.as_pointer_value(), loop_var)
            .unwrap();

        self.builder.build_unconditional_branch(body_block).unwrap();

        self.builder.position_at_end(current_block);

        let parallel_range_for_each = self
            .module
            .get_function("parallel_range_for_each")
            .unwrap_or_else(|| {
                let parallel_range_for_each_type = self.context.void_type().fn_type(
                    &[
                        i64_type.into(),
                        i64_type.into(),
                        i64_type.into(),
                        self.context
                            .ptr_type(inkwell::AddressSpace::default())
                            .into(),
                    ],
                    false,
                );
                self.module.add_function(
                    "parallel_range_for_each",
                    parallel_range_for_each_type,
                    None,
                )
            });

        self.builder
            .build_call(
                parallel_range_for_each,
                &[
                    start_val.into(),
                    end_val.into(),
                    step_val.into(),
                    loop_body_fn.as_global_value().as_pointer_value().into(),
                ],
                "parallel_range_call",
            )
            .unwrap();

        self.builder.build_unconditional_branch(exit_block).unwrap();

        entry_block
    }
}
