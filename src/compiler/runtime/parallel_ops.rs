// parallel_ops.rs - Parallel processing operations using Rayon
// This file implements parallel processing capabilities for Cheetah

use rayon::prelude::*;
use std::sync::atomic::{AtomicUsize, Ordering};

// Constants for parallel processing
const MIN_PARALLEL_SIZE: usize = 1000; // Minimum size for parallel processing
// Removed unused constant PARALLEL_CHUNK_SIZE

// Global counters for parallel processing statistics
static PARALLEL_OPERATIONS: AtomicUsize = AtomicUsize::new(0);
static SEQUENTIAL_OPERATIONS: AtomicUsize = AtomicUsize::new(0);

/// Initialize parallel processing
pub fn init() {
    // Reset counters
    PARALLEL_OPERATIONS.store(0, Ordering::Relaxed);
    SEQUENTIAL_OPERATIONS.store(0, Ordering::Relaxed);
}

/// Clean up parallel processing
pub fn cleanup() {
    // Print statistics
    let parallel_ops = PARALLEL_OPERATIONS.load(Ordering::Relaxed);
    let sequential_ops = SEQUENTIAL_OPERATIONS.load(Ordering::Relaxed);

    if parallel_ops > 0 || sequential_ops > 0 {
        eprintln!("[PARALLEL STATS] Parallel operations: {}, Sequential operations: {}",
                 parallel_ops, sequential_ops);
    }
}

/// Determine if a range should be processed in parallel
pub fn should_parallelize(size: usize) -> bool {
    size >= MIN_PARALLEL_SIZE
}

/// Process a range in parallel using Rayon
///
/// This function takes a range and a function to apply to each element,
/// and processes the range in parallel if it's large enough.
///
/// # Arguments
/// * `start` - The start of the range
/// * `end` - The end of the range
/// * `step` - The step size
/// * `f` - The function to apply to each element
///
/// # Returns
/// A vector containing the results of applying the function to each element
pub fn parallel_range_map<F, R>(start: i64, end: i64, step: i64, f: F) -> Vec<R>
where
    F: Fn(i64) -> R + Send + Sync,
    R: Send,
{
    // Calculate the size of the range
    let size = if step == 0 {
        0
    } else if step > 0 && start < end {
        ((end - start - 1) / step + 1) as usize
    } else if step < 0 && start > end {
        ((start - end - 1) / (-step) + 1) as usize
    } else {
        0
    };

    // Decide whether to process in parallel
    if should_parallelize(size) {
        // Track parallel operations
        PARALLEL_OPERATIONS.fetch_add(1, Ordering::Relaxed);

        // Create a parallel iterator for the range
        (0..size)
            .into_par_iter()
            .map(|i| {
                let value = start + (i as i64) * step;
                f(value)
            })
            .collect()
    } else {
        // Track sequential operations
        SEQUENTIAL_OPERATIONS.fetch_add(1, Ordering::Relaxed);

        // Process sequentially for small ranges
        (0..size)
            .map(|i| {
                let value = start + (i as i64) * step;
                f(value)
            })
            .collect()
    }
}

/// Process a range in parallel with a for-each operation
///
/// This function takes a range and a function to apply to each element,
/// and processes the range in parallel if it's large enough.
///
/// # Arguments
/// * `start` - The start of the range
/// * `end` - The end of the range
/// * `step` - The step size
/// * `f` - The function to apply to each element
pub fn parallel_range_for_each<F>(start: i64, end: i64, step: i64, f: F)
where
    F: Fn(i64) + Send + Sync,
{
    // Calculate the size of the range
    let size = if step == 0 {
        0
    } else if step > 0 && start < end {
        ((end - start - 1) / step + 1) as usize
    } else if step < 0 && start > end {
        ((start - end - 1) / (-step) + 1) as usize
    } else {
        0
    };

    // Decide whether to process in parallel
    if should_parallelize(size) {
        // Track parallel operations
        PARALLEL_OPERATIONS.fetch_add(1, Ordering::Relaxed);

        // Process in parallel
        (0..size)
            .into_par_iter()
            .for_each(|i| {
                let value = start + (i as i64) * step;
                f(value);
            });
    } else {
        // Track sequential operations
        SEQUENTIAL_OPERATIONS.fetch_add(1, Ordering::Relaxed);

        // Process sequentially for small ranges
        for i in 0..size {
            let value = start + (i as i64) * step;
            f(value);
        }
    }
}

