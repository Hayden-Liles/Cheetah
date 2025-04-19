#!/usr/bin/env bash
set -euo pipefail

# 1) Build your CLI & copy it + the runtime lib where the wrapper can see it
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR"

echo "🔨 Building Cheetah CLI…"
cargo build --release

CLI_LIB_DIR="/usr/local/lib/cheetah"
echo "📦 Installing to $CLI_LIB_DIR…"
sudo mkdir -p "$CLI_LIB_DIR/target/release"

# copy the cheetah CLI binary
sudo cp target/release/cheetah "$CLI_LIB_DIR/cheetah-cli"
sudo chmod +x "$CLI_LIB_DIR/cheetah-cli"

# copy the static runtime archive so emit_to_aot can link it
sudo cp target/release/libcheetah.a "$CLI_LIB_DIR/target/release/"
sudo chmod 644 "$CLI_LIB_DIR/target/release/libcheetah.a"

# 2) Drop a wrapper on your PATH
BIN_DIR="/usr/local/bin"
echo "🚀 Installing wrapper to $BIN_DIR/cheetah…"

sudo tee "$BIN_DIR/cheetah" > /dev/null << 'EOF'
#!/usr/bin/env bash
set -euo pipefail

CHEETAH_CLI="/usr/local/lib/cheetah/cheetah-cli"
RUNTIME_DIR="/usr/local/lib/cheetah"
BUILD_DIR=".cheetah_build"

# Ensure build directory
mkdir -p "$BUILD_DIR"

# Compile $1.ch → $BUILD_DIR/<basename>
build_file() {
  local src="$1"; shift
  local abs_src base
  abs_src="$(realpath "$src")"
  base="$(basename "${src%.*}")"
  echo "⏳ Building $src → $BUILD_DIR/$base"
  pushd "$BUILD_DIR" >/dev/null
    env CARGO_MANIFEST_DIR="$RUNTIME_DIR" \
        "$CHEETAH_CLI" compile --object -o "$base" "$abs_src"
  popd >/dev/null
  sudo chmod +x "$BUILD_DIR/$base"
  echo "✅ Built $BUILD_DIR/$base"
}

# Run (exec) $BUILD_DIR/$base with any args
run_file() {
  local src="$1"; shift
  local base="$(basename "${src%.*}")"
  local exe="$BUILD_DIR/$base"
  if [[ ! -x "$exe" ]]; then
    build_file "$src"
  fi
  echo "▶️  Exec’ing $exe $*"
  exec "$exe" "$@"
}

case "$1" in
  build)
    [[ -z "${2-}" ]] && { echo "Usage: cheetah build <file.ch>"; exit 1; }
    build_file "$2"
    ;;

  run)
    [[ -z "${2-}" ]] && { echo "Usage: cheetah run <file.ch> [args…]"; exit 1; }
    run_file "$2" "${@:3}"
    ;;

  *.ch)
    # one‐shot: cheetah foo.ch [args…]
    run_file "$1" "${@:2}"
    ;;

  *)
    # forward all other verbs to the real CLI
    exec "$CHEETAH_CLI" "$@"
    ;;
esac
EOF

sudo chmod +x "$BIN_DIR/cheetah"

echo
echo "✅ Done! You can now:"
echo "   • cheetah build <file.ch>       → produce AOT binary in ./.cheetah_build/"
echo "   • cheetah run <file.ch> [args]  → exec the AOT binary directly"
echo "   • cheetah <file.ch> [args]      → build & exec in one step"
echo "   • cheetah <other-subcmd>…       → repl, lex, parse, etc."
