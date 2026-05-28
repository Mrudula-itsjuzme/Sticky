Write-Host "🔨 Building Sticky release for Windows..."
cargo build --release

$BinDir = "$env:LOCALAPPDATA\Programs\Sticky"
if (!(Test-Path -Path $BinDir)) {
    New-Item -ItemType Directory -Path $BinDir | Out-Null
}

Write-Host "📦 Installing binary to $BinDir..."
Copy-Item "target\release\sticky.exe" -Destination "$BinDir\sticky.exe" -Force

$DesktopPath = [Environment]::GetFolderPath("Desktop")
$ShortcutPath = "$DesktopPath\Sticky.lnk"

Write-Host "🖥️ Creating Desktop Shortcut..."
$WshShell = New-Object -comObject WScript.Shell
$Shortcut = $WshShell.CreateShortcut($ShortcutPath)
$Shortcut.TargetPath = "$BinDir\sticky.exe"
$Shortcut.WorkingDirectory = $BinDir
$Shortcut.IconLocation = "$BinDir\sticky.exe"
$Shortcut.Save()

Write-Host "✅ Installation Complete! A shortcut has been placed on your Desktop."
