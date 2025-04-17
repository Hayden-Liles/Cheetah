// range_ops.rs - Runtime support for range operations

use inkwell::context::Context;
use inkwell::module::Module;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use rayon::prelude::*;

// Global counter for range operations to detect potential issues
static RANGE_OPERATION_COUNT: AtomicUsize = AtomicUsize::new(0);

// Flag to enable range operation tracking
static RANGE_DEBUG_ENABLED: AtomicBool = AtomicBool::new(false);

// Constants for range optimization
const VERY_LARGE_RANGE_THRESHOLD: i64 = 10_000_000; // 10 million iterations is very large
const RANGE_SIZE_LIMIT: i64 = 100_000_000; // Absolute maximum range size to prevent segfaults
const VECTORIZATION_THRESHOLD: i64 = 1_000; // Minimum size for vectorization
const VECTOR_WIDTH: usize = 4; // Process 4 integers at once for SIMD

// New constants for parallel processing
const MIN_PARALLEL_RANGE: i64 = 10_000; // Minimum size for parallel processing
const MAX_THREADS: usize = 12; // Maximum number of threads to use
const UNROLL_FACTOR: usize = 8; // Loop unrolling factor

/// Register range operation functions in the module
pub fn register_range_functions<'ctx>(context: &'ctx Context, module: &mut Module<'ctx>) {
    // Create range_1 function (range with stop only)
    let range_1_type = context.i64_type().fn_type(&[context.i64_type().into()], false);
    module.add_function("range_1", range_1_type, None);

    // Create range_2 function (range with start and stop)
    let range_2_type = context.i64_type().fn_type(
        &[
            context.i64_type().into(), // start
            context.i64_type().into(), // stop
        ],
        false,
    );
    module.add_function("range_2", range_2_type, None);

    // Create range_3 function (range with start, stop, and step)
    let range_3_type = context.i64_type().fn_type(
        &[
            context.i64_type().into(), // start
            context.i64_type().into(), // stop
            context.i64_type().into(), // step
        ],
        false,
    );
    module.add_function("range_3", range_3_type, None);

    // Create range_cleanup function (for memory management)
    let range_cleanup_type = context.void_type().fn_type(&[], false);
    module.add_function("range_cleanup", range_cleanup_type, None);

    // Add new optimized functions
    let arithmetic_sum_type = context.i64_type().fn_type(
        &[
            context.i64_type().into(), // first
            context.i64_type().into(), // last
            context.i64_type().into(), // count
        ],
        false,
    );
    module.add_function("compute_arithmetic_sum", arithmetic_sum_type, None);

    // Add vectorized range sum function
    let vec_range_sum_type = context.i64_type().fn_type(
        &[
            context.i64_type().into(), // start
            context.i64_type().into(), // end
        ],
        false,
    );
    module.add_function("vectorized_range_sum", vec_range_sum_type, None);

    // Add parallel range functions
    let parallel_range_sum_type = context.i64_type().fn_type(
        &[
            context.i64_type().into(), // start
            context.i64_type().into(), // end
            context.i64_type().into(), // chunk_size
        ],
        false,
    );
    module.add_function("parallel_range_sum", parallel_range_sum_type, None);

    // Add unrolled range function
    let unrolled_range_sum_type = context.i64_type().fn_type(
        &[
            context.i64_type().into(), // start
            context.i64_type().into(), // end
        ],
        false,
    );
    module.add_function("unrolled_range_sum", unrolled_range_sum_type, None);
}

/// Initialize range operations
pub fn init() {
    // Check environment variables to enable debugging features
    if std::env::var("CHEETAH_RANGE_DEBUG").is_ok() {
        eprintln!("Range debug mode enabled");
        RANGE_DEBUG_ENABLED.store(true, Ordering::Relaxed);
    }

    // Reset operation count
    RANGE_OPERATION_COUNT.store(0, Ordering::Relaxed);
}

