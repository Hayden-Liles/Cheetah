// circular_buffer.rs - Optimized circular buffer for output operations
// This file implements a pre-allocated circular buffer to avoid constant memory allocation

use std::io::{self, Write, Result as IoResult};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::cell::RefCell;
use std::thread_local;
use std::collections::HashMap;

// Constants for buffer management
const BUFFER_CAPACITY: usize = 8192; // 8KB buffer (reduced from 32KB)
const FLUSH_THRESHOLD: usize = 4096; // Flush at 4KB (reduced from 16KB)
const MAX_INT_LENGTH: usize = 20; // Maximum length of an integer as string
const MAX_INTERNED_STRINGS: usize = 64; // Maximum number of interned strings (reduced from 1024)

// Global counters for buffer operations
static BUFFER_OPERATIONS: AtomicUsize = AtomicUsize::new(0);
static BUFFER_BYTES_WRITTEN: AtomicUsize = AtomicUsize::new(0);
static BUFFER_BYTES_SAVED: AtomicUsize = AtomicUsize::new(0);

// Thread-local pre-allocated circular buffer and string cache
thread_local! {
    static OUTPUT_BUFFER: RefCell<CircularBuffer> = RefCell::new(CircularBuffer::new(BUFFER_CAPACITY));
    static STRING_CACHE: RefCell<HashMap<u64, Vec<u8>>> = RefCell::new(HashMap::with_capacity(MAX_INTERNED_STRINGS));
    static LAST_STRING: RefCell<Option<(Vec<u8>, usize)>> = RefCell::new(None); // (string, repeat count)
}

/// Circular buffer for output operations
pub struct CircularBuffer {
    buffer: Vec<u8>,
    read_pos: usize,
    write_pos: usize,
    size: usize,
    capacity: usize,
}

impl CircularBuffer {
    /// Create a new circular buffer
    pub fn new(capacity: usize) -> Self {
        CircularBuffer {
            buffer: vec![0; capacity],
            read_pos: 0,
            write_pos: 0,
            size: 0,
            capacity,
        }
    }

    /// Write a byte to the buffer
    pub fn write_byte(&mut self, byte: u8) -> io::Result<()> {
        if self.size == self.capacity {
            // Buffer is full, flush it
            self.flush()?;
        }

        self.buffer[self.write_pos] = byte;
        self.write_pos = (self.write_pos + 1) % self.capacity;
        self.size += 1;

        Ok(())
    }

    /// Write a slice to the buffer
    pub fn write(&mut self, slice: &[u8]) -> io::Result<()> {
        // If the slice is larger than the buffer capacity, write it directly
        if slice.len() > self.capacity {
            // Flush the buffer first
            self.flush()?;

            // Write the slice directly to stdout
            io::stdout().write_all(slice)?;
            return Ok(());
        }

        // If the slice won't fit in the remaining space, flush the buffer
        if slice.len() > self.capacity - self.size {
            self.flush()?;
        }

        // Write the slice to the buffer
        for &byte in slice {
            self.write_byte(byte)?;
        }

        // Auto-flush if buffer gets too large
        if self.size > FLUSH_THRESHOLD {
            self.flush()?;
        }

        Ok(())
    }

    /// Flush the buffer to stdout
    pub fn flush(&mut self) -> io::Result<()> {
        if self.size == 0 {
            return Ok(());
        }

        // If the buffer is contiguous, write it all at once
        if self.read_pos < self.write_pos {
            io::stdout().write_all(&self.buffer[self.read_pos..self.write_pos])?;
        } else {
            // Otherwise, write in two parts
            io::stdout().write_all(&self.buffer[self.read_pos..self.capacity])?;
            io::stdout().write_all(&self.buffer[0..self.write_pos])?;
        }

        // Flush stdout
        io::stdout().flush()?;

        // Reset the buffer
        self.read_pos = 0;
        self.write_pos = 0;
        self.size = 0;

        Ok(())
    }

    /// Write an integer to the buffer
    pub fn write_int(&mut self, value: i64) -> io::Result<()> {
        // Use a stack-allocated buffer for the integer
        let mut int_buffer = [0u8; MAX_INT_LENGTH];
        let mut pos = MAX_INT_LENGTH;

        // Handle zero specially
        if value == 0 {
            self.write_byte(b'0')?;
            return Ok(());
        }

        // Handle negative numbers
        let mut val = value;
        if value < 0 {
            self.write_byte(b'-')?;
            val = -val;
        }

        // Convert the integer to a string
        while val > 0 && pos > 0 {
            pos -= 1;
            int_buffer[pos] = b'0' + (val % 10) as u8;
            val /= 10;
        }

        // Write the string to the buffer
        self.write(&int_buffer[pos..MAX_INT_LENGTH])
    }

    /// Write a float to the buffer
    pub fn write_float(&mut self, value: f64) -> io::Result<()> {
        // Use ryu for fast float-to-string conversion
        let mut buffer = ryu::Buffer::new();
        let s = buffer.format(value);

        // Write the string to the buffer
        self.write(s.as_bytes())
    }

    /// Write a boolean to the buffer
    pub fn write_bool(&mut self, value: bool) -> io::Result<()> {
        // Use static strings to avoid allocations
        let s = if value { b"True" as &[u8] } else { b"False" as &[u8] };

        // Write the string to the buffer
        self.write(s)
    }

