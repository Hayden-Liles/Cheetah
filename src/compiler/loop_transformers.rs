use inkwell::{
    basic_block::BasicBlock,
    builder::Builder,
    context::Context,
    module::Module,
    values::{BasicValueEnum, FunctionValue, IntValue},
    IntPredicate,
};
use crate::compiler::runtime::memory_profiler;

// === Loop Flattening ===

/// Flattens nested loops to prevent stack overflow
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
                if let Some(target) = inst.get_operand(0) {
                    if let Some(block) = target.right() {
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

    /// Flatten a nested loop into a single loop with state variables
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
        _body_block: BasicBlock<'ctx>,
        exit_block: BasicBlock<'ctx>,
    ) -> BasicBlock<'ctx> {
        let i64_t = self.context.i64_type();
        let entry = self.builder.get_insert_block().unwrap();

        let init_bb = self.context.append_basic_block(function, "flat_init");
        let cond_bb = self.context.append_basic_block(function, "flat_cond");
        let body_bb = self.context.append_basic_block(function, "flat_body");
        let inc_bb = self.context.append_basic_block(function, "flat_inc");

        self.builder.build_unconditional_branch(init_bb).unwrap();
        self.builder.position_at_end(init_bb);

        let outer_ptr = self.builder.build_alloca(i64_t, "outer_ptr").unwrap();
        let inner_ptr = self.builder.build_alloca(i64_t, "inner_ptr").unwrap();
        self.builder.build_store(outer_ptr, outer_start).unwrap();
        self.builder.build_store(inner_ptr, inner_start).unwrap();
        self.builder.build_unconditional_branch(cond_bb).unwrap();

        self.builder.position_at_end(cond_bb);
        let outer_val = self.builder.build_load(i64_t, outer_ptr, "ov").unwrap().into_int_value();
        let inner_val = self.builder.build_load(i64_t, inner_ptr, "iv").unwrap().into_int_value();

        let oc = self.builder
            .build_int_compare(IntPredicate::SLT, outer_val, outer_end, "oc")
            .unwrap();
        let ic = self.builder
            .build_int_compare(IntPredicate::SLT, inner_val, inner_end, "ic")
            .unwrap();
        let cc = self.builder.build_and(oc, ic, "cc").unwrap();
        self.builder.build_conditional_branch(cc, body_bb, exit_block).unwrap();

        self.builder.position_at_end(body_bb);
        self.builder.build_store(outer_loop_var.into_pointer_value(), outer_val).unwrap();
        self.builder.build_store(inner_loop_var.into_pointer_value(), inner_val).unwrap();
        self.builder.build_unconditional_branch(inc_bb).unwrap();

        self.builder.position_at_end(inc_bb);
        let next_inner = self.builder.build_int_add(inner_val, inner_step, "ni").unwrap();
        self.builder.build_store(inner_ptr, next_inner).unwrap();

        let done_inner = self.builder
            .build_int_compare(IntPredicate::SGE, next_inner, inner_end, "done_i")
            .unwrap();
        let reset_bb = self.context.append_basic_block(function, "reset_i");
        let cont_bb = self.context.append_basic_block(function, "cont");

        self.builder.build_conditional_branch(done_inner, reset_bb, cont_bb).unwrap();
        self.builder.position_at_end(reset_bb);
        self.builder.build_store(inner_ptr, inner_start).unwrap();
        let next_outer = self.builder.build_int_add(outer_val, outer_step, "no").unwrap();
        self.builder.build_store(outer_ptr, next_outer).unwrap();
        self.builder.build_unconditional_branch(cont_bb).unwrap();

        self.builder.position_at_end(cont_bb);
        self.builder.build_unconditional_branch(cond_bb).unwrap();

        entry
    }
}

// === Loop Optimization ===

const MIN_CHUNK_SIZE: u64 = 5000;
const MAX_CHUNK_SIZE: u64 = 200000;
const DEFAULT_CHUNK_SIZE: u64 = 50000;
const LARGE_RANGE_THRESHOLD: u64 = 1000000;
const VERY_LARGE_RANGE_THRESHOLD: u64 = 10000000;
const ADAPTIVE_CHUNK_FACTOR: f64 = 0.8;
const MEMORY_SCALING_FACTOR: f64 = 0.6;
const SYSTEM_MEMORY_THRESHOLD: f64 = 0.8;
const UNROLL_THRESHOLD: u64 = 16;
const PARTIAL_UNROLL_FACTOR: u64 = 4;

