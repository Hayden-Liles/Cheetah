// buffered_output.rs - Buffered output system for improved performance

use std::cell::RefCell;
use std::io::{self, Write};
use std::sync::atomic::{AtomicUsize, AtomicBool, Ordering};

// Import our debug utilities
use super::debug_utils;

// Fast number formatting libraries
use itoa;
use ryu;

// Constants for buffer management
const BUFFER_CAPACITY: usize = 1024; // 1KB buffer - reduced to prevent memory issues
const FLUSH_THRESHOLD: usize = 512; // Flush at 512B - reduced to prevent buffer overflow
const CHECKPOINT_INTERVAL: usize = 10; // Reset stack state every 10 operations - reduced to prevent stack overflow
const DIRECT_OUTPUT_THRESHOLD: usize = 1; // Always use direct output - critical to prevent stack overflow
const MAX_STACK_DEPTH: usize = 5; // Maximum allowed stack depth before warning - reduced to prevent stack overflow
const STACK_OVERFLOW_THRESHOLD: usize = 10; // Threshold for potential stack overflow - reduced to prevent stack overflow

// Global flag to force direct output mode (set to true when stack gets too deep)
static FORCE_DIRECT_OUTPUT: AtomicBool = AtomicBool::new(true); // Default to true to prevent stack overflows

// Global static buffer for number conversion to avoid stack allocations
static mut NUMBER_BUFFER: [u8; 32] = [0; 32];

// Global static buffer for newlines to avoid allocations
static NEWLINE: [u8; 1] = [b'\n'];

// Global counter for operations to trigger checkpoints
static OPERATION_COUNTER: AtomicUsize = AtomicUsize::new(0);

// Global counter for tracking stack depth
static STACK_DEPTH: AtomicUsize = AtomicUsize::new(0);

// Global flag to enable/disable stack debugging
static STACK_DEBUG_ENABLED: AtomicUsize = AtomicUsize::new(1); // 1 = enabled, 0 = disabled

// Initialize the module - call this at the start of execution
pub fn init() {
    // Initialize debug utilities first
    debug_utils::init();

    // Reset all counters and flags
    OPERATION_COUNTER.store(0, Ordering::Relaxed);
    STACK_DEPTH.store(0, Ordering::Relaxed);
    STACK_DEBUG_ENABLED.store(1, Ordering::Relaxed);

    // CRITICAL: Always force direct output mode to avoid stack overflows
    // This is absolutely essential for preventing stack overflows in large loops
    FORCE_DIRECT_OUTPUT.store(true, Ordering::Relaxed);
    debug_utils::debug_log("Direct output mode FORCED to prevent stack overflow");

    // We no longer allow disabling direct output mode as it causes stack overflows
    // This is a permanent fix to ensure stability

    // Log the current state
    debug_utils::debug_log(&format!("Buffered output initialized. Direct output: {}",
                                  FORCE_DIRECT_OUTPUT.load(Ordering::Relaxed)));

    // Set up a panic hook to detect stack overflows
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        // If this is a stack overflow, switch to direct output mode
        if panic_info.to_string().contains("stack overflow") {
            eprintln!("Stack overflow detected! Switching to direct output mode.");
            FORCE_DIRECT_OUTPUT.store(true, Ordering::Relaxed);

            // Log memory usage
            debug_utils::log_memory_usage();
        }

        // Call the default hook
        default_hook(panic_info);
    }));
}

// Function to increment stack depth and check for potential overflow
#[inline]
fn enter_function(name: &str) -> usize {
    // Track operation for debugging
    debug_utils::track_operation(name);

    // If direct output is forced, don't track stack depth
    if FORCE_DIRECT_OUTPUT.load(Ordering::Relaxed) {
        return 0;
    }

    if STACK_DEBUG_ENABLED.load(Ordering::Relaxed) == 0 {
        return 0;
    }

    let depth = STACK_DEPTH.fetch_add(1, Ordering::Relaxed);

    // If we're getting close to a stack overflow, log a warning
    if depth > MAX_STACK_DEPTH {
        // Get the current operation count
        let count = OPERATION_COUNTER.load(Ordering::Relaxed);

        // Log a warning with the current stack depth and operation count
        debug_utils::debug_log(&format!("WARNING: Deep stack detected! Depth: {}, Function: {}, Op Count: {}",
                 depth, name, count));

        // Check if we're approaching stack overflow
        if debug_utils::check_stack_depth(name) || depth > STACK_OVERFLOW_THRESHOLD {
            // Log critical warning
            debug_utils::debug_log(&format!("CRITICAL: Stack overflow imminent! Depth: {}", depth));

            // Disable stack debugging to avoid recursive panics
            STACK_DEBUG_ENABLED.store(0, Ordering::Relaxed);

            // Switch to direct output mode to avoid further stack growth
            eprintln!("Switching to direct output mode to avoid stack overflow");

            // Force all future operations to use direct output
            OPERATION_COUNTER.store(DIRECT_OUTPUT_THRESHOLD + 1, Ordering::Relaxed);

            // Set the global flag to force direct output for all future operations
            FORCE_DIRECT_OUTPUT.store(true, Ordering::Relaxed);

            // Log memory usage
            debug_utils::log_memory_usage();
        }
    }

    depth
}

