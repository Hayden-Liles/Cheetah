// range_ops.rs - Runtime support for range operations

use inkwell::context::Context;
use inkwell::module::Module;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

// Global counter for range operations to detect potential issues
static RANGE_OPERATION_COUNT: AtomicUsize = AtomicUsize::new(0);

// Flag to enable range operation tracking
static RANGE_DEBUG_ENABLED: AtomicBool = AtomicBool::new(false);

// Constants for range optimization
const VERY_LARGE_RANGE_THRESHOLD: i64 = 10_000_000; // 10 million iterations is very large
const RANGE_SIZE_LIMIT: i64 = 100_000_000; // Absolute maximum range size to prevent segfaults

// We'll use a simpler approach without caching to avoid dependencies

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

    // No cache to clear in this simplified version
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

/// Calculate range size with caching for performance
pub fn calculate_range_size(start: i64, stop: i64, step: i64) -> i64 {
    // Simple calculation without caching

    // Calculate the range size
    let mut size = if step == 0 {
        0 // Avoid division by zero
    } else if step == 1 && start < stop {
        stop - start // Optimize for common case
    } else if (step > 0 && start < stop) || (step < 0 && start > stop) {
        (stop - start) / step + ((stop - start) % step != 0) as i64
    } else {
        0 // Invalid range
    };

    // Safety check: limit the maximum range size to prevent segfaults
    if size > RANGE_SIZE_LIMIT {
        eprintln!("[RANGE WARNING] Range size {} exceeds limit {}. Limiting to prevent segfault.",
                 size, RANGE_SIZE_LIMIT);
        size = RANGE_SIZE_LIMIT;
    }

    // No caching in this simplified version

    size
}

/// Clean up range operations (free memory, etc.)
pub fn cleanup() {
    // No cache to clear in this simplified version

    // Reset operation count
    RANGE_OPERATION_COUNT.store(0, Ordering::Relaxed);
}

/// Range function with one argument (stop)
#[unsafe(no_mangle)]
#[allow(improper_ctypes_definitions)]
pub extern "C" fn range_1(stop: i64) -> i64 {
    // Safety check: ensure stop is reasonable
    let safe_stop = if stop > RANGE_SIZE_LIMIT {
        eprintln!("[RANGE WARNING] Range stop value {} exceeds limit {}. Limiting to prevent segfault.",
                 stop, RANGE_SIZE_LIMIT);
        RANGE_SIZE_LIMIT
    } else {
        stop
    };

    track_range_operation(0, safe_stop, 1);
    calculate_range_size(0, safe_stop, 1)
}

/// Range function with two arguments (start, stop)
#[unsafe(no_mangle)]
#[allow(improper_ctypes_definitions)]
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
    calculate_range_size(safe_start, safe_stop, 1)
}

/// Range function with three arguments (start, stop, step)
#[unsafe(no_mangle)]
#[allow(improper_ctypes_definitions)]
pub extern "C" fn range_3(start: i64, stop: i64, step: i64) -> i64 {
    // Safety check for step
    let safe_step = if step == 0 { 1 } else { step };

    // Calculate the theoretical range size
    let range_size = if (safe_step > 0 && start < stop) || (safe_step < 0 && start > stop) {
        ((stop - start) / safe_step).abs() + (((stop - start) % safe_step) != 0) as i64
    } else {
        0
    };

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
    calculate_range_size(safe_start, safe_stop, safe_step)
}

/// Clean up range operations
#[unsafe(no_mangle)]
#[allow(improper_ctypes_definitions)]
pub extern "C" fn range_cleanup() {
    cleanup();
}