/// Track a range operation
pub fn track_range_operation(start: i64, stop: i64, step: i64) {
    let count = RANGE_OPERATION_COUNT.fetch_add(1, Ordering::Relaxed);

    // Only log if debug is enabled
    if RANGE_DEBUG_ENABLED.load(Ordering::Relaxed) {
        // Log every 1000 operations to avoid flooding
        if count % 1000 == 0 {
            eprintln!(
                "[RANGE DEBUG] Operation count: {}, Range: {}..{} step {}",
                count, start, stop, step
            );
        }

        // Warn about very large ranges
        if (stop - start).abs() > VERY_LARGE_RANGE_THRESHOLD {
            eprintln!(
                "[RANGE WARNING] Very large range detected: {}..{} step {} (size: {})",
                start, stop, step, (stop - start).abs()
            );
        }
    }
}

/// Calculate range size with optimized paths
pub fn calculate_range_size(start: i64, stop: i64, step: i64) -> i64 {
    eprintln!("[DEBUG] calculate_range_size called with start={}, stop={}, step={}", start, stop, step);

    // Fast path for common cases
    let mut size = if step == 0 {
        eprintln!("[DEBUG] step is zero, returning 0");
        0
    } else if step == 1 && start < stop {
        eprintln!("[DEBUG] optimizing for common case: step=1, start < stop");
        // Use direct calculation for consecutive integers
        stop - start
    } else if (step > 0 && start < stop) || (step < 0 && start > stop) {
        eprintln!("[DEBUG] calculating size for general case");
        let diff = (stop - start).abs();
        let abs_step = step.abs();
        // Optimize division for power-of-two steps
        if (abs_step as u64).is_power_of_two() {
            diff >> abs_step.trailing_zeros()
        } else {
            (diff + abs_step - 1) / abs_step // Ceiling division
        }
    } else {
        eprintln!("[DEBUG] invalid range, returning 0");
        0
    };

    // Apply safety limits
    if size > RANGE_SIZE_LIMIT {
        eprintln!("[RANGE WARNING] Range size {} exceeds limit {}. Limiting to prevent segfault.",
                 size, RANGE_SIZE_LIMIT);
        size = RANGE_SIZE_LIMIT;
    }

    size
}

/// Clean up range operations (free memory, etc.)
pub fn cleanup() {
    // Reset operation count
    RANGE_OPERATION_COUNT.store(0, Ordering::Relaxed);

    // Print metrics before cleanup if debug is enabled
    print_performance_metrics();

    // Reset metrics
    METRICS.total_operations.store(0, Ordering::Relaxed);
    METRICS.vectorized_ops.store(0, Ordering::Relaxed);
    METRICS.parallel_ops.store(0, Ordering::Relaxed);
    METRICS.unrolled_ops.store(0, Ordering::Relaxed);
}

/// Range function with one argument (stop)
#[unsafe(no_mangle)]
pub extern "C" fn range_1(stop: i64) -> i64 {
    // Safety check: ensure stop is reasonable
    eprintln!("[DEBUG] range_1 called with stop={}", stop);
    let safe_stop = if stop > RANGE_SIZE_LIMIT {
        eprintln!("[RANGE WARNING] Range stop value {} exceeds limit {}. Limiting to prevent segfault.",
                 stop, RANGE_SIZE_LIMIT);
        RANGE_SIZE_LIMIT
    } else {
        stop
    };

    track_range_operation(0, safe_stop, 1);

    // The range size is the value we return
    // For range objects, we'll use the integer value itself as a "pointer"
    // This allows us to distinguish range objects from regular lists
    let size = calculate_range_size(0, safe_stop, 1);
    eprintln!("[DEBUG] range_1 returning size: {}", size);
    size
}

/// Range function with two arguments (start, stop)
#[unsafe(no_mangle)]
pub extern "C" fn range_2(start: i64, stop: i64) -> i64 {
    // Safety check: ensure range is reasonable
    let range_size = if start < stop { stop - start } else { 0 };
    let (safe_start, safe_stop) = if range_size > RANGE_SIZE_LIMIT {
        eprintln!("[RANGE WARNING] Range size {} exceeds limit {}. Limiting to prevent segfault.",
                 range_size, RANGE_SIZE_LIMIT);
        (start, start + RANGE_SIZE_LIMIT)
    } else {
        (start, stop)
    };

    track_range_operation(safe_start, safe_stop, 1);

    // Calculate and return the range size as a "pointer"
    let size = calculate_range_size(safe_start, safe_stop, 1);
    println!("range_2 returning size: {}", size);
    size
}

