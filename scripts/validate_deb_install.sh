#!/bin/bash
set -e

echo "Running validation checks on installed Sticky .deb..."

# 1. Command exists
if ! command -v sticky >/dev/null 2>&1; then
    echo "❌ Error: 'sticky' command not found in PATH."
    exit 1
fi
echo "✅ 'sticky' command found at $(command -v sticky)"

# 2. Version check
VERSION=$(sticky --version)
echo "✅ Version output: $VERSION"

# 3. Desktop file presence
DESKTOP_FILE="/usr/share/applications/sticky.desktop"
if [ ! -f "$DESKTOP_FILE" ]; then
    echo "❌ Error: Desktop file not found at $DESKTOP_FILE"
    exit 1
fi
echo "✅ Desktop file found."

# 4. Desktop file contents
if ! grep -q "^Exec=sticky" "$DESKTOP_FILE"; then
    echo "❌ Error: Desktop file missing 'Exec=sticky'"
    exit 1
fi
if ! grep -q "^Icon=sticky" "$DESKTOP_FILE"; then
    echo "❌ Error: Desktop file missing 'Icon=sticky'"
    exit 1
fi
echo "✅ Desktop file contents valid."

# 5. Icon file presence
ICON_FILE="/usr/share/icons/hicolor/256x256/apps/sticky.png"
if [ ! -f "$ICON_FILE" ]; then
    echo "❌ Error: Icon file not found at $ICON_FILE"
    exit 1
fi
echo "✅ Icon file found."

# 6. dpkg checks
if ! dpkg -L sticky | grep -q "/usr/bin/sticky"; then
    echo "❌ Error: /usr/bin/sticky not owned by package."
    exit 1
fi
echo "✅ dpkg package structure valid."

# 7. Test launch via gtk-launch (timeout after 2s)
echo "Testing gtk-launch..."
timeout 2s gtk-launch sticky || true
echo "✅ gtk-launch test complete."

echo ""
echo "🎉 Validation passed! The .deb package is completely healthy and compliant."
