// range_iterator.rs - Optimized range iterator implementation
// This file implements a generator-style range iterator that doesn't allocate the entire sequence

use std::cell::RefCell;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread_local;

// Constants for range optimization
const RANGE_SIZE_LIMIT: i64 = 100_000_000; // Absolute maximum range size to prevent segfaults
const ITERATOR_POOL_SIZE: usize = 8; // Number of iterators to keep in the pool (reduced from 32)
// Chunk sizes for different range sizes
const CHUNK_SIZE_MEDIUM: i64 = 10_000; // Chunk size for medium ranges
const CHUNK_SIZE_LARGE: i64 = 100_000; // Chunk size for large ranges
const RANGE_SIZE_MEDIUM: i64 = 1_000_000; // Threshold for medium ranges
const RANGE_SIZE_LARGE: i64 = 10_000_000; // Threshold for large ranges

// Global counters for range iterator statistics
static ACTIVE_ITERATORS: AtomicUsize = AtomicUsize::new(0);
static TOTAL_ITERATORS_CREATED: AtomicUsize = AtomicUsize::new(0);
static TOTAL_ITERATIONS: AtomicUsize = AtomicUsize::new(0);
static POOL_HITS: AtomicUsize = AtomicUsize::new(0);

// Thread-local iterator pool
thread_local! {
    static ITERATOR_POOL: RefCell<Vec<RangeIterator>> = RefCell::new(Vec::with_capacity(ITERATOR_POOL_SIZE));
}

// Chunking state for large ranges
#[derive(Clone)]
struct ChunkState {
    current_chunk_start: i64,
    current_chunk_end: i64,
    chunk_size: i64,
    overall_end: i64,
}

/// Range iterator that generates values on demand
#[repr(C)]
pub struct RangeIterator {
    start: i64,
    stop: i64,
    step: i64,
    current: i64,
    size: i64,
    is_active: bool,
    // Chunking for large ranges
    chunking: Option<ChunkState>,
}

impl RangeIterator {
    /// Create a new range iterator
    pub fn new(start: i64, stop: i64, step: i64) -> Self {
        // Calculate the range size
        let size = calculate_range_size(start, stop, step);

        // Track statistics
        ACTIVE_ITERATORS.fetch_add(1, Ordering::Relaxed);
        TOTAL_ITERATORS_CREATED.fetch_add(1, Ordering::Relaxed);

        // Determine if we need chunking for large ranges
        let chunking = if size > RANGE_SIZE_MEDIUM {
            let chunk_size = if size > RANGE_SIZE_LARGE {
                CHUNK_SIZE_LARGE
            } else {
                CHUNK_SIZE_MEDIUM
            };

            Some(ChunkState {
                current_chunk_start: start,
                current_chunk_end: std::cmp::min(start + chunk_size * step, stop),
                chunk_size,
                overall_end: stop,
            })
        } else {
            None
        };

        RangeIterator {
            start,
            stop,
            step,
            current: start,
            size,
            is_active: true,
            chunking,
        }
    }

    /// Get an iterator from the pool or create a new one
    pub fn get_from_pool(start: i64, stop: i64, step: i64) -> Self {
        // Try to get an iterator from the pool
        let iter_opt = ITERATOR_POOL.with(|pool| {
            let mut pool = pool.borrow_mut();
            pool.pop()
        });

        if let Some(mut iter) = iter_opt {
            // Reset the iterator with new values
            iter.reset(start, stop, step);
            POOL_HITS.fetch_add(1, Ordering::Relaxed);
            iter
        } else {
            // If no iterator is available in the pool, create a new one
            Self::new(start, stop, step)
        }
    }

