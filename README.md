# Sticky / Antigrav

Sticky (Antigrav) is a powerful, modern GTK4-based desktop note-taking and whiteboard application written in Rust. It goes beyond simple sticky notes by providing an infinite whiteboard mode, Pomodoro timers, code snippets, file attachments, and AI-powered meeting summaries. 

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

## 🚀 Getting Started

### Prerequisites

- Rust toolchain (recommended via `rustup`)
- GTK4 and libadwaita development libraries
- SQLite development libraries
- Linux native tools: `arecord` (for mic), `spd-say` (for TTS)

On Debian/Ubuntu:
```bash
sudo apt install libgtk-4-dev libadwaita-1-dev libsqlite3-dev speech-dispatcher alsa-utils
```

### Installation

```bash
cargo build --release
cargo run --release
```

## 📖 How to Use

1. **Creating Notes**: Launch the app or use the System Tray icon to create a "New Note".
2. **Adding Content**: Double-click anywhere inside the note window to add a text block.
3. **Block Types**: Use the icons in the header bar to add specific blocks:
   - 📝 **Checklist**: Adds a to-do list block.
   - 💻 **Code**: Adds a monospace code snippet block.
   - ⏱️ **Timer**: Adds a Pomodoro focus timer.
   - 📎 **Files**: Drag and drop any file into the window to attach it.
4. **Organizing**: Drag blocks around by clicking and dragging them.
5. **Whiteboard Mode**: Click the `[ ]` (Expand) button in the top right. The note will expand to fill the screen and switch to a dot-grid infinite canvas. Un-maximize to return to sticky note mode.
6. **AI Summaries**: 
   - Ensure `OPENAI_API_KEY` is exported in your environment.
   - Click the microphone icon to start recording. Click again to stop. Wait a few seconds, and an AI summary block will appear alongside automatically extracted actionable Checklists!
7. **Connecting Blocks (Mind-Map)**: Click the 🔗 icon on a block, then click the 🔗 icon on another block. A beautiful curved arrow will connect them in the background!
8. **Searching**: Press `Ctrl+Shift+F` (or use the tray menu) to search across all your saved notes.
9. **Command Palette**: Press `Ctrl+K` on any note to open the quick-action palette.
10. **Recycle Bin**: Trashed notes can be restored by right-clicking the System Tray icon and selecting "Restore Last Deleted".

## 📂 Data Storage

Everything is automatically saved to a local SQLite database in your user data directory (e.g., `~/.local/share/antigrav/notes.db`). Deleting a note sends it to the Recycle Bin, and it can be permanently purged using the "Empty Trash" option in the System Tray.

## 🛠 Architecture

- `src/main.rs` — App bootstrap, tray icon (`ksni`), global search, and DB init.
- `src/db.rs` — SQLite data access layer for notes and blocks.
- `src/window.rs` — The sticky note window, header controls, and whiteboard toggles.
- `src/canvas.rs` — The drag-and-drop canvas and infinite scroll dot-grid rendering.
- `src/text_block.rs` — The multi-purpose block widget (Text, Code, Timer, Files, Checklists).
- `src/portals.rs` — Flatpak/XDG portal integration for autostart and color picking.
- `src/style.css` — GTK CSS for themes, ASMR animations, and layout.