/// High-performance loop optimizer
pub struct LoopOptimizer<'ctx> {
    builder: &'ctx Builder<'ctx>,
    context: &'ctx Context,
}

impl<'ctx> LoopOptimizer<'ctx> {
    pub fn new(builder: &'ctx Builder<'ctx>, context: &'ctx Context) -> Self {
        Self { builder, context }
    }

    /// Decide if chunking is needed
    pub fn should_chunk_loop(&self, start: IntValue<'ctx>, end: IntValue<'ctx>) -> bool {
        if let (Some(s), Some(e)) = (start.get_sign_extended_constant(), end.get_sign_extended_constant()) {
            return e > s && (e - s) as u64 > MIN_CHUNK_SIZE;
        }
        false
    }

    /// Compute chunk size
    pub fn calculate_chunk_size(&self, start: IntValue<'ctx>, end: IntValue<'ctx>) -> u64 {
        let range = if let (Some(s), Some(e)) = (start.get_sign_extended_constant(), end.get_sign_extended_constant()) {
            if e > s { (e - s) as u64 } else { 0 }
        } else {
            return self.adjust_chunk_size_for_memory(DEFAULT_CHUNK_SIZE);
        };

        let base = self.get_base_chunk_size(range);
        let mem_adj = self.adjust_chunk_size_for_memory(base);
        self.adjust_for_complexity(mem_adj)
    }

    fn get_base_chunk_size(&self, range: u64) -> u64 {
        if range > VERY_LARGE_RANGE_THRESHOLD {
            MAX_CHUNK_SIZE
        } else if range > LARGE_RANGE_THRESHOLD {
            let sqrt = ((range as f64).sqrt() as u64).max(1) / 50;
            (DEFAULT_CHUNK_SIZE * sqrt).clamp(MIN_CHUNK_SIZE, MAX_CHUNK_SIZE)
        } else if range > MAX_CHUNK_SIZE {
            let pow2 = (range as f64).log2().floor() as u32;
            (2u64.pow(pow2)).clamp(MIN_CHUNK_SIZE, MAX_CHUNK_SIZE)
        } else if range > MIN_CHUNK_SIZE {
            range
        } else {
            MIN_CHUNK_SIZE
        }
    }

    fn adjust_chunk_size_for_memory(&self, size: u64) -> u64 {
        let cur = memory_profiler::get_current_memory_usage() as f64;
        let peak = memory_profiler::get_peak_memory_usage() as f64;
        if cur == 0.0 || peak == 0.0 { return size; }
        let ratio = cur / peak;
        if ratio > SYSTEM_MEMORY_THRESHOLD {
            let scale = 1.0 - ((ratio - SYSTEM_MEMORY_THRESHOLD) / (1.0 - SYSTEM_MEMORY_THRESHOLD));
            ((size as f64 * scale * MEMORY_SCALING_FACTOR) as u64).max(MIN_CHUNK_SIZE)
        } else {
            size
        }
    }

    fn adjust_for_complexity(&self, size: u64) -> u64 {
        let adj = (size as f64 * ADAPTIVE_CHUNK_FACTOR) as u64;
        let rem = adj % PARTIAL_UNROLL_FACTOR;
        let final_size = if rem == 0 { adj } else { adj + (PARTIAL_UNROLL_FACTOR - rem) };
        final_size.max(MIN_CHUNK_SIZE)
    }