/// Range function with three arguments (start, stop, step)
#[unsafe(no_mangle)]
pub extern "C" fn range_3(start: i64, stop: i64, step: i64) -> i64 {
    // Safety check for step
    let safe_step = if step == 0 { 1 } else { step };

    // Calculate the theoretical range size
    let range_size = calculate_range_size(start, stop, safe_step);

    // Apply safety limits
    let (safe_start, safe_stop) = if range_size > RANGE_SIZE_LIMIT {
        eprintln!("[RANGE WARNING] Range size {} exceeds limit {}. Limiting to prevent segfault.",
                 range_size, RANGE_SIZE_LIMIT);
        if safe_step > 0 {
            (start, start + (RANGE_SIZE_LIMIT * safe_step))
        } else {
            (start, start + (RANGE_SIZE_LIMIT * safe_step))
        }
    } else {
        (start, stop)
    };

    track_range_operation(safe_start, safe_stop, safe_step);

    // Calculate and return the range size as a "pointer"
    let size = calculate_range_size(safe_start, safe_stop, safe_step);
    println!("range_3 returning size: {}", size);
    size
}

/// Clean up range operations
#[unsafe(no_mangle)]
pub extern "C" fn range_cleanup() {
    cleanup();
}

/// Compute sum of arithmetic sequence directly
#[unsafe(no_mangle)]
pub extern "C" fn compute_arithmetic_sum(first: i64, last: i64, count: i64) -> i64 {
    if count <= 0 {
        return 0;
    }
    // Use the arithmetic sequence sum formula: count * (first + last) / 2
    // Check for overflow
    if let Some(sum) = count.checked_mul(first.checked_add(last).unwrap_or(i64::MAX))
        .and_then(|x| x.checked_div(2)) {
        sum
    } else {
        eprintln!("[ARITHMETIC WARNING] Overflow detected in sum calculation");
        i64::MAX
    }
}

/// Vectorized range sum implementation
#[unsafe(no_mangle)]
pub extern "C" fn vectorized_range_sum(start: i64, end: i64) -> i64 {
    let size = end - start;

    if size <= VECTORIZATION_THRESHOLD {
        // Use regular sum for small ranges
        return compute_arithmetic_sum(start, end - 1, size);
    }

    let vector_chunks = size as usize / VECTOR_WIDTH;
    let mut sum = 0i64;

    // Process in SIMD vectors
    for chunk in 0..vector_chunks {
        let base = start + (chunk * VECTOR_WIDTH) as i64;
        let mut chunk_sum = 0;

        // Simulate SIMD operation (actual SIMD would use platform-specific intrinsics)
        for i in 0..VECTOR_WIDTH {
            chunk_sum += base + i as i64;
        }

        sum += chunk_sum;
    }

    // Handle remaining elements
    let remaining_start = start + (vector_chunks * VECTOR_WIDTH) as i64;
    for i in remaining_start..end {
        sum += i;
    }

    sum
}

/// Parallel range sum implementation using Rayon
#[unsafe(no_mangle)]
pub extern "C" fn parallel_range_sum(start: i64, end: i64, chunk_size: i64) -> i64 {
    let size = end - start;

    if size <= MIN_PARALLEL_RANGE {
        return compute_arithmetic_sum(start, end - 1, size);
    }

    // Calculate chunk size based on number of available cores
    let num_chunks = (size / chunk_size).max(1) as usize;
    let actual_chunk_size = (size / num_chunks as i64).max(1);

    // Create chunks and process them in parallel
    let sum: i64 = (0..num_chunks)
        .into_par_iter()
        .map(|chunk_idx| {
            let chunk_start = start + (chunk_idx as i64 * actual_chunk_size);
            let chunk_end = if chunk_idx == num_chunks - 1 {
                end
            } else {
                chunk_start + actual_chunk_size
            };
            compute_chunk_sum(chunk_start, chunk_end)
        })
        .sum();

    sum
}

