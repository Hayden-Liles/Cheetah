#!/bin/bash

# Run the benchmark with loop unrolling enabled
echo "Running benchmark with loop unrolling enabled..."
time cargo run --release --bin cheetah -- run examples/loop_unrolling_test.ch 2> unrolled_output.log
echo "Output from Cheetah program:"
cat unrolled_output.log | grep -v "\[LOOP UNROLL\]"
echo "\nLoop unrolling debug messages:"
cat unrolled_output.log | grep "\[LOOP UNROLL\]"

# Temporarily disable loop unrolling by modifying the threshold
echo "Temporarily disabling loop unrolling..."
sed -i 's/const UNROLL_THRESHOLD: u64 = 8;/const UNROLL_THRESHOLD: u64 = 0;/' src/compiler/loop_optimizer.rs
sed -i 's/const PARTIAL_UNROLL_FACTOR: u64 = 4;/const PARTIAL_UNROLL_FACTOR: u64 = 1;/' src/compiler/loop_optimizer.rs

# Run the benchmark with loop unrolling disabled
echo "Running benchmark with loop unrolling disabled..."
time cargo run --release --bin cheetah -- run examples/loop_unrolling_test.ch 2> normal_output.log
echo "Output from Cheetah program:"
cat normal_output.log | grep -v "\[LOOP UNROLL\]"
echo "\nLoop unrolling debug messages:"
cat normal_output.log | grep "\[LOOP UNROLL\]"

# Restore the original thresholds
echo "Restoring original loop unrolling settings..."
sed -i 's/const UNROLL_THRESHOLD: u64 = 0;/const UNROLL_THRESHOLD: u64 = 8;/' src/compiler/loop_optimizer.rs
sed -i 's/const PARTIAL_UNROLL_FACTOR: u64 = 1;/const PARTIAL_UNROLL_FACTOR: u64 = 4;/' src/compiler/loop_optimizer.rs

echo "Benchmark complete!"