    /// Main entry: unroll, then chunk, then simple optimize
    pub fn optimize_range_loop(
        &self,
        function: FunctionValue<'ctx>,
        start: IntValue<'ctx>,
        end: IntValue<'ctx>,
        step: IntValue<'ctx>,
        var_ptr: BasicValueEnum<'ctx>,
        body: BasicBlock<'ctx>,
        exit: BasicBlock<'ctx>
    ) -> BasicBlock<'ctx> {
        if let Some(unrolled) = self.try_unroll_loop(function, start, end, step, var_ptr, body, exit) {
            return unrolled;
        }
        if self.should_chunk_loop(start, end) {
            return self.create_chunked_loop(function, start, end, step, var_ptr, body, exit);
        }
        let inc_bb = self.context.append_basic_block(function, "opt_inc");
        self.optimize_loop_condition(function, start, end, step, var_ptr, body, exit, inc_bb);
        self.builder.get_insert_block().unwrap()
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
        if let (Some(start_const), Some(end_const)) = (
            start_val.get_sign_extended_constant(),
            end_val.get_sign_extended_constant(),
        ) {
            let range_size = if end_const > start_const {
                (end_const - start_const) as u64
            } else {
                0
            };

            if range_size > VERY_LARGE_RANGE_THRESHOLD {
                println!(
                    "[LOOP WARNING] Very large range detected: {} iterations",
                    range_size
                );
            }
        }
        let i64_type = self.context.i64_type();
        let entry_block = self.builder.get_insert_block().unwrap();

        let chunk_init_block = self.context.append_basic_block(function, "chunk_init");
        let chunk_cond_block = self.context.append_basic_block(function, "chunk_cond");
        let chunk_body_block = self.context.append_basic_block(function, "chunk_body");
        let chunk_inc_block = self.context.append_basic_block(function, "chunk_inc");
        let inner_loop_block = self.context.append_basic_block(function, "inner_loop");

        self.builder
            .build_unconditional_branch(chunk_init_block)
            .unwrap();

        self.builder.position_at_end(chunk_init_block);

        let chunk_counter_ptr = self
            .builder
            .build_alloca(i64_type, "chunk_counter")
            .unwrap();
        self.builder
            .build_store(chunk_counter_ptr, start_val)
            .unwrap();

        self.builder
            .build_unconditional_branch(chunk_cond_block)
            .unwrap();

        self.builder.position_at_end(chunk_cond_block);

        let current_chunk = self
            .builder
            .build_load(i64_type, chunk_counter_ptr, "current_chunk")
            .unwrap()
            .into_int_value();

        let chunk_cond = self
            .builder
            .build_int_compare(IntPredicate::SLT, current_chunk, end_val, "chunk_cond")
            .unwrap();

        self.builder
            .build_conditional_branch(chunk_cond, chunk_body_block, exit_block)
            .unwrap();

        self.builder.position_at_end(chunk_body_block);

        let dynamic_chunk_size = self.calculate_chunk_size(start_val, end_val);
        let chunk_size = i64_type.const_int(dynamic_chunk_size, false);
        let chunk_end = self
            .builder
            .build_int_add(current_chunk, chunk_size, "chunk_end")
            .unwrap();

        if cfg!(debug_assertions) {
            let range_size_str = if let (Some(start_const), Some(end_const)) = (
                start_val.get_sign_extended_constant(),
                end_val.get_sign_extended_constant(),
            ) {
                format!("{}", (end_const - start_const) as u64)
            } else {
                "unknown".to_string()
            };

            println!(
                "[LOOP CHUNKING] Using dynamic chunk size: {} for loop with range size: {}",
                dynamic_chunk_size, range_size_str
            );

            let current_memory = memory_profiler::get_current_memory_usage();
            let peak_memory = memory_profiler::get_peak_memory_usage();
            if current_memory > 0 && peak_memory > 0 {
                println!(
                    "[LOOP MEMORY] Current memory: {:.2} MB, Peak: {:.2} MB, Usage ratio: {:.2}%",
                    current_memory as f64 / (1024.0 * 1024.0),
                    peak_memory as f64 / (1024.0 * 1024.0),
                    (current_memory as f64 / peak_memory as f64) * 100.0
                );
            }
        }

        let use_chunk_end = self
            .builder
            .build_int_compare(IntPredicate::SLT, chunk_end, end_val, "use_chunk_end")
            .unwrap();

        let actual_chunk_end = self
            .builder
            .build_select(use_chunk_end, chunk_end, end_val, "actual_chunk_end")
            .unwrap()
            .into_int_value();

        self.builder
            .build_store(loop_var_ptr.into_pointer_value(), current_chunk)
            .unwrap();