// Function to decrement stack depth
#[inline]
fn exit_function(depth: usize) {
    if STACK_DEBUG_ENABLED.load(Ordering::Relaxed) == 0 || depth == 0 {
        return;
    }

    STACK_DEPTH.fetch_sub(1, Ordering::Relaxed);
}

thread_local! {
    // Global output buffer for print operations
    static OUTPUT_BUFFER: RefCell<Vec<u8>> = RefCell::new(Vec::with_capacity(BUFFER_CAPACITY));
}

/// Flush the output buffer to stdout
pub fn flush_output_buffer() {
    OUTPUT_BUFFER.with(|buffer| {
        let mut buffer = buffer.borrow_mut();
        if !buffer.is_empty() {
            let _ = io::stdout().write_all(&buffer);
            let _ = io::stdout().flush();
            buffer.clear();
        }
    });
}

/// Write a string to the output buffer
pub fn write_to_buffer(s: &str) {
    // Use a performance tracker to monitor this function
    let _perf = debug_utils::PerformanceTracker::new("write_to_buffer", 5);

    // CRITICAL: Always use direct output to prevent stack overflow
    // This is the most reliable way to prevent stack overflows in large loops
    // Write directly to stdout
    let _ = io::stdout().write_all(s.as_bytes());
    return;

    // The code below is no longer used but kept for reference
    /*
    // Track stack depth for debugging
    let depth = enter_function("write_to_buffer");

    // If we're getting too deep, switch to direct output
    if depth > MAX_STACK_DEPTH {
        debug_utils::debug_log(&format!("Stack depth {} exceeded threshold in write_to_buffer", depth));
        let _ = io::stdout().write_all(s.as_bytes());
        exit_function(depth);
        return;
    }

    // For very large strings, write directly to stdout to avoid buffer overflow
    if s.len() > BUFFER_CAPACITY {
        debug_utils::debug_log(&format!("Large string ({}B) detected, using direct output", s.len()));
        let _ = io::stdout().write_all(s.as_bytes());
        exit_function(depth);
        return;
    }
    */

    /* Buffered output code removed to prevent stack overflow
    OUTPUT_BUFFER.with(|buffer| {
        let mut buffer = buffer.borrow_mut();

        // If adding this string would overflow the buffer, flush first
        if buffer.len() + s.len() > BUFFER_CAPACITY {
            let _ = io::stdout().write_all(&buffer);
            buffer.clear();
        }

        // Add the string to the buffer
        buffer.extend_from_slice(s.as_bytes());

        // Auto-flush if buffer gets too large
        if buffer.len() > FLUSH_THRESHOLD {
            let _ = io::stdout().write_all(&buffer);
            let _ = io::stdout().flush();
            buffer.clear();
        }
    });

    exit_function(depth);
    */
}

/// Write a string with a newline to the output buffer and flush
pub fn writeln_to_buffer(s: &str) {
    // CRITICAL: Always use direct output to prevent stack overflow
    // This is the most reliable way to prevent stack overflows in large loops
    let _ = io::stdout().write_all(s.as_bytes());
    let _ = io::stdout().write_all(b"\n");
    let _ = io::stdout().flush();
    return;

    /* Buffered output code removed to prevent stack overflow
    // For very large strings, write directly to stdout to avoid buffer overflow
    if s.len() > BUFFER_CAPACITY {
        let _ = io::stdout().write_all(s.as_bytes());
        let _ = io::stdout().write_all(b"\n");
        let _ = io::stdout().flush();
        return;
    }

    OUTPUT_BUFFER.with(|buffer| {
        let mut buffer = buffer.borrow_mut();

        // If adding this string would overflow the buffer, flush first
        if buffer.len() + s.len() + 1 > BUFFER_CAPACITY {
            let _ = io::stdout().write_all(&buffer);
            buffer.clear();
        }

        // Add the string and newline to the buffer
        buffer.extend_from_slice(s.as_bytes());
        buffer.push(b'\n');

        // Always flush on newline for interactive output
        let _ = io::stdout().write_all(&buffer);
        let _ = io::stdout().flush();
        buffer.clear();
    });
    */
}

