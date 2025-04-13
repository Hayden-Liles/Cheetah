// debug_utils.rs - Utilities for debugging runtime issues

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::backtrace::Backtrace;
use std::time::Instant;

// Debug flags
static DEBUG_ENABLED: AtomicBool = AtomicBool::new(false);
static STACK_TRACE_ENABLED: AtomicBool = AtomicBool::new(false);
static PERFORMANCE_TRACKING_ENABLED: AtomicBool = AtomicBool::new(false);
static OPERATION_COUNT: AtomicUsize = AtomicUsize::new(0);
static LAST_STACK_TRACE_TIME: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
static STACK_TRACE_INTERVAL_MS: u64 = 1000; // Only capture stack trace every second

// Initialize debug utilities
pub fn init() {
    // Check environment variables to enable debugging features
    if std::env::var("CHEETAH_DEBUG").is_ok() {
        eprintln!("Debug mode enabled");
        DEBUG_ENABLED.store(true, Ordering::Relaxed);
    }

    if std::env::var("CHEETAH_STACK_TRACE").is_ok() {
        eprintln!("Stack trace debugging enabled");
        STACK_TRACE_ENABLED.store(true, Ordering::Relaxed);
    }

    if std::env::var("CHEETAH_PERF_TRACKING").is_ok() {
        eprintln!("Performance tracking enabled");
        PERFORMANCE_TRACKING_ENABLED.store(true, Ordering::Relaxed);
    }

    // Reset operation count
    OPERATION_COUNT.store(0, Ordering::Relaxed);
}

// Log a debug message if debug mode is enabled
pub fn debug_log(message: &str) {
    if DEBUG_ENABLED.load(Ordering::Relaxed) {
        eprintln!("[DEBUG] {}", message);
    }
}

// Track an operation and periodically log stack information
pub fn track_operation(operation_name: &str) -> usize {
    let count = OPERATION_COUNT.fetch_add(1, Ordering::Relaxed);

    // Only log every 10,000 operations to avoid flooding
    if DEBUG_ENABLED.load(Ordering::Relaxed) && count % 10000 == 0 {
        eprintln!("[DEBUG] Operation count: {}, Last operation: {}", count, operation_name);
    }

    // Capture stack trace periodically if enabled
    if STACK_TRACE_ENABLED.load(Ordering::Relaxed) && count % 100000 == 0 {
        // Check if enough time has passed since the last stack trace
        let now = Instant::now().elapsed().as_millis() as u64;
        let last = LAST_STACK_TRACE_TIME.load(Ordering::Relaxed);

        if now - last > STACK_TRACE_INTERVAL_MS {
            LAST_STACK_TRACE_TIME.store(now, Ordering::Relaxed);
            let bt = Backtrace::capture();
            eprintln!("[STACK TRACE] Operation count: {}, Backtrace: {:?}", count, bt);
        }
    }

    count
}

// Performance tracking utilities
pub struct PerformanceTracker {
    operation: String,
    start_time: Instant,
    threshold_ms: u64,
}

impl PerformanceTracker {
    pub fn new(operation: &str, threshold_ms: u64) -> Self {
        Self {
            operation: operation.to_string(),
            start_time: Instant::now(),
            threshold_ms,
        }
    }
}

impl Drop for PerformanceTracker {
    fn drop(&mut self) {
        if PERFORMANCE_TRACKING_ENABLED.load(Ordering::Relaxed) {
            let elapsed = self.start_time.elapsed();
            let elapsed_ms = elapsed.as_millis() as u64;

            // Only log operations that take longer than the threshold
            if elapsed_ms > self.threshold_ms {
                eprintln!(
                    "[PERF] Operation '{}' took {}ms",
                    self.operation,
                    elapsed_ms
                );
            }
        }
    }
}

// Helper to track memory usage
pub fn log_memory_usage() {
    // Always log memory usage regardless of debug flag
    // This is critical for diagnosing stack overflow issues

    // This is a simple approximation - in a real implementation you might
    // want to use a crate like psutil to get more accurate memory usage
    #[cfg(target_os = "linux")]
    {
        if let Ok(status) = std::fs::read_to_string("/proc/self/status") {
            if let Some(line) = status.lines().find(|l| l.starts_with("VmRSS:")) {
                eprintln!("[MEMORY] {}", line.trim());
            }

            // Also log stack usage if available
            if let Some(line) = status.lines().find(|l| l.starts_with("VmStk:")) {
                eprintln!("[STACK] {}", line.trim());
            }
        }
    }

    // For other platforms, just log that we can't get memory info
    #[cfg(not(target_os = "linux"))]
    {
        eprintln!("[MEMORY] Memory usage information not available on this platform");
    }
}

// Helper to check if we're approaching stack overflow
pub fn check_stack_depth(function_name: &str) -> bool {
    if !DEBUG_ENABLED.load(Ordering::Relaxed) {
        return false;
    }

    // A simple way to estimate remaining stack space is to allocate a small array
    // If this fails, we're close to stack overflow
    let result = std::panic::catch_unwind(|| {
        let _buffer = [0u8; 1024]; // 1KB buffer
    });

    if result.is_err() {
        eprintln!("[STACK WARNING] Stack space low in function: {}", function_name);

        // Capture a stack trace
        if STACK_TRACE_ENABLED.load(Ordering::Relaxed) {
            let bt = Backtrace::capture();
            eprintln!("[STACK TRACE] Function: {}, Backtrace: {:?}", function_name, bt);
        }

        return true;
    }

    false
}