        self.builder
            .build_unconditional_branch(inner_loop_block)
            .unwrap();

        self.builder.position_at_end(inner_loop_block);

        let current_val = self
            .builder
            .build_load(i64_type, loop_var_ptr.into_pointer_value(), "current")
            .unwrap()
            .into_int_value();

        let inner_cond = self
            .builder
            .build_int_compare(
                IntPredicate::SLT,
                current_val,
                actual_chunk_end,
                "inner_cond",
            )
            .unwrap();

        let inner_body_block = self.context.append_basic_block(function, "inner_body");
        let inner_inc_block = self.context.append_basic_block(function, "inner_inc");

        self.builder
            .build_conditional_branch(inner_cond, inner_body_block, chunk_inc_block)
            .unwrap();

        self.builder.position_at_end(inner_body_block);

        self.builder.build_unconditional_branch(body_block).unwrap();

        self.builder.position_at_end(inner_inc_block);

        let next_val = self
            .builder
            .build_int_add(current_val, step_val, "next")
            .unwrap();
        self.builder
            .build_store(loop_var_ptr.into_pointer_value(), next_val)
            .unwrap();

        self.builder
            .build_unconditional_branch(inner_loop_block)
            .unwrap();

        self.builder.position_at_end(chunk_inc_block);

        self.builder
            .build_store(chunk_counter_ptr, actual_chunk_end)
            .unwrap();

        self.builder
            .build_unconditional_branch(chunk_cond_block)
            .unwrap();

