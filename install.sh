#!/bin/bash

# Cheetah language installer script

# Ensure we're in the right directory
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR"

# Build the project in release mode
echo "Building Cheetah in release mode..."
cargo build --release

# Create the installation directory if it doesn't exist
INSTALL_DIR="/usr/local/bin"
if [ ! -d "$INSTALL_DIR" ]; then
    echo "Creating installation directory $INSTALL_DIR..."
    sudo mkdir -p "$INSTALL_DIR"
fi

# Copy the binary to the installation directory
echo "Installing Cheetah to $INSTALL_DIR..."
sudo cp target/release/cheetah "$INSTALL_DIR/"

# Make it executable
sudo chmod +x "$INSTALL_DIR/cheetah"

echo "Installation complete!"
echo "You can now run Cheetah with: cheetah your_file.ch"
echo "Or start the REPL with: cheetah"
