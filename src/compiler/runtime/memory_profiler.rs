// memory_profiler.rs - Memory usage tracking and profiling
// This file implements memory usage tracking for the Cheetah runtime

use std::sync::atomic::{AtomicUsize, Ordering};

// Constants for memory profiling
const ALLOCATION_TRACKING_THRESHOLD: usize = 4096; // Track allocations larger than 4KB (increased from 1KB)

// Global counters for memory usage
static TOTAL_ALLOCATIONS: AtomicUsize = AtomicUsize::new(0);
static TOTAL_DEALLOCATIONS: AtomicUsize = AtomicUsize::new(0);
static CURRENT_MEMORY_USAGE: AtomicUsize = AtomicUsize::new(0);
static PEAK_MEMORY_USAGE: AtomicUsize = AtomicUsize::new(0);
static LARGE_ALLOCATIONS: AtomicUsize = AtomicUsize::new(0);



/// Initialize the memory profiler
pub fn init() {
    // Reset counters
    TOTAL_ALLOCATIONS.store(0, Ordering::Relaxed);
    TOTAL_DEALLOCATIONS.store(0, Ordering::Relaxed);
    CURRENT_MEMORY_USAGE.store(0, Ordering::Relaxed);
    PEAK_MEMORY_USAGE.store(0, Ordering::Relaxed);
    LARGE_ALLOCATIONS.store(0, Ordering::Relaxed);
}

/// Track a memory allocation
pub fn track_alloc(size: usize, _location: &str) {
    // Only track allocations above the threshold to reduce overhead
    if size >= ALLOCATION_TRACKING_THRESHOLD {
        // Update counters
        TOTAL_ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        let current = CURRENT_MEMORY_USAGE.fetch_add(size, Ordering::Relaxed);
        let new_usage = current + size;

        // Update peak memory usage
        let mut peak = PEAK_MEMORY_USAGE.load(Ordering::Relaxed);
        while peak < new_usage && !PEAK_MEMORY_USAGE.compare_exchange_weak(
            peak, new_usage, Ordering::Relaxed, Ordering::Relaxed
        ).is_ok() {
            peak = PEAK_MEMORY_USAGE.load(Ordering::Relaxed);
        }

        // Track large allocations
        LARGE_ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
    }
}

/// Track a memory deallocation
pub fn track_dealloc(size: usize) {
    // Only track deallocations above the threshold to reduce overhead
    if size >= ALLOCATION_TRACKING_THRESHOLD {
        // Update counters
        TOTAL_DEALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        CURRENT_MEMORY_USAGE.fetch_sub(size, Ordering::Relaxed);
    }
}



/// Get the current memory usage in bytes
pub fn get_current_memory_usage() -> usize {
    CURRENT_MEMORY_USAGE.load(Ordering::Relaxed)
}

/// Get the peak memory usage in bytes
pub fn get_peak_memory_usage() -> usize {
    PEAK_MEMORY_USAGE.load(Ordering::Relaxed)
}

/// Get the total number of allocations
pub fn get_total_allocations() -> usize {
    TOTAL_ALLOCATIONS.load(Ordering::Relaxed)
}

/// Get the total number of deallocations
pub fn get_total_deallocations() -> usize {
    TOTAL_DEALLOCATIONS.load(Ordering::Relaxed)
}

/// Get the number of large allocations
pub fn get_large_allocations() -> usize {
    LARGE_ALLOCATIONS.load(Ordering::Relaxed)
}

/// Print memory usage statistics
pub fn print_memory_stats() {
    let peak = get_peak_memory_usage();
    let allocs = get_total_allocations();
    let large = get_large_allocations();

    // Only print if we have meaningful data
    if peak > 0 {
        eprintln!("[MEMORY STATS]");
        eprintln!("  Peak memory usage: {:.2} MB", bytes_to_mb(peak));
        if allocs > 0 {
            eprintln!("  Large allocations (>= {} KB): {}", ALLOCATION_TRACKING_THRESHOLD / 1024, large);
        }
    }
}

/// Clean up the memory profiler
pub fn cleanup() {
    // Print memory stats
    print_memory_stats();
}

/// Convert bytes to megabytes
fn bytes_to_mb(bytes: usize) -> f64 {
    bytes as f64 / (1024.0 * 1024.0)
}

/// Register memory allocation functions in the module
pub fn register_memory_functions<'ctx>(context: &'ctx inkwell::context::Context, module: &mut inkwell::module::Module<'ctx>) {
    use inkwell::AddressSpace;

    // Create track_allocation function
    let track_allocation_type = context.void_type().fn_type(
        &[
            context.i64_type().into(), // size
            context.ptr_type(AddressSpace::default()).into(), // location
        ],
        false,
    );
    module.add_function("track_allocation", track_allocation_type, None);

    // Create track_deallocation function
    let track_deallocation_type = context.void_type().fn_type(
        &[context.i64_type().into()], // size
        false,
    );
    module.add_function("track_deallocation", track_deallocation_type, None);

    // Create get_current_memory_usage function
    let get_memory_usage_type = context.i64_type().fn_type(&[], false);
    module.add_function("get_current_memory_usage", get_memory_usage_type, None);

    // Create get_peak_memory_usage function
    let get_peak_memory_type = context.i64_type().fn_type(&[], false);
    module.add_function("get_peak_memory_usage", get_peak_memory_type, None);
}

/// Track allocation (C interface)
#[unsafe(no_mangle)]
pub extern "C" fn track_allocation(size: i64, location: *const i8) {
    let location_str = if location.is_null() {
        "unknown"
    } else {
        unsafe {
            let c_str = std::ffi::CStr::from_ptr(location);
            c_str.to_str().unwrap_or("invalid")
        }
    };

    track_alloc(size as usize, location_str);
}

/// Track deallocation (C interface)
#[unsafe(no_mangle)]
pub extern "C" fn track_deallocation(size: i64) {
    track_dealloc(size as usize);
}

/// Get current memory usage (C interface)
#[unsafe(no_mangle)]
pub extern "C" fn get_current_memory_usage_c() -> i64 {
    get_current_memory_usage() as i64
}

/// Get peak memory usage (C interface)
#[unsafe(no_mangle)]
pub extern "C" fn get_peak_memory_usage_c() -> i64 {
    get_peak_memory_usage() as i64
}