/// Process a collection in parallel
///
/// This function takes a collection and a function to apply to each element,
/// and processes the collection in parallel if it's large enough.
///
/// # Arguments
/// * `collection` - The collection to process
/// * `f` - The function to apply to each element
///
/// # Returns
/// A vector containing the results of applying the function to each element
pub fn parallel_collection_map<T, F, R>(collection: &[T], f: F) -> Vec<R>
where
    T: Sync,
    F: Fn(&T) -> R + Send + Sync,
    R: Send,
{
    // Decide whether to process in parallel
    if should_parallelize(collection.len()) {
        // Track parallel operations
        PARALLEL_OPERATIONS.fetch_add(1, Ordering::Relaxed);

        // Process in parallel
        collection
            .par_iter()
            .map(f)
            .collect()
    } else {
        // Track sequential operations
        SEQUENTIAL_OPERATIONS.fetch_add(1, Ordering::Relaxed);

        // Process sequentially for small collections
        collection
            .iter()
            .map(f)
            .collect()
    }
}

/// Process a collection in parallel with a for-each operation
///
/// This function takes a collection and a function to apply to each element,
/// and processes the collection in parallel if it's large enough.
///
/// # Arguments
/// * `collection` - The collection to process
/// * `f` - The function to apply to each element
pub fn parallel_collection_for_each<T, F>(collection: &[T], f: F)
where
    T: Sync,
    F: Fn(&T) + Send + Sync,
{
    // Decide whether to process in parallel
    if should_parallelize(collection.len()) {
        // Track parallel operations
        PARALLEL_OPERATIONS.fetch_add(1, Ordering::Relaxed);

        // Process in parallel
        collection
            .par_iter()
            .for_each(f);
    } else {
        // Track sequential operations
        SEQUENTIAL_OPERATIONS.fetch_add(1, Ordering::Relaxed);

        // Process sequentially for small collections
        collection
            .iter()
            .for_each(f);
    }
}

/// Register parallel processing functions in the module
pub fn register_parallel_functions<'ctx>(context: &'ctx inkwell::context::Context, module: &mut inkwell::module::Module<'ctx>) {
    use inkwell::AddressSpace;

    // Create parallel_range_map function
    let parallel_range_map_type = context.ptr_type(AddressSpace::default()).fn_type(
        &[
            context.i64_type().into(), // start
            context.i64_type().into(), // end
            context.i64_type().into(), // step
            context.ptr_type(AddressSpace::default()).into(), // function pointer
        ],
        false,
    );
    module.add_function("parallel_range_map", parallel_range_map_type, None);

    // Create parallel_range_for_each function
    let parallel_range_for_each_type = context.void_type().fn_type(
        &[
            context.i64_type().into(), // start
            context.i64_type().into(), // end
            context.i64_type().into(), // step
            context.ptr_type(AddressSpace::default()).into(), // function pointer
        ],
        false,
    );
    module.add_function("parallel_range_for_each", parallel_range_for_each_type, None);

    // Create parallel_collection_map function
    let parallel_collection_map_type = context.ptr_type(AddressSpace::default()).fn_type(
        &[
            context.ptr_type(AddressSpace::default()).into(), // collection pointer
            context.i64_type().into(), // collection length
            context.ptr_type(AddressSpace::default()).into(), // function pointer
        ],
        false,
    );
    module.add_function("parallel_collection_map", parallel_collection_map_type, None);

    // Create parallel_collection_for_each function
    let parallel_collection_for_each_type = context.void_type().fn_type(
        &[
            context.ptr_type(AddressSpace::default()).into(), // collection pointer
            context.i64_type().into(), // collection length
            context.ptr_type(AddressSpace::default()).into(), // function pointer
        ],
        false,
    );
    module.add_function("parallel_collection_for_each", parallel_collection_for_each_type, None);
}