/// Write a single character to the output buffer
#[inline]
pub fn write_char_to_buffer(c: char) {
    // CRITICAL: Always use direct output to prevent stack overflow
    // This is the most reliable way to prevent stack overflows in large loops
    let mut buf = [0u8; 4];
    let s = c.encode_utf8(&mut buf);
    let _ = io::stdout().write_all(s.as_bytes());
    return;

    /* Buffered output code removed to prevent stack overflow
    // Check if we're in forced direct output mode first (fastest path)
    if FORCE_DIRECT_OUTPUT.load(Ordering::Relaxed) {
        // Write directly to stdout
        let mut buf = [0u8; 4];
        let s = c.encode_utf8(&mut buf);
        let _ = io::stdout().write_all(s.as_bytes());
        return;
    }

    // Track stack depth for debugging
    let depth = enter_function("write_char_to_buffer");

    // If we're getting too deep, switch to direct output
    if depth > MAX_STACK_DEPTH {
        eprintln!("Stack depth {} exceeded threshold in write_char_to_buffer", depth);
        let mut buf = [0u8; 4];
        let s = c.encode_utf8(&mut buf);
        let _ = io::stdout().write_all(s.as_bytes());
        exit_function(depth);
        return;
    }

    // Fast path for ASCII characters
    if c.is_ascii() {
        let mut buf = [0u8; 1];
        buf[0] = c as u8;
        OUTPUT_BUFFER.with(|buffer| {
            let mut buffer = buffer.borrow_mut();
            buffer.push(c as u8);

            // Auto-flush if buffer gets too large
            if buffer.len() > FLUSH_THRESHOLD {
                let _ = io::stdout().write_all(&buffer);
                let _ = io::stdout().flush();
                buffer.clear();
            }
        });
    } else {
        // For non-ASCII characters, use the standard approach
        let mut buf = [0u8; 4];
        let s = c.encode_utf8(&mut buf);
        write_to_buffer(s);
    }

    exit_function(depth);
    */
}

