#!/usr/bin/env bash
set -euo pipefail

# 1) Build the Cheetah CLI in release mode
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR"

echo "🔨 Building Cheetah CLI…"
cargo build --release

# 2) Install the CLI binary
BIN_PATH="/usr/local/bin/cheetah"
echo "📦 Installing CLI to $BIN_PATH…"
sudo install -m 0755 target/release/cheetah "$BIN_PATH"

# 3) Install the runtime library for AOT linking
RUNTIME_DIR="/usr/local/lib/cheetah"
echo "📦 Installing runtime to $RUNTIME_DIR…"
sudo mkdir -p "$RUNTIME_DIR"
sudo install -m 0644 target/release/libcheetah.a "$RUNTIME_DIR/libcheetah.a"

cat <<EOF

✅ Done!
• You now have a single 'cheetah' binary on your PATH.
• To run a .ch file in one step:
    cheetah ./your_script.ch
  – the CLI will compile it (into ./.cheetah_build/) then exec() into the native binary.
• For REPL, lex, parse, etc., just use:
    cheetah <subcommand> [args…]
EOF
