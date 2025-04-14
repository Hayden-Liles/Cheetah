#!/bin/bash

# Run the benchmark with loop unrolling enabled
echo "Running benchmark with loop unrolling enabled..."
cargo run --release --bin cheetah -- run benchmarks/loop_unrolling_simple.ch

# Temporarily disable loop unrolling by modifying the threshold
echo "Temporarily disabling loop unrolling..."
sed -i 's/const UNROLL_THRESHOLD: u64 = 8;/const UNROLL_THRESHOLD: u64 = 0;/' src/compiler/loop_optimizer.rs
sed -i 's/const PARTIAL_UNROLL_FACTOR: u64 = 4;/const PARTIAL_UNROLL_FACTOR: u64 = 1;/' src/compiler/loop_optimizer.rs

# Run the benchmark with loop unrolling disabled
echo "Running benchmark with loop unrolling disabled..."
cargo run --release --bin cheetah -- run benchmarks/loop_unrolling_simple.ch

# Restore the original thresholds
echo "Restoring original loop unrolling settings..."
sed -i 's/const UNROLL_THRESHOLD: u64 = 0;/const UNROLL_THRESHOLD: u64 = 8;/' src/compiler/loop_optimizer.rs
sed -i 's/const PARTIAL_UNROLL_FACTOR: u64 = 1;/const PARTIAL_UNROLL_FACTOR: u64 = 4;/' src/compiler/loop_optimizer.rs

echo "Benchmark complete!"
