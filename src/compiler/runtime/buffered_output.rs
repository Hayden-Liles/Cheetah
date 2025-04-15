// optimized_buffered_output.rs - Efficient buffered output system

use std::cell::RefCell;
use std::io::{self, Write};
use std::sync::atomic::{AtomicUsize, AtomicBool, Ordering};

// Import fast number formatting libraries
use itoa;
use ryu;

// Constants for buffer management
const BUFFER_CAPACITY: usize = 32768; // 32KB buffer - increased from 1KB
const FLUSH_THRESHOLD: usize = 24576; // Flush at 24KB - 75% of capacity
// Removed unused constant MAX_INT_LENGTH

// Global counters for buffer operations
static BUFFER_OPERATIONS: AtomicUsize = AtomicUsize::new(0);
static FORCE_DIRECT_OUTPUT: AtomicBool = AtomicBool::new(false);

// Pre-allocated number buffer to avoid allocations
static mut NUMBER_BUFFER: [u8; 32] = [0; 32];

// Static buffers for common characters to avoid allocations
static NEWLINE: [u8; 1] = [b'\n'];
// Removed unused static SPACE

// Thread-local buffer
thread_local! {
    static OUTPUT_BUFFER: RefCell<Vec<u8>> = RefCell::new(Vec::with_capacity(BUFFER_CAPACITY));
}

// Static number formatter for integers
// This avoids many allocations when printing numbers
static mut INT_FORMATTERS: [Option<itoa::Buffer>; 10] = [None, None, None, None, None, None, None, None, None, None];
static INT_FORMATTER_INDEX: AtomicUsize = AtomicUsize::new(0);

/// Initialize the buffered output system
pub fn init() {
    // Reset counters
    BUFFER_OPERATIONS.store(0, Ordering::Relaxed);
    FORCE_DIRECT_OUTPUT.store(false, Ordering::Relaxed);

    // Ensure the buffer is empty
    OUTPUT_BUFFER.with(|buffer| {
        let mut buffer = buffer.borrow_mut();
        buffer.clear();
        // Pre-allocate the full capacity
        buffer.reserve(BUFFER_CAPACITY);
    });

    // Pre-initialize static formatters
    for i in 0..10 {
        unsafe {
            INT_FORMATTERS[i] = Some(itoa::Buffer::new());
        }
    }
}

/// Flush the output buffer to stdout
#[inline(always)]
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
#[inline(always)]
pub fn write_to_buffer(s: &str) {
    // Fast path for empty strings
    if s.is_empty() {
        return;
    }

    // Check if direct output is forced
    if FORCE_DIRECT_OUTPUT.load(Ordering::Relaxed) {
        let _ = io::stdout().write_all(s.as_bytes());
        return;
    }

    // Record this operation
    BUFFER_OPERATIONS.fetch_add(1, Ordering::Relaxed);

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
}

/// Write a string with a newline to the output buffer and flush
#[inline(always)]
pub fn writeln_to_buffer(s: &str) {
    // Check if direct output is forced
    if FORCE_DIRECT_OUTPUT.load(Ordering::Relaxed) {
        let _ = io::stdout().write_all(s.as_bytes());
        let _ = io::stdout().write_all(&NEWLINE);
        let _ = io::stdout().flush();
        return;
    }

    // For very large strings, write directly to stdout
    if s.len() > BUFFER_CAPACITY {
        let _ = io::stdout().write_all(s.as_bytes());
        let _ = io::stdout().write_all(&NEWLINE);
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
}

/// Write a single character to the output buffer
#[inline(always)]
pub fn write_char_to_buffer(c: char) {
    // Check if direct output is forced
    if FORCE_DIRECT_OUTPUT.load(Ordering::Relaxed) {
        let mut buf = [0u8; 4];
        let s = c.encode_utf8(&mut buf);
        let _ = io::stdout().write_all(s.as_bytes());
        return;
    }

    // Fast path for ASCII characters
    if c.is_ascii() {
        OUTPUT_BUFFER.with(|buffer| {
            let mut buffer = buffer.borrow_mut();

            // If the buffer is full, flush it
            if buffer.len() + 1 > BUFFER_CAPACITY {
                let _ = io::stdout().write_all(&buffer);
                buffer.clear();
            }

            buffer.push(c as u8);

            // Auto-flush if buffer gets too large
            if buffer.len() > FLUSH_THRESHOLD {
                let _ = io::stdout().write_all(&buffer);
                let _ = io::stdout().flush();
                buffer.clear();
            }
        });
    } else {
        // For non-ASCII characters, use a small buffer
        let mut buf = [0u8; 4];
        let s = c.encode_utf8(&mut buf);
        write_to_buffer(s);
    }
}

/// Ultra-optimized integer writer - critical for performance in number-heavy loops
#[inline(always)]
pub fn write_int_to_buffer(value: i64) {
    // Check if direct output is forced
    if FORCE_DIRECT_OUTPUT.load(Ordering::Relaxed) {
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
            return;
        }
    }

    // Use a shared formatter pool for efficiency
    let formatter_idx = INT_FORMATTER_INDEX.fetch_add(1, Ordering::Relaxed) % 10;
    let s = unsafe {
        INT_FORMATTERS[formatter_idx].as_mut().expect("Formatter not initialized").format(value)
    };

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
}

/// Write a float to the output buffer
#[inline(always)]
pub fn write_float_to_buffer(value: f64) {
    // Check if direct output is forced
    if FORCE_DIRECT_OUTPUT.load(Ordering::Relaxed) {
        // Use ryu for fast float formatting
        let mut buffer = ryu::Buffer::new();
        let s = buffer.format(value);
        let _ = io::stdout().write_all(s.as_bytes());
        return;
    }

    // Use stack-allocated buffer for float formatting
    let mut buffer = ryu::Buffer::new();
    let s = buffer.format(value);

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
}

/// Write a boolean to the output buffer
#[inline(always)]
pub fn write_bool_to_buffer(value: bool) {
    // Check if direct output is forced
    if FORCE_DIRECT_OUTPUT.load(Ordering::Relaxed) {
        if value {
            let _ = io::stdout().write_all(b"True");
        } else {
            let _ = io::stdout().write_all(b"False");
        }
        return;
    }

    // Use static strings to avoid allocations
    let s = if value { "True" } else { "False" };

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
}

/// Write a newline to the output buffer
#[inline(always)]
pub fn write_newline() {
    // Check if direct output is forced
    if FORCE_DIRECT_OUTPUT.load(Ordering::Relaxed) {
        let _ = io::stdout().write_all(&NEWLINE);
        let _ = io::stdout().flush();
        return;
    }

    OUTPUT_BUFFER.with(|buffer| {
        let mut buffer = buffer.borrow_mut();

        // If adding a newline would overflow the buffer, flush first
        if buffer.len() + 1 > BUFFER_CAPACITY {
            let _ = io::stdout().write_all(&buffer);
            buffer.clear();
        }

        // Add the newline to the buffer
        buffer.push(b'\n');

        // Flush on newline for interactive output
        let _ = io::stdout().write_all(&buffer);
        let _ = io::stdout().flush();
        buffer.clear();
    });
}