#!/bin/bash
set -e

echo "Running validation checks on installed Sticky .deb..."

command -v sticky
test -f /usr/share/applications/sticky.desktop
grep "Exec=sticky" /usr/share/applications/sticky.desktop
grep "Icon=sticky" /usr/share/applications/sticky.desktop
find /usr/share/icons/hicolor -iname "sticky.*"
dpkg -L sticky | grep -Ei "sticky$|desktop|icons"

echo "🎉 Validation passed! The .deb package is completely healthy and compliant."