    /// Write a character to the buffer
    pub fn write_char(&mut self, c: char) -> io::Result<()> {
        // Fast path for ASCII characters
        if c.is_ascii() {
            self.write_byte(c as u8)?;
        } else {
            // For non-ASCII characters, use a small buffer
            let mut buf = [0u8; 4];
            let s = c.encode_utf8(&mut buf);
            self.write(s.as_bytes())?;
        }

        Ok(())
    }

    /// Write a string to the buffer
    pub fn write_str(&mut self, s: &str) -> io::Result<()> {
        self.write(s.as_bytes())
    }

    /// Write a string with a newline to the buffer and flush
    pub fn writeln_str(&mut self, s: &str) -> io::Result<()> {
        self.write(s.as_bytes())?;
        self.write_byte(b'\n')?;
        self.flush()
    }
}

/// Initialize the circular buffer
pub fn init() {
    // Reset the buffer operations counter
    BUFFER_OPERATIONS.store(0, Ordering::Relaxed);
    BUFFER_BYTES_WRITTEN.store(0, Ordering::Relaxed);
    BUFFER_BYTES_SAVED.store(0, Ordering::Relaxed);

    // Clear the string cache
    STRING_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        cache.clear();
    });

    // Reset the last string
    LAST_STRING.with(|last| {
        *last.borrow_mut() = None;
    });
}

/// Flush the output buffer
pub fn flush_output_buffer() -> IoResult<()> {
    OUTPUT_BUFFER.with(|buffer| {
        let mut buffer = buffer.borrow_mut();
        buffer.flush()
    }).or_else(|_| io::stdout().flush())
}

/// Write a string to the output buffer
pub fn write_to_buffer(s: &str) -> IoResult<()> {
    // Track buffer operations
    BUFFER_OPERATIONS.fetch_add(1, Ordering::Relaxed);

    OUTPUT_BUFFER.with(|buffer| {
        let mut buffer = buffer.borrow_mut();
        buffer.write_str(s)
    }).or_else(|_| io::stdout().write_all(s.as_bytes()))
}

/// Write a string with a newline to the output buffer and flush
pub fn writeln_to_buffer(s: &str) -> IoResult<()> {
    // Track buffer operations
    BUFFER_OPERATIONS.fetch_add(1, Ordering::Relaxed);

    OUTPUT_BUFFER.with(|buffer| {
        let mut buffer = buffer.borrow_mut();
        buffer.writeln_str(s)
    }).or_else(|_| -> IoResult<()> {
        io::stdout().write_all(s.as_bytes())?;
        io::stdout().write_all(b"\n")?;
        io::stdout().flush()
    })
}

/// Write an integer to the output buffer
pub fn write_int_to_buffer(value: i64) -> IoResult<()> {
    // Track buffer operations
    BUFFER_OPERATIONS.fetch_add(1, Ordering::Relaxed);

    OUTPUT_BUFFER.with(|buffer| {
        let mut buffer = buffer.borrow_mut();
        buffer.write_int(value)
    }).or_else(|_| write!(io::stdout(), "{}", value))
}

/// Write a float to the output buffer
pub fn write_float_to_buffer(value: f64) -> IoResult<()> {
    // Track buffer operations
    BUFFER_OPERATIONS.fetch_add(1, Ordering::Relaxed);

    OUTPUT_BUFFER.with(|buffer| {
        let mut buffer = buffer.borrow_mut();
        buffer.write_float(value)
    }).or_else(|_| write!(io::stdout(), "{}", value))
}

/// Write a boolean to the output buffer
pub fn write_bool_to_buffer(value: bool) -> IoResult<()> {
    // Track buffer operations
    BUFFER_OPERATIONS.fetch_add(1, Ordering::Relaxed);

    OUTPUT_BUFFER.with(|buffer| {
        let mut buffer = buffer.borrow_mut();
        buffer.write_bool(value)
    }).or_else(|_| write!(io::stdout(), "{}", value))
}

/// Write a character to the output buffer
pub fn write_char_to_buffer(c: char) -> IoResult<()> {
    // Track buffer operations
    BUFFER_OPERATIONS.fetch_add(1, Ordering::Relaxed);

    OUTPUT_BUFFER.with(|buffer| {
        let mut buffer = buffer.borrow_mut();
        buffer.write_char(c)
    }).or_else(|_| write!(io::stdout(), "{}", c))
}

/// Clean up the circular buffer
pub fn cleanup() {
    // Flush the buffer
    let _ = flush_output_buffer();

    // Log buffer operations and compression stats
    let ops = BUFFER_OPERATIONS.load(Ordering::Relaxed);
    let bytes_written = BUFFER_BYTES_WRITTEN.load(Ordering::Relaxed);
    let bytes_saved = BUFFER_BYTES_SAVED.load(Ordering::Relaxed);

    if ops > 0 {
        eprintln!("[BUFFER INFO] {} buffer operations performed", ops);
        if bytes_written > 0 && bytes_saved > 0 {
            let compression_ratio = (bytes_written as f64) / ((bytes_written - bytes_saved) as f64);
            eprintln!("[BUFFER INFO] Compression ratio: {:.2}x ({} bytes saved)",
                     compression_ratio, bytes_saved);
        }
    }

    // Clear the string cache
    STRING_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        cache.clear();
    });

    // Reset the last string
    LAST_STRING.with(|last| {
        *last.borrow_mut() = None;
    });
}
