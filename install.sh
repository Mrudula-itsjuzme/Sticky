#!/bin/bash
set -e

echo "🔨 Building Sticky release (this may take a moment)..."
cargo build --release

echo "📦 Installing binary to ~/.local/bin..."
mkdir -p ~/.local/bin
cp target/release/sticky ~/.local/bin/sticky

echo "🖥️ Creating Desktop Entry..."
mkdir -p ~/.local/share/applications
cat << 'DESKTOP' > ~/.local/share/applications/sticky.desktop
[Desktop Entry]
Name=Sticky
Comment=Modern Floating Sticky Notes and Whiteboard
Exec=sticky
Icon=accessories-text-editor
Terminal=false
Type=Application
Categories=Utility;Office;
DESKTOP

# Update desktop database
update-desktop-database ~/.local/share/applications 2>/dev/null || true

echo "✅ Installation Complete! You can now launch 'Sticky' from your application menu or run 'sticky' in your terminal."
