#!/usr/bin/env bash
set -e

echo "🔨 Building collab CLI and server..."

# Build CLI
echo "Building CLI..."
cd collab-cli
cargo build --release
cd ..

# Build Server
echo "Building server..."
cd collab-server
cargo build --release
cd ..

echo "✓ Build complete!"
echo ""
echo "📦 Binaries located at:"
echo "  CLI:    collab-cli/target/release/collab"
echo "  Server: collab-server/target/release/collab-server"
echo ""
echo "To install system-wide (requires sudo):"
echo "  sudo cp collab-cli/target/release/collab /usr/local/bin/"
echo "  sudo cp collab-server/target/release/collab-server /usr/local/bin/"
echo ""
echo "To configure, create ~/.collab.toml:"
echo "  host = \"http://your-server:8000\""
echo "  instance = \"your-worker-name\""
echo "  recipients = [\"other-worker\"]"
echo ""
echo "Run 'collab config-path' to see the exact config file location."