/// Helper function to compute sum for a chunk
fn compute_chunk_sum(start: i64, end: i64) -> i64 {
    if end - start <= VECTORIZATION_THRESHOLD {
        compute_arithmetic_sum(start, end - 1, end - start)
    } else {
        vectorized_range_sum(start, end)
    }
}

/// Unrolled range sum implementation
#[unsafe(no_mangle)]
pub extern "C" fn unrolled_range_sum(start: i64, end: i64) -> i64 {
    let size = end - start;

    if size <= UNROLL_FACTOR as i64 {
        return compute_arithmetic_sum(start, end - 1, size);
    }

    let mut sum = 0i64;
    let mut i = start;

    // Main unrolled loop
    while i + UNROLL_FACTOR as i64 <= end {
        sum += i;
        sum += i + 1;
        sum += i + 2;
        sum += i + 3;
        sum += i + 4;
        sum += i + 5;
        sum += i + 6;
        sum += i + 7;
        i += UNROLL_FACTOR as i64;
    }

    // Handle remaining elements
    while i < end {
        sum += i;
        i += 1;
    }

    sum
}

/// Optimized range iterator that chooses the best implementation
#[unsafe(no_mangle)]
pub extern "C" fn optimized_range_sum(start: i64, end: i64) -> i64 {
    let size = end - start;

    // Choose the most appropriate implementation based on range size
    match size {
        s if s <= VECTORIZATION_THRESHOLD => {
            // Use direct formula for small ranges
            compute_arithmetic_sum(start, end - 1, size)
        }
        s if s <= MIN_PARALLEL_RANGE => {
            // Use vectorized implementation for medium ranges
            vectorized_range_sum(start, end)
        }
        s if is_power_of_two(s) => {
            // Use unrolled implementation for power-of-two sizes
            unrolled_range_sum(start, end)
        }
        _ => {
            // Use parallel implementation for large ranges
            let chunk_size = (size / MAX_THREADS as i64).max(MIN_PARALLEL_RANGE);
            parallel_range_sum(start, end, chunk_size)
        }
    }
}

/// Performance monitoring structure
#[derive(Default)]
pub struct RangePerformanceMetrics {
    pub total_operations: AtomicUsize,
    pub vectorized_ops: AtomicUsize,
    pub parallel_ops: AtomicUsize,
    pub unrolled_ops: AtomicUsize,
}

// Global performance metrics
lazy_static::lazy_static! {
    static ref METRICS: RangePerformanceMetrics = RangePerformanceMetrics::default();
}

// Update performance metrics
fn update_metrics(operation_type: &str) {
    METRICS.total_operations.fetch_add(1, Ordering::Relaxed);
    match operation_type {
        "vectorized" => { METRICS.vectorized_ops.fetch_add(1, Ordering::Relaxed); }
        "parallel" => { METRICS.parallel_ops.fetch_add(1, Ordering::Relaxed); }
        "unrolled" => { METRICS.unrolled_ops.fetch_add(1, Ordering::Relaxed); }
        _ => {}
    }
}

/// Helper function to check if an i64 is a power of two
fn is_power_of_two(n: i64) -> bool {
    n > 0 && (n & (n - 1)) == 0
}

/// Print performance metrics
pub fn print_performance_metrics() {
    if RANGE_DEBUG_ENABLED.load(Ordering::Relaxed) {
        eprintln!("Range Operation Performance Metrics:");
        eprintln!("Total operations: {}", METRICS.total_operations.load(Ordering::Relaxed));
        eprintln!("Vectorized operations: {}", METRICS.vectorized_ops.load(Ordering::Relaxed));
        eprintln!("Parallel operations: {}", METRICS.parallel_ops.load(Ordering::Relaxed));
        eprintln!("Unrolled operations: {}", METRICS.unrolled_ops.load(Ordering::Relaxed));
    }
}