        entry_block
    }

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
        if let (Some(start_const), Some(end_const), Some(step_const)) = (
            start_val.get_sign_extended_constant(),
            end_val.get_sign_extended_constant(),
            step_val.get_sign_extended_constant(),
        ) {
            if step_const == 0 {
                eprintln!("[LOOP UNROLL] Skipping loop with zero step");
                return None;
            }

            let num_iterations = if step_const > 0 && end_const > start_const {
                (end_const - start_const + step_const - 1) / step_const
            } else if step_const < 0 && start_const > end_const {
                (start_const - end_const - step_const - 1) / (-step_const)
            } else {
                eprintln!("[LOOP UNROLL] Skipping loop with invalid bounds or step direction: start={}, end={}, step={}",
                         start_const, end_const, step_const);
                return None;
            };

            let num_iterations_u64 = num_iterations as u64;

            if num_iterations_u64 <= UNROLL_THRESHOLD && num_iterations_u64 > 0 {
                eprintln!(
                    "[LOOP UNROLL] Fully unrolling loop with {} iterations",
                    num_iterations_u64
                );
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

            if num_iterations_u64 > UNROLL_THRESHOLD
                && num_iterations_u64 <= 500
                && num_iterations_u64 % PARTIAL_UNROLL_FACTOR == 0
            {
                eprintln!(
                    "[LOOP UNROLL] Partially unrolling loop with {} iterations (factor: {})",
                    num_iterations_u64, PARTIAL_UNROLL_FACTOR
                );
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

            eprintln!(
                "[LOOP UNROLL] Not unrolling loop with {} iterations",
                num_iterations_u64
            );
        }

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

        let mut current_val = start_val;
        let mut current_block = entry_block;

        for i in 0..num_iterations {
            let iter_block = self
                .context
                .append_basic_block(function, &format!("unrolled_iter_{}", i));

            self.builder.position_at_end(current_block);
            self.builder.build_unconditional_branch(iter_block).unwrap();

            self.builder.position_at_end(iter_block);

            self.builder
                .build_store(loop_var_ptr.into_pointer_value(), current_val)
                .unwrap();

            self.builder.build_unconditional_branch(body_block).unwrap();

            let cont_block = self
                .context
                .append_basic_block(function, &format!("unrolled_cont_{}", i));

            let body_terminator = body_block.get_terminator();

            if body_terminator.is_some() {
                let new_body_block = self
                    .context
                    .append_basic_block(function, &format!("unrolled_body_{}", i));

                self.builder.position_at_end(new_body_block);

                self.builder.build_unconditional_branch(cont_block).unwrap();
            } else {
                self.builder.position_at_end(body_block);

                self.builder.build_unconditional_branch(cont_block).unwrap();
            }

            self.builder.position_at_end(cont_block);

            current_val = self
                .builder
                .build_int_add(current_val, step_val, &format!("unrolled_next_{}", i))
                .unwrap();

            current_block = cont_block;
        }

        self.builder.position_at_end(current_block);
        self.builder.build_unconditional_branch(exit_block).unwrap();

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

        let header_block = self.context.append_basic_block(function, "unroll_header");
        let body_start_block = self
            .context
            .append_basic_block(function, "unroll_body_start");
        let inc_block = self.context.append_basic_block(function, "unroll_inc");

        self.builder.position_at_end(entry_block);
        self.builder
            .build_unconditional_branch(header_block)
            .unwrap();

        self.builder.position_at_end(header_block);
        self.builder
            .build_store(loop_var_ptr.into_pointer_value(), start_val)
            .unwrap();

        let current_val = self
            .builder
            .build_load(i64_type, loop_var_ptr.into_pointer_value(), "current")
            .unwrap()
            .into_int_value();

        let step_positive = self
            .builder
            .build_int_compare(
                IntPredicate::SGT,
                step_val,
                i64_type.const_int(0, true),
                "step_positive",
            )
            .unwrap();

        let cond_pos = self
            .builder
            .build_int_compare(IntPredicate::SLT, current_val, end_val, "cond_pos")
            .unwrap();

        let cond_neg = self
            .builder
            .build_int_compare(IntPredicate::SGT, current_val, end_val, "cond_neg")
            .unwrap();

        let condition = self
            .builder
            .build_select(step_positive, cond_pos, cond_neg, "loop_condition")
            .unwrap()
            .into_int_value();

        self.builder
            .build_conditional_branch(condition, body_start_block, exit_block)
            .unwrap();

        self.builder.position_at_end(body_start_block);

        let unrolled_step = self
            .builder
            .build_int_mul(
                step_val,
                i64_type.const_int(unroll_factor, false),
                "unrolled_step",
            )
            .unwrap();

        for i in 0..unroll_factor {
            let current_index = if i == 0 {
                current_val
            } else {
                let offset = self
                    .builder
                    .build_int_mul(
                        step_val,
                        i64_type.const_int(i, false),
                        &format!("offset_{}", i),
                    )
                    .unwrap();

                self.builder
                    .build_int_add(current_val, offset, &format!("unrolled_val_{}", i))
                    .unwrap()
            };

            self.builder
                .build_store(loop_var_ptr.into_pointer_value(), current_index)
                .unwrap();

            let body_cont_block = self
                .context
                .append_basic_block(function, &format!("body_cont_{}", i));

            self.builder.build_unconditional_branch(body_block).unwrap();

            self.builder.position_at_end(body_cont_block);

            if i == unroll_factor - 1 {
                self.builder.build_unconditional_branch(inc_block).unwrap();
            }
        }

        self.builder.position_at_end(inc_block);

        let current_val = self
            .builder
            .build_load(
                i64_type,
                loop_var_ptr.into_pointer_value(),
                "current_at_end",
            )
            .unwrap()
            .into_int_value();

        let next_val = self
            .builder
            .build_int_add(current_val, unrolled_step, "next_unrolled")
            .unwrap();
        self.builder
            .build_store(loop_var_ptr.into_pointer_value(), next_val)
            .unwrap();

        self.builder
            .build_unconditional_branch(header_block)
            .unwrap();

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

        let cond_block = self.context.append_basic_block(function, "optimized_cond");
        self.builder.build_unconditional_branch(cond_block).unwrap();
        self.builder.position_at_end(cond_block);

        self.builder
            .build_store(loop_var_ptr.into_pointer_value(), start_val)
            .unwrap();

        let current_val = self
            .builder
            .build_load(i64_type, loop_var_ptr.into_pointer_value(), "current")
            .unwrap()
            .into_int_value();

        let step_positive = self
            .builder
            .build_int_compare(
                IntPredicate::SGT,
                step_val,
                i64_type.const_int(0, true),
                "step_positive",
            )
            .unwrap();

        let cond_pos = self
            .builder
            .build_int_compare(IntPredicate::SLT, current_val, end_val, "cond_pos")
            .unwrap();

        let cond_neg = self
            .builder
            .build_int_compare(IntPredicate::SGT, current_val, end_val, "cond_neg")
            .unwrap();

        let condition = self
            .builder
            .build_select(step_positive, cond_pos, cond_neg, "loop_condition")
            .unwrap()
            .into_int_value();

        self.builder
            .build_conditional_branch(condition, body_block, exit_block)
            .unwrap();

        self.builder.position_at_end(inc_block);

        let current_val = self
            .builder
            .build_load(i64_type, loop_var_ptr.into_pointer_value(), "current_inc")
            .unwrap()
            .into_int_value();
        let next_val = self
            .builder
            .build_int_add(current_val, step_val, "next")
            .unwrap();
        self.builder
            .build_store(loop_var_ptr.into_pointer_value(), next_val)
            .unwrap();

        self.builder.build_unconditional_branch(cond_block).unwrap();
    }
}

