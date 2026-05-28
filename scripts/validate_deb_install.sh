#!/bin/bash
set -e

echo "Running validation checks on installed Sticky .deb..."

# 1. Command exists
if ! command -v sticky >/dev/null 2>&1; then
    echo "❌ Error: 'sticky' command not found in PATH."
    exit 1
fi
echo "✅ 'sticky' command found at $(command -v sticky)"

# 2. Desktop file presence
DESKTOP_FILE="/usr/share/applications/sticky.desktop"
if [ ! -f "$DESKTOP_FILE" ]; then
    echo "❌ Error: Desktop file not found at $DESKTOP_FILE"
    exit 1
fi
echo "✅ Desktop file found."

# 3. Desktop file contents
if ! grep -q "^Exec=sticky" "$DESKTOP_FILE"; then
    echo "❌ Error: Desktop file missing 'Exec=sticky'"
    exit 1
fi
if ! grep -q "^Icon=sticky" "$DESKTOP_FILE"; then
    echo "❌ Error: Desktop file missing 'Icon=sticky'"
    exit 1
fi
echo "✅ Desktop file contents valid."

# 4. Icon file presence
ICON_FILE=$(find /usr/share/icons/hicolor -iname "sticky.*" | head -n 1)
if [ -z "$ICON_FILE" ]; then
    echo "❌ Error: Icon file not found in /usr/share/icons/hicolor"
    exit 1
fi
echo "✅ Icon file found at $ICON_FILE"

# 5. dpkg checks
if ! dpkg -L sticky | grep -Ei -q "sticky$|desktop|icons"; then
    echo "❌ Error: dpkg does not list the expected files."
    exit 1
fi
echo "✅ dpkg package structure valid."

echo ""
echo "🎉 Validation passed! The .deb package is completely healthy and compliant."