/// Write an integer to the output buffer
#[inline]
pub fn write_int_to_buffer(value: i64) {
    // Check if we're in forced direct output mode first (fastest path)
    if FORCE_DIRECT_OUTPUT.load(Ordering::Relaxed) {
        // Use the static buffer for direct output
        unsafe {
            // Convert the integer to a string directly
            let mut idx = 0;

            // Handle negative numbers
            let (is_negative, abs_value) = if value < 0 {
                (true, if value == i64::MIN { i64::MAX as u64 + 1 } else { (-value) as u64 })
            } else {
                (false, value as u64)
            };

            // Convert to digits (in reverse order)
            let mut num = abs_value;
            if num == 0 {
                NUMBER_BUFFER[idx] = b'0';
                idx += 1;
            } else {
                while num > 0 && idx < 31 {
                    NUMBER_BUFFER[idx] = b'0' + (num % 10) as u8;
                    idx += 1;
                    num /= 10;
                }

                if is_negative && idx < 31 {
                    NUMBER_BUFFER[idx] = b'-';
                    idx += 1;
                }
            }

            // Reverse the digits
            let mut i = 0;
            let mut j = idx - 1;
            while i < j {
                NUMBER_BUFFER[i] ^= NUMBER_BUFFER[j];
                NUMBER_BUFFER[j] ^= NUMBER_BUFFER[i];
                NUMBER_BUFFER[i] ^= NUMBER_BUFFER[j];
                i += 1;
                j -= 1;
            }

            // Write directly to stdout
            let _ = io::stdout().write_all(&NUMBER_BUFFER[0..idx]);
        }
        return;
    }

    // Track stack depth for debugging
    let depth = enter_function("write_int_to_buffer");

    // Increment operation counter and check if we need a checkpoint
    let count = OPERATION_COUNTER.fetch_add(1, Ordering::Relaxed);

    // Always use direct output for integers to avoid stack overflow
    // This is a critical optimization for large loops
    if true {
        // Log stack depth for debugging if it's the reason we're using direct output
        if depth > MAX_STACK_DEPTH {
            eprintln!("Stack depth {} exceeded threshold in write_int_to_buffer at operation {}",
                     depth, count);
        }
        // SAFETY: We're using this in a controlled way and ensuring thread safety
        // through atomic operations. The NUMBER_BUFFER is only used in this function
        // and we're careful to avoid race conditions.
        unsafe {
            // Convert the integer to a string directly using a non-recursive approach
            let mut idx = 0;

            // Handle negative numbers
            let (is_negative, abs_value) = if value < 0 {
                (true, if value == i64::MIN { i64::MAX as u64 + 1 } else { (-value) as u64 })
            } else {
                (false, value as u64)
            };

            // Convert to digits (in reverse order)
            let mut num = abs_value;
            if num == 0 {
                NUMBER_BUFFER[idx] = b'0';
                idx += 1;
            } else {
                while num > 0 && idx < 31 {
                    NUMBER_BUFFER[idx] = b'0' + (num % 10) as u8;
                    idx += 1;
                    num /= 10;
                }

                if is_negative && idx < 31 {
                    NUMBER_BUFFER[idx] = b'-';
                    idx += 1;
                }
            }

            // Reverse the digits
            let mut i = 0;
            let mut j = idx - 1;
            while i < j {
                NUMBER_BUFFER[i] ^= NUMBER_BUFFER[j];
                NUMBER_BUFFER[j] ^= NUMBER_BUFFER[i];
                NUMBER_BUFFER[i] ^= NUMBER_BUFFER[j];
                i += 1;
                j -= 1;
            }

            // Write directly to stdout without any function calls that could grow the stack
            let _ = io::stdout().write_all(&NUMBER_BUFFER[0..idx]);

            // Add a newline every 10 numbers to make output more readable
            if count % 10 == 0 {
                let _ = io::stdout().write_all(b"\n");
            }

            // Periodically flush to ensure output is visible
            if count % 500 == 0 {
                let _ = io::stdout().flush();
            }
        }

        exit_function(depth);
        return;
    }

    // More aggressive checkpointing for normal operation
    if count % CHECKPOINT_INTERVAL == 0 {
        // Force a complete flush to reset stack state
        OUTPUT_BUFFER.with(|buffer| {
            let mut buffer = buffer.borrow_mut();
            if !buffer.is_empty() {
                let _ = io::stdout().write_all(&buffer);
                let _ = io::stdout().flush();
                buffer.clear();
            }
        });
    }

    // Fast path for small integers (0-9) - direct implementation to avoid recursion
    if value >= 0 && value < 10 {
        OUTPUT_BUFFER.with(|buffer| {
            let mut buffer = buffer.borrow_mut();
            buffer.push(b'0' + value as u8);

            // Auto-flush if buffer gets too large
            if buffer.len() > FLUSH_THRESHOLD {
                let _ = io::stdout().write_all(&buffer);
                let _ = io::stdout().flush();
                buffer.clear();
            }
        });
        return;
    }

    // For medium to large integers, use direct output to avoid stack growth
    if value > 1_000 || value < -1_000 {
        // Use itoa for efficient formatting without recursion
        let mut buffer = itoa::Buffer::new();
        let s = buffer.format(value);

        // Write directly to stdout for values that might contribute to stack overflow
        if count > DIRECT_OUTPUT_THRESHOLD / 2 {
            let _ = io::stdout().write_all(s.as_bytes());
            return;
        }

        // Otherwise use the buffer system but avoid recursive calls
        OUTPUT_BUFFER.with(|buf| {
            let mut buf = buf.borrow_mut();

            // If adding this string would overflow the buffer, flush first
            if buf.len() + s.len() > BUFFER_CAPACITY {
                let _ = io::stdout().write_all(&buf);
                buf.clear();
            }

            // Add the string to the buffer
            buf.extend_from_slice(s.as_bytes());

            // Auto-flush if buffer gets too large
            if buf.len() > FLUSH_THRESHOLD {
                let _ = io::stdout().write_all(&buf);
                let _ = io::stdout().flush();
                buf.clear();
            }
        });
        return;
    }

    // Use stack-allocated buffer for small integers to avoid heap allocations
    let mut buffer = itoa::Buffer::new();
    let s = buffer.format(value);

    // Use a non-recursive implementation for the final write
    OUTPUT_BUFFER.with(|buf| {
        let mut buf = buf.borrow_mut();

        // If adding this string would overflow the buffer, flush first
        if buf.len() + s.len() > BUFFER_CAPACITY {
            let _ = io::stdout().write_all(&buf);
            buf.clear();
        }

        // Add the string to the buffer
        buf.extend_from_slice(s.as_bytes());

        // Auto-flush if buffer gets too large
        if buf.len() > FLUSH_THRESHOLD {
            let _ = io::stdout().write_all(&buf);
            let _ = io::stdout().flush();
            buf.clear();
        }
    });

    exit_function(depth);
}