    /// Reset the iterator with new values
    pub fn reset(&mut self, start: i64, stop: i64, step: i64) {
        self.start = start;
        self.stop = stop;
        self.step = step;
        self.current = start;
        self.size = calculate_range_size(start, stop, step);
        self.is_active = true;

        // Determine if we need chunking for large ranges
        self.chunking = if self.size > RANGE_SIZE_MEDIUM {
            let chunk_size = if self.size > RANGE_SIZE_LARGE {
                CHUNK_SIZE_LARGE
            } else {
                CHUNK_SIZE_MEDIUM
            };

            Some(ChunkState {
                current_chunk_start: start,
                current_chunk_end: std::cmp::min(start + chunk_size * step, stop),
                chunk_size,
                overall_end: stop,
            })
        } else {
            None
        };

        // Track active iterators
        ACTIVE_ITERATORS.fetch_add(1, Ordering::Relaxed);
    }

    /// Get the next value from the iterator
    pub fn next(&mut self) -> Option<i64> {
        if !self.is_active
            || (self.step > 0 && self.current >= self.stop)
            || (self.step < 0 && self.current <= self.stop)
        {
            return None;
        }

        // Track iterations
        TOTAL_ITERATIONS.fetch_add(1, Ordering::Relaxed);

        // Handle chunked ranges
        if let Some(ref mut chunk) = self.chunking {
            // Check if we've reached the end of the current chunk
            let reached_chunk_end = if self.step > 0 {
                self.current >= chunk.current_chunk_end
            } else {
                self.current <= chunk.current_chunk_end
            };

            if reached_chunk_end {
                // Move to the next chunk
                chunk.current_chunk_start = chunk.current_chunk_end;
                chunk.current_chunk_end = if self.step > 0 {
                    std::cmp::min(
                        chunk.current_chunk_start + chunk.chunk_size * self.step,
                        chunk.overall_end,
                    )
                } else {
                    std::cmp::max(
                        chunk.current_chunk_start + chunk.chunk_size * self.step,
                        chunk.overall_end,
                    )
                };

                // Check if we've reached the end of all chunks
                let reached_end = if self.step > 0 {
                    chunk.current_chunk_start >= chunk.overall_end
                } else {
                    chunk.current_chunk_start <= chunk.overall_end
                };

                if reached_end {
                    return None;
                }

                // Start at the beginning of the new chunk
                self.current = chunk.current_chunk_start;
            }
        } else {
            // Regular non-chunked range
            // Check if we've reached the end
            let reached_end = if self.step > 0 {
                self.current >= self.stop
            } else {
                self.current <= self.stop
            };

            if reached_end {
                return None;
            }
        }

        // Get the current value
        let value = self.current;

        // Increment the current value
        self.current += self.step;

        Some(value)
    }

    /// Return the iterator to the pool
    pub fn return_to_pool(mut self) {
        // Reset the iterator
        self.is_active = false;

        // Track active iterators
        ACTIVE_ITERATORS.fetch_sub(1, Ordering::Relaxed);

        // Add the iterator to the pool if there's space
        ITERATOR_POOL.with(|pool| {
            let mut pool = pool.borrow_mut();
            if pool.len() < ITERATOR_POOL_SIZE {
                pool.push(self);
            }
        });
    }

    /// Get the size of the range
    pub fn size(&self) -> i64 {
        self.size
    }
}

impl Drop for RangeIterator {
    fn drop(&mut self) {
        if self.is_active {
            // Track active iterators
            ACTIVE_ITERATORS.fetch_sub(1, Ordering::Relaxed);
            self.is_active = false;
        }
    }
}

/// Calculate range size with safety limits
pub fn calculate_range_size(start: i64, stop: i64, step: i64) -> i64 {
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
        eprintln!(
            "[RANGE WARNING] Range size {} exceeds limit {}. Limiting to prevent segfault.",
            size, RANGE_SIZE_LIMIT
        );
        size = RANGE_SIZE_LIMIT;
    }

    size
}