// === Parallel Loop Optimization ===

const MIN_PARALLEL_SIZE: u64 = 1000;

/// Parallel loop optimizer using Rayon-like dispatch
pub struct ParallelLoopOptimizer<'ctx> {
    builder: &'ctx Builder<'ctx>,
    context: &'ctx Context,
    module: &'ctx Module<'ctx>,
}

impl<'ctx> ParallelLoopOptimizer<'ctx> {
    pub fn new(builder: &'ctx Builder<'ctx>, context: &'ctx Context, module: &'ctx Module<'ctx>) -> Self {
        Self { builder, context, module }
    }

    pub fn should_parallelize(&self, start: IntValue<'ctx>, end: IntValue<'ctx>) -> bool {
        if let (Some(s), Some(e)) = (start.get_sign_extended_constant(), end.get_sign_extended_constant()) {
            e > s && (e - s) as u64 >= MIN_PARALLEL_SIZE
        } else {
            false
        }
    }

    pub fn create_parallel_loop(
        &self,
        function: FunctionValue<'ctx>,
        start: IntValue<'ctx>,
        end: IntValue<'ctx>,
        step: IntValue<'ctx>,
        _var_ptr: BasicValueEnum<'ctx>,
        body_bb: BasicBlock<'ctx>,
        exit_bb: BasicBlock<'ctx>,
    ) -> BasicBlock<'ctx> {
        let i64_t = self.context.i64_type();
        let entry = self.builder.get_insert_block().unwrap();
        let par_bb = self.context.append_basic_block(function, "parallel_loop");
        self.builder.build_unconditional_branch(par_bb).unwrap();
        self.builder.position_at_end(par_bb);

        let fn_ty = self.context.void_type().fn_type(&[i64_t.into()], false);
        let loop_fn = self.module.add_function(
            &format!("parallel_body_{}", function.get_name().to_str().unwrap_or("fn")),
            fn_ty,
            None,
        );
        let entry_fn_bb = self.context.append_basic_block(loop_fn, "entry");
        let ret_bb = self.builder.get_insert_block().unwrap();
        self.builder.position_at_end(entry_fn_bb);

        let idx = loop_fn.get_first_param().unwrap().into_int_value();
        let glob = self.module.add_global(i64_t, None, &format!("gvar_{}", function.get_name().to_str().unwrap_or("fn")));
        glob.set_initializer(&i64_t.const_zero());
        self.builder.build_store(glob.as_pointer_value(), idx).unwrap();
        self.builder.build_unconditional_branch(body_bb).unwrap();

        self.builder.position_at_end(ret_bb);
        let pr_fn = self.module.get_function("parallel_range_for_each").unwrap_or_else(|| {
            let ty = self.context.void_type().fn_type(
                &[i64_t.into(), i64_t.into(), i64_t.into(), self.context.ptr_type(inkwell::AddressSpace::default()).into()],
                false,
            );
            self.module.add_function("parallel_range_for_each", ty, None)
        });
        self.builder.build_call(
            pr_fn,
            &[start.into(), end.into(), step.into(), loop_fn.as_global_value().as_pointer_value().into()],
            "call_par",
        ).unwrap();
        self.builder.build_unconditional_branch(exit_bb).unwrap();

        entry
    }
}