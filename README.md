# Sticky

Sticky is a powerful, modern GTK4-based desktop note-taking and whiteboard application written in Rust. It goes beyond simple sticky notes by providing an infinite whiteboard mode, Pomodoro timers, code snippets, file attachments, and AI-powered meeting summaries.

![Sticky Notes Showcase](https://via.placeholder.com/800x450.png?text=Showcase+Screenshot+-+Sticky+Notes+on+Desktop)
*(Note: Replace placeholder with an actual screenshot or GIF of the app running!)*

## ⚠️ Status

**Sticky is currently an experimental desktop productivity app.** Core sticky-note and whiteboard features are fully implemented and stable. AI meeting summaries require an OpenAI API key. 

*Note: Some native desktop-window behaviors, such as edge peeking and magnetic snapping, rely on X11 semantics and may be limited on modern Wayland compositors without explicit extension support.*

## ✨ Features

- **Floating Sticky Notes**: Borderless, resizable, and color-customizable notes that float on your desktop.
- **Infinite Whiteboard Mode**: Expand any note into a full-screen, infinite scrollable whiteboard with a dot-grid background.
- **Block-Based Canvas**: Double-click anywhere to add different types of blocks:
  - **Text & Handwriting**: Standard text with a beautiful cursive handwriting font.
  - **Checklists**: Native checkboxes with a satisfying strike-through animation.
  - **Code Snippets**: Monospaced font blocks for code and scripts.
  - **Pomodoro Timers**: Built-in 25-minute focus timers that alert you when done.
  - **File Attachments**: Drag and drop any file (images, PDFs, documents) to create an attachment block. Click to open in your default app.
- **Mind-Map Mode**: Click the link (🔗) button on one block and then another to draw beautiful, organic bezier curves between them on the whiteboard!
- **WYSIWYG Markdown Editing**: Type standard markdown (like `**bold**` or `*italic*`) and watch it instantly render as clean, stylized text the moment you click away.
- **Export Notes**: Instantly export any note's contents to a standard `.md` file with the native save dialog.
- **Templates & Themes**: Use the Command Palette to spawn predefined templates (Daily Planner, Meeting Notes, Bug Tracker, Kanban, LaTeX) or apply gorgeous UI themes (Pastel, Dark Glass, Terminal, etc.).
- **Data Portability (Backup & Restore)**: Built-in SQLite backup support. Export your entire database from the System Tray to a safe location, or import an existing backup to restore your notes.
- **AI Integration**:
  - **Meeting Transcripts**: Click the microphone to record audio (uses `arecord`).
  - **Summaries**: Automatically transcribed and summarized using OpenAI's Whisper and GPT APIs.
- **Offline TTS**: A "Read Aloud" button on every block uses native Linux TTS (`spd-say`) to read your notes offline.
- **System Tray Integration**: Runs quietly in the background. Access quick actions, search, and the Recycle Bin directly from your system tray.
- **Recycle Bin**: Accidentally delete a note? Don't worry! Deleted notes are sent to a soft-delete trash and can be instantly restored from the System Tray.
- **Global Spotlight Search**: Press `Ctrl+Shift+F` (or use the tray menu) to bring up a frosted-glass search bar to instantly find and focus any note.
- **Command Palette**: Press `Ctrl+K` to quickly insert checklists, code snippets, timers, or change the note background color without using the mouse.
- **ASMR Animations**: Satisfying hover physics, bouncy transitions, and a "peel-off" animation when trashing notes.
- **Always on Top**: Pin your notes to keep them visible over other windows.

---

## 🚀 Getting Started

### Prerequisites

- **Rust** toolchain (recommended via [rustup](https://rustup.rs/))
- **GTK4** and **libadwaita** development libraries
- **SQLite** development libraries
- Linux native tools: `arecord` (for mic), `spd-say` (for TTS)

**Debian / Ubuntu:**
```bash
sudo apt install libgtk-4-dev libadwaita-1-dev libsqlite3-dev speech-dispatcher alsa-utils
```

**Fedora:**
```bash
sudo dnf install gtk4-devel libadwaita-devel sqlite-devel speech-dispatcher alsa-utils
```

**Arch Linux:**
```bash
sudo pacman -S gtk4 libadwaita sqlite speech-dispatcher alsa-utils
```

---

## 📦 Installation

### Linux — `.deb` Installer (Recommended)

A proper Debian package (`.deb`) is provided. This installs the binary system-wide, adds a desktop entry to your app launcher, and auto-resolves all dependencies.

**Build the `.deb`:**
```bash
cargo install cargo-deb   # one-time setup
cargo deb
```

**Install it:**
```bash
sudo dpkg -i target/debian/sticky_0.1.0-1_amd64.deb
```

Or simply double-click the `.deb` file in your file manager — it will open in your Software Center / GDebi, just like installing any other app.

**Uninstall:**
```bash
sudo apt remove sticky
```

### Linux — Quick Install Script

If you prefer a user-local install (no `sudo` required):
```bash
./install.sh
```
This builds the release binary, copies it to `~/.local/bin/sticky`, and creates a `.desktop` entry in your app launcher.

### Windows — `.msi` Installer

A WiX manifest is included to generate a proper Windows `.msi` installer with Start Menu and Desktop shortcuts.

**Prerequisites:**
- [Rust](https://rustup.rs/) toolchain for Windows
- [WiX Toolset v3](https://wixtoolset.org/) installed and on your PATH
- GTK4 runtime libraries for Windows (via [gvsbuild](https://github.com/wingtk/gvsbuild) or [MSYS2](https://www.msys2.org/))

**Build the `.msi`:**
```powershell
cargo install cargo-wix   # one-time setup
cargo wix
```

The installer will be output to `target/wix/sticky-0.1.0-x86_64.msi`. Double-click it to install like any Windows application.

### Windows — Quick Install Script

For a lightweight install without WiX:
```powershell
.\install.ps1
```
This builds the release binary, copies it to `%LOCALAPPDATA%\Programs\Sticky`, and creates a Desktop shortcut.

---

## 📖 How to Use

1. **Creating Notes**: Launch the app or use the System Tray icon to create a "New Note".
2. **Adding Content**: Double-click anywhere inside the note window to add a text block.
3. **Block Types**: Use the icons in the header bar or the Command Palette to add specific blocks:
   - 📝 **Checklist** — Adds a to-do list block.
   - 💻 **Code** — Adds a monospace code snippet block.
   - ⏱️ **Timer** — Adds a Pomodoro focus timer.
   - 📎 **Files** — Drag and drop any file into the window to attach it.
4. **Organizing**: Drag blocks around by clicking and dragging them.
5. **Whiteboard Mode**: Click the `[ ]` (Expand) button in the top right. The note will expand to fill the screen and switch to a dot-grid infinite canvas. Un-maximize to return to sticky note mode.
6. **AI Summaries**:
   - Export `OPENAI_API_KEY` in your environment.
   - Click the microphone icon to start recording. Click again to stop. Wait a few seconds — an AI summary block will appear alongside automatically extracted actionable Checklists!
7. **Connecting Blocks (Mind-Map)**: Click the 🔗 icon on a block, then click the 🔗 icon on another block. A beautiful curved arrow will connect them in the background!
8. **Searching**: Press `Ctrl+Shift+F` (or use the tray menu) to search across all your saved notes.
9. **Command Palette**: Press `Ctrl+K` on any note to open the quick-action palette.
10. **Recycle Bin**: Trashed notes can be restored from the System Tray → "Restore Last Deleted". Use "Empty Trash" to permanently purge.

---

## ⌨️ Keyboard Shortcuts

| Shortcut | Action |
|---|---|
| `Ctrl+Shift+F` | Global Spotlight Search |
| `Ctrl+K` | Command Palette |
| `Ctrl+B` | Bold selected text |
| `Ctrl+I` | Italicize selected text |

---

## 📂 Data Storage

Everything is automatically saved to a local SQLite database:
- **Linux**: `~/.local/share/sticky/notes.db`
- **Windows**: `%APPDATA%\sticky\notes.db`

Deleting a note sends it to the Recycle Bin. It can be permanently purged using the "Empty Trash" option in the System Tray.

---

## 🛠 Architecture

| File | Purpose |
|---|---|
| `src/main.rs` | App bootstrap, system tray icon (`ksni`), global search, and DB init |
| `src/db.rs` | SQLite data access layer for notes, blocks, and mind-map links |
| `src/window.rs` | The sticky note window, header controls, and whiteboard toggles |
| `src/canvas.rs` | Drag-and-drop canvas, infinite dot-grid, and bezier arrow rendering |
| `src/text_block.rs` | Multi-purpose block widget (Text, Code, Timer, Files, Checklists) |
| `src/portals.rs` | Flatpak/XDG portal integration for autostart and color picking |
| `src/style.css` | GTK CSS for themes, ASMR animations, and layout |

---

## 📦 Project Structure

```
Sticky/
├── src/
│   ├── main.rs
│   ├── db.rs
│   ├── window.rs
│   ├── canvas.rs
│   ├── text_block.rs
│   ├── portals.rs
│   └── style.css
├── assets/
│   └── sticky.desktop        # Linux desktop entry
├── wix/
│   └── main.wxs              # Windows .msi installer manifest
├── docs/
│   └── ROADMAP.md             # Feature roadmap & pending work
├── install.sh                 # Linux quick-install script
├── install.ps1                # Windows quick-install script
├── Cargo.toml                 # Rust package config + deb metadata
└── README.md
```

---

## 🪪 License

MIT