/// Initialize the range iterator system
pub fn init() {
    // Reset counters
    ACTIVE_ITERATORS.store(0, Ordering::Relaxed);
    TOTAL_ITERATORS_CREATED.store(0, Ordering::Relaxed);
    TOTAL_ITERATIONS.store(0, Ordering::Relaxed);
    POOL_HITS.store(0, Ordering::Relaxed);

    // Pre-populate the iterator pool
    ITERATOR_POOL.with(|pool| {
        let mut pool = pool.borrow_mut();
        pool.clear();

        // Create just one iterator for the pool to reduce initial memory usage
        pool.push(RangeIterator {
            start: 0,
            stop: 0,
            step: 1,
            current: 0,
            size: 0,
            is_active: false,
            chunking: None,
        });
    });
}

/// Clean up the range iterator system
pub fn cleanup() {
    // Clear the iterator pool
    ITERATOR_POOL.with(|pool| {
        let mut pool = pool.borrow_mut();
        pool.clear();
    });

    // Log statistics
    let active = ACTIVE_ITERATORS.load(Ordering::Relaxed);
    let total = TOTAL_ITERATORS_CREATED.load(Ordering::Relaxed);
    let iterations = TOTAL_ITERATIONS.load(Ordering::Relaxed);
    let pool_hits = POOL_HITS.load(Ordering::Relaxed);

    if active > 0 {
        eprintln!(
            "[RANGE WARNING] {} active iterators not properly returned to pool",
            active
        );
    }

    if total > 0 {
        eprintln!(
            "[RANGE INFO] Created {} iterators, {} pool hits ({:.1}%), {} iterations",
            total,
            pool_hits,
            (pool_hits as f64 / total as f64) * 100.0,
            iterations
        );
    }
}

/// Create a new range iterator with one argument (stop)
#[unsafe(no_mangle)]
pub extern "C" fn range_iterator_1(stop: i64) -> *mut RangeIterator {
    // Safety check: ensure stop is reasonable
    eprintln!("[DEBUG] range_iterator_1 called with stop={}", stop);
    let safe_stop = if stop > RANGE_SIZE_LIMIT {
        eprintln!(
            "[RANGE WARNING] Range stop value {} exceeds limit {}. Limiting to prevent segfault.",
            stop, RANGE_SIZE_LIMIT
        );
        RANGE_SIZE_LIMIT
    } else {
        stop
    };

    // Get an iterator from the pool
    let iter = RangeIterator::get_from_pool(0, safe_stop, 1);

    // Allocate memory for the iterator
    let iter_box = Box::new(iter);

    // Convert to raw pointer
    let ptr = Box::into_raw(iter_box);
    eprintln!("[DEBUG] range_iterator_1 returning pointer: {:p}", ptr);
    ptr
}

/// Create a new range iterator with two arguments (start, stop)
#[unsafe(no_mangle)]
pub extern "C" fn range_iterator_2(start: i64, stop: i64) -> *mut RangeIterator {
    // Safety check: ensure range is reasonable
    let range_size = if start < stop { stop - start } else { 0 };
    let (safe_start, safe_stop) = if range_size > RANGE_SIZE_LIMIT {
        eprintln!(
            "[RANGE WARNING] Range size {} exceeds limit {}. Limiting to prevent segfault.",
            range_size, RANGE_SIZE_LIMIT
        );
        (start, start + RANGE_SIZE_LIMIT)
    } else {
        (start, stop)
    };

    // Get an iterator from the pool
    let iter = RangeIterator::get_from_pool(safe_start, safe_stop, 1);

    // Allocate memory for the iterator
    let iter_box = Box::new(iter);

    // Convert to raw pointer
    Box::into_raw(iter_box)
}

/// Create a new range iterator with three arguments (start, stop, step)
#[unsafe(no_mangle)]
pub extern "C" fn range_iterator_3(start: i64, stop: i64, step: i64) -> *mut RangeIterator {
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
        eprintln!(
            "[RANGE WARNING] Range size {} exceeds limit {}. Limiting to prevent segfault.",
            range_size, RANGE_SIZE_LIMIT
        );
        if safe_step > 0 {
            (start, start + (RANGE_SIZE_LIMIT * safe_step))
        } else {
            (start, start + (RANGE_SIZE_LIMIT * safe_step))
        }
    } else {
        (start, stop)
    };

    // Get an iterator from the pool
    let iter = RangeIterator::get_from_pool(safe_start, safe_stop, safe_step);

    // Allocate memory for the iterator
    let iter_box = Box::new(iter);

    // Convert to raw pointer
    Box::into_raw(iter_box)
}

