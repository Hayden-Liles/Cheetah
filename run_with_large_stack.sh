#!/bin/bash
# Run a Cheetah program with a larger stack size
# The default stack size is 8MB, let's increase it to 32MB
RUST_MIN_STACK=33554432 cargo run --bin cheetah -- -j "$@"
