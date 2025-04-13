#!/bin/bash
# Run a Cheetah program with direct output mode enabled
export CHEETAH_FORCE_DIRECT_OUTPUT=1
cargo run --bin cheetah -- -j "$@"