/// Write a float to the output buffer
#[inline]
pub fn write_float_to_buffer(value: f64) {
    // Check if we're in high-iteration mode
    let count = OPERATION_COUNTER.load(Ordering::Relaxed);

    // For high iteration counts, write directly to stdout
    if count > DIRECT_OUTPUT_THRESHOLD {
        // Convert float to string directly and write to stdout
        // This avoids any potential stack growth from buffer operations
        let s = value.to_string();
        let _ = io::stdout().write_all(s.as_bytes());

        // Add a newline every 10 numbers to make output more readable
        if count % 10 == 0 {
            let _ = io::stdout().write_all(b"\n");
        }

        // Periodically flush to ensure output is visible
        if count % 500 == 0 {
            let _ = io::stdout().flush();
        }
        return;
    }

    // Use stack-allocated buffer for floats to avoid heap allocations
    let mut buffer = ryu::Buffer::new();
    let s = buffer.format(value);

    // Otherwise use the normal buffer path but avoid recursion
    OUTPUT_BUFFER.with(|buf| {
        let mut buf = buf.borrow_mut();

        // If adding this string would overflow the buffer, flush first
        if buf.len() + s.len() > BUFFER_CAPACITY {
            let _ = io::stdout().write_all(&buf);
            buf.clear();
        }

        // Add the string to the buffer
        buf.extend_from_slice(s.as_bytes());

        // Auto-flush if buffer gets too large
        if buf.len() > FLUSH_THRESHOLD {
            let _ = io::stdout().write_all(&buf);
            let _ = io::stdout().flush();
            buf.clear();
        }
    });
}

/// Write a boolean to the output buffer
#[inline]
pub fn write_bool_to_buffer(value: bool) {
    // Check if we're in high-iteration mode
    let count = OPERATION_COUNTER.load(Ordering::Relaxed);

    // For high iteration counts, write directly to stdout
    if count > DIRECT_OUTPUT_THRESHOLD {
        // Write directly to stdout without any buffer operations
        if value {
            let _ = io::stdout().write_all(b"True");
        } else {
            let _ = io::stdout().write_all(b"False");
        }

        // Add a newline every 10 values to make output more readable
        if count % 10 == 0 {
            let _ = io::stdout().write_all(b"\n");
        }

        // Periodically flush to ensure output is visible
        if count % 500 == 0 {
            let _ = io::stdout().flush();
        }
        return;
    }

    // Use static strings to avoid allocations
    let s = if value { "True" } else { "False" };

    // Use a non-recursive implementation
    OUTPUT_BUFFER.with(|buf| {
        let mut buf = buf.borrow_mut();

        // If adding this string would overflow the buffer, flush first
        if buf.len() + s.len() > BUFFER_CAPACITY {
            let _ = io::stdout().write_all(&buf);
            buf.clear();
        }

        // Add the string to the buffer
        buf.extend_from_slice(s.as_bytes());

        // Auto-flush if buffer gets too large
        if buf.len() > FLUSH_THRESHOLD {
            let _ = io::stdout().write_all(&buf);
            let _ = io::stdout().flush();
            buf.clear();
        }
    });
}

/// Write a newline to the output buffer
#[inline]
pub fn write_newline() {
    // CRITICAL: Always use direct output to prevent stack overflow
    // This is the most reliable way to prevent stack overflows in large loops
    let _ = io::stdout().write_all(&NEWLINE);
    let _ = io::stdout().flush(); // Flush to ensure newline is visible immediately
    return;

    /* Buffered output code removed to prevent stack overflow
    // Check if we're in forced direct output mode first (fastest path)
    if FORCE_DIRECT_OUTPUT.load(Ordering::Relaxed) {
        // Write directly to stdout
        let _ = io::stdout().write_all(&NEWLINE);
        return;
    }

    // Track stack depth for debugging
    let depth = enter_function("write_newline");

    // Check if we're in high-iteration mode
    let count = OPERATION_COUNTER.load(Ordering::Relaxed);

    // For high iteration counts or deep stack, write directly to stdout
    if count > DIRECT_OUTPUT_THRESHOLD || depth > MAX_STACK_DEPTH {
        // Log stack depth for debugging if it's the reason we're using direct output
        if depth > MAX_STACK_DEPTH {
            eprintln!("Stack depth {} exceeded threshold in write_newline", depth);
        }

        // Write directly to stdout without any buffer operations
        let _ = io::stdout().write_all(&NEWLINE);
        exit_function(depth);
        return;
    }

    // Otherwise use the normal buffer path
    write_char_to_buffer('\n');

    exit_function(depth);
    */
}