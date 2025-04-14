use std::time::Instant;
use std::f64;

fn main() {
    println!("Testing loop unrolling performance");

    // Number of iterations for benchmarking
    const ITERATIONS: i32 = 10_000_000;

    // Test small loop (should be fully unrolled)
    println!("\nSmall loop (8 iterations) x {} runs:", ITERATIONS);
    let mut total_unrolled = 0;
    let mut total_normal = 0;

    let start = Instant::now();
    for _ in 0..ITERATIONS {
        total_unrolled += test_small_loop_unrolled();
    }
    let unrolled_duration = start.elapsed();

    let start = Instant::now();
    for _ in 0..ITERATIONS {
        total_normal += test_small_loop_normal();
    }
    let normal_duration = start.elapsed();

    println!("Unrolled time: {:?}", unrolled_duration);
    println!("Normal time: {:?}", normal_duration);
    println!("Speedup: {:.2}x", normal_duration.as_nanos() as f64 / unrolled_duration.as_nanos() as f64);
    println!("Unrolled result: {}", total_unrolled / ITERATIONS);
    println!("Normal result: {}", total_normal / ITERATIONS);

    // Test medium loop (should be partially unrolled)
    println!("\nMedium loop (100 iterations) x {} runs:", ITERATIONS / 100);
    let mut total_unrolled = 0;
    let mut total_normal = 0;

    let start = Instant::now();
    for _ in 0..(ITERATIONS / 100) {
        total_unrolled += test_medium_loop_unrolled();
    }
    let unrolled_duration = start.elapsed();

    let start = Instant::now();
    for _ in 0..(ITERATIONS / 100) {
        total_normal += test_medium_loop_normal();
    }
    let normal_duration = start.elapsed();

    println!("Unrolled time: {:?}", unrolled_duration);
    println!("Normal time: {:?}", normal_duration);
    println!("Speedup: {:.2}x", normal_duration.as_nanos() as f64 / unrolled_duration.as_nanos() as f64);
    println!("Unrolled result: {}", total_unrolled / (ITERATIONS / 100));
    println!("Normal result: {}", total_normal / (ITERATIONS / 100));

    // Test large loop (should be chunked)
    println!("\nLarge loop (10000 iterations) x {} runs:", ITERATIONS / 10000);
    let mut total_unrolled = 0;
    let mut total_normal = 0;

    let start = Instant::now();
    for _ in 0..(ITERATIONS / 10000) {
        total_unrolled += test_large_loop_unrolled();
    }
    let unrolled_duration = start.elapsed();

    let start = Instant::now();
    for _ in 0..(ITERATIONS / 10000) {
        total_normal += test_large_loop_normal();
    }
    let normal_duration = start.elapsed();

    println!("Unrolled time: {:?}", unrolled_duration);
    println!("Normal time: {:?}", normal_duration);
    println!("Speedup: {:.2}x", normal_duration.as_nanos() as f64 / unrolled_duration.as_nanos() as f64);
    println!("Unrolled result: {}", total_unrolled / (ITERATIONS / 10000));
    println!("Normal result: {}", total_normal / (ITERATIONS / 10000));

    // Test with more complex computation to better demonstrate the benefits
    println!("\nComplex computation in loop:");
    let start = Instant::now();
    let complex_unrolled = test_complex_unrolled(1000000);
    let unrolled_duration = start.elapsed();

    let start = Instant::now();
    let complex_normal = test_complex_normal(1000000);
    let normal_duration = start.elapsed();

    println!("Unrolled time: {:?}", unrolled_duration);
    println!("Normal time: {:?}", normal_duration);
    println!("Speedup: {:.2}x", normal_duration.as_nanos() as f64 / unrolled_duration.as_nanos() as f64);
    println!("Unrolled result: {}", complex_unrolled);
    println!("Normal result: {}", complex_normal);
}

// Small loop tests
fn test_small_loop_unrolled() -> i32 {
    let mut result = 0;

    // Manually unrolled loop with 8 iterations
    result += 0;
    result += 1;
    result += 2;
    result += 3;
    result += 4;
    result += 5;
    result += 6;
    result += 7;

    result
}

fn test_small_loop_normal() -> i32 {
    let mut result = 0;

    // Normal loop with 8 iterations
    for i in 0..8 {
        result += i;
    }

    result
}

// Medium loop tests
fn test_medium_loop_unrolled() -> i32 {
    let mut result = 0;

    // Partially unrolled loop with 100 iterations
    for i in (0..100).step_by(4) {
        if i < 100 { result += i; }
        if i + 1 < 100 { result += i + 1; }
        if i + 2 < 100 { result += i + 2; }
        if i + 3 < 100 { result += i + 3; }
    }

    result
}

fn test_medium_loop_normal() -> i32 {
    let mut result = 0;

    // Normal loop with 100 iterations
    for i in 0..100 {
        result += i;
    }

    result
}

// Large loop tests
fn test_large_loop_unrolled() -> i32 {
    let mut result = 0;

    // Chunked loop with 10000 iterations
    for chunk in (0..10000).step_by(1000) {
        let end = std::cmp::min(chunk + 1000, 10000);
        for i in chunk..end {
            result += i;
        }
    }

    result
}

fn test_large_loop_normal() -> i32 {
    let mut result = 0;

    // Normal loop with 10000 iterations
    for i in 0..10000 {
        result += i;
    }

    result
}

// Complex computation tests - these better demonstrate the benefits of loop unrolling
// because they do more work per iteration
fn test_complex_unrolled(iterations: i32) -> f64 {
    let mut result = 0.0;

    // Process in chunks of 4 iterations
    let chunks = iterations / 4;
    let remainder = iterations % 4;

    for _ in 0..chunks {
        // Unrolled loop with 4 iterations per chunk
        result = f64::sqrt(result + 1.0) * 1.01;
        result = f64::sqrt(result + 1.0) * 1.01;
        result = f64::sqrt(result + 1.0) * 1.01;
        result = f64::sqrt(result + 1.0) * 1.01;
    }

    // Handle remaining iterations
    for _ in 0..remainder {
        result = f64::sqrt(result + 1.0) * 1.01;
    }

    result
}

fn test_complex_normal(iterations: i32) -> f64 {
    let mut result = 0.0;

    // Normal loop
    for _ in 0..iterations {
        result = f64::sqrt(result + 1.0) * 1.01;
    }

    result
}
