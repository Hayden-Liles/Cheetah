// debug_utils.rs - Utilities for debugging runtime issues

use std::backtrace::Backtrace;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::Instant;

// Debug flags
static DEBUG_ENABLED: AtomicBool = AtomicBool::new(false);
static STACK_TRACE_ENABLED: AtomicBool = AtomicBool::new(false);
static PERFORMANCE_TRACKING_ENABLED: AtomicBool = AtomicBool::new(false);
static OPERATION_COUNT: AtomicUsize = AtomicUsize::new(0);
static LAST_STACK_TRACE_TIME: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
static STACK_TRACE_INTERVAL_MS: u64 = 1000;

// Initialize debug utilities
pub fn init() {
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

    if DEBUG_ENABLED.load(Ordering::Relaxed) && count % 10000 == 0 {
        eprintln!(
            "[DEBUG] Operation count: {}, Last operation: {}",
            count, operation_name
        );
    }

    if STACK_TRACE_ENABLED.load(Ordering::Relaxed) && count % 100000 == 0 {
        let now = Instant::now().elapsed().as_millis() as u64;
        let last = LAST_STACK_TRACE_TIME.load(Ordering::Relaxed);

        if now - last > STACK_TRACE_INTERVAL_MS {
            LAST_STACK_TRACE_TIME.store(now, Ordering::Relaxed);
            let bt = Backtrace::capture();
            eprintln!(
                "[STACK TRACE] Operation count: {}, Backtrace: {:?}",
                count, bt
            );
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

            if elapsed_ms > self.threshold_ms {
                eprintln!(
                    "[PERF] Operation '{}' took {}ms",
                    self.operation, elapsed_ms
                );
            }
        }
    }
}

// Helper to check if we're approaching stack overflow
pub fn check_stack_depth(function_name: &str) -> bool {
    if !DEBUG_ENABLED.load(Ordering::Relaxed) {
        return false;
    }

    let result = std::panic::catch_unwind(|| {
        let _buffer = [0u8; 1024];
    });

    if result.is_err() {
        eprintln!(
            "[STACK WARNING] Stack space low in function: {}",
            function_name
        );

        if STACK_TRACE_ENABLED.load(Ordering::Relaxed) {
            let bt = Backtrace::capture();
            eprintln!(
                "[STACK TRACE] Function: {}, Backtrace: {:?}",
                function_name, bt
            );
        }

        return true;
    }

    false
}