/// Get the next value from a range iterator
#[unsafe(no_mangle)]
pub extern "C" fn range_iterator_next(iter_ptr: *mut RangeIterator, value_ptr: *mut i64) -> bool {
    eprintln!("[DEBUG] range_iterator_next called with iter_ptr={:p}, value_ptr={:p}", iter_ptr, value_ptr);
    if iter_ptr.is_null() || value_ptr.is_null() {
        eprintln!("[DEBUG] range_iterator_next: null pointer detected, returning false");
        return false;
    }

    unsafe {
        let iter = &mut *iter_ptr;
        eprintln!("[DEBUG] range_iterator_next: current={}, stop={}, step={}", iter.current, iter.stop, iter.step);
        if let Some(value) = iter.next() {
            eprintln!("[DEBUG] range_iterator_next: got value {}, storing at {:p}", value, value_ptr);
            *value_ptr = value;
            true
        } else {
            eprintln!("[DEBUG] range_iterator_next: no more values, returning false");
            false
        }
    }
}

/// Get the size of a range iterator
#[unsafe(no_mangle)]
pub extern "C" fn range_iterator_size(iter_ptr: *mut RangeIterator) -> i64 {
    if iter_ptr.is_null() {
        return 0;
    }

    unsafe {
        let iter = &*iter_ptr;
        iter.size()
    }
}

/// Free a range iterator
#[unsafe(no_mangle)]
pub extern "C" fn range_iterator_free(iter_ptr: *mut RangeIterator) {
    if iter_ptr.is_null() {
        return;
    }

    unsafe {
        // Convert back to Box and drop
        let iter = Box::from_raw(iter_ptr);

        // Return to pool instead of dropping
        iter.return_to_pool();
    }
}

/// Register range iterator functions in the module
pub fn register_range_iterator_functions<'ctx>(
    context: &'ctx inkwell::context::Context,
    module: &mut inkwell::module::Module<'ctx>,
) {
    use inkwell::AddressSpace;

    // Create range_iterator_1 function
    let range_iterator_1_type = context.ptr_type(AddressSpace::default()).fn_type(
        &[context.i64_type().into()], // stop
        false,
    );
    module.add_function("range_iterator_1", range_iterator_1_type, None);

    // Create range_iterator_2 function
    let range_iterator_2_type = context.ptr_type(AddressSpace::default()).fn_type(
        &[
            context.i64_type().into(), // start
            context.i64_type().into(), // stop
        ],
        false,
    );
    module.add_function("range_iterator_2", range_iterator_2_type, None);

    // Create range_iterator_3 function
    let range_iterator_3_type = context.ptr_type(AddressSpace::default()).fn_type(
        &[
            context.i64_type().into(), // start
            context.i64_type().into(), // stop
            context.i64_type().into(), // step
        ],
        false,
    );
    module.add_function("range_iterator_3", range_iterator_3_type, None);

    // Create range_iterator_next function
    let range_iterator_next_type = context.bool_type().fn_type(
        &[
            context.ptr_type(AddressSpace::default()).into(), // iterator
            context.ptr_type(AddressSpace::default()).into(), // value
        ],
        false,
    );
    module.add_function("range_iterator_next", range_iterator_next_type, None);

    // Create range_iterator_size function
    let range_iterator_size_type = context.i64_type().fn_type(
        &[context.ptr_type(AddressSpace::default()).into()], // iterator
        false,
    );
    module.add_function("range_iterator_size", range_iterator_size_type, None);

    // Create range_iterator_free function
    let range_iterator_free_type = context.void_type().fn_type(
        &[context.ptr_type(AddressSpace::default()).into()], // iterator
        false,
    );
    module.add_function("range_iterator_free", range_iterator_free_type, None);
}
