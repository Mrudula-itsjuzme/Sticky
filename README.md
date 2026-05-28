# Sticky

**Sticky** is a modern Rust + GTK4 desktop productivity app that combines sticky notes, an infinite whiteboard, Pomodoro timers, code snippets, file attachments, and AI-powered meeting summaries.

It is built for students, developers, writers, and productivity-focused users who want a local-first desktop workspace instead of another cloud-first notes app.

[![Latest Release](https://img.shields.io/github/v/release/Mrudula-itsjuzme/Sticky?style=for-the-badge)](https://github.com/Mrudula-itsjuzme/Sticky/releases/latest)
[![Built with Rust](https://img.shields.io/badge/built%20with-Rust-orange?style=for-the-badge)](https://www.rust-lang.org/)
[![GTK4](https://img.shields.io/badge/UI-GTK4-blue?style=for-the-badge)](https://gtk-rs.org/)
[![Platform](https://img.shields.io/badge/platform-Linux%20%7C%20Windows-lightgrey?style=for-the-badge)](https://github.com/Mrudula-itsjuzme/Sticky/releases/latest)

---

## Status

Sticky is currently an experimental desktop productivity app.

Core sticky-note and whiteboard features are implemented. AI meeting summaries require an OpenAI API key. Some native desktop-window behaviors, such as edge peeking and magnetic snapping, may vary depending on X11 or Wayland compositor support.

---

## What is Sticky?

Sticky is a local-first desktop notes app written in Rust using GTK4. It goes beyond basic sticky notes by combining floating notes, an infinite whiteboard, block-based editing, Pomodoro timers, code snippets, file attachments, search, and AI-assisted meeting summaries.

It can be used as a:

- Sticky notes manager
- Infinite whiteboard
- Pomodoro focus tool
- Code snippet organizer
- File attachment board
- Meeting notes and summary assistant
- Visual workspace for ideas, tasks, and study sessions

---

## Features

### Floating sticky notes

Create borderless, resizable, color-customizable sticky notes that stay on your desktop.

### Infinite whiteboard mode

Expand a note into a full-screen whiteboard with an infinite dot-grid canvas for visual planning, brainstorming, and idea mapping.

### Block-based canvas

Double-click anywhere on the canvas to add different types of blocks:

- Text blocks
- Checklist blocks
- Code snippet blocks
- Pomodoro timer blocks
- File attachment blocks

### Mind-map links

Connect blocks visually using curved links to build lightweight mind maps and idea flows.

### WYSIWYG Markdown editing

Write simple Markdown-style text and let Sticky render it into clean formatted content.

### Export notes

Export note contents to a standard Markdown file using the native save dialog.

### Templates and themes

Use the command palette to create templates such as daily planners, meeting notes, bug trackers, Kanban boards, and LaTeX blocks. Apply visual themes such as Pastel, Dark Glass, Terminal, and more.

### Backup and restore

Export the local SQLite database as a backup, or import an existing backup to restore your notes.

### AI meeting summaries

Record meeting audio, transcribe it, and generate AI-powered summaries and action items using OpenAI APIs.

> AI features require an `OPENAI_API_KEY`.

### Offline text-to-speech

Use native Linux text-to-speech support to read note content aloud.

### System tray integration

Run Sticky quietly in the background and access quick actions, search, and restore options from the tray.

### Recycle bin

Deleted notes are soft-deleted first, so they can be restored before being permanently removed.

### Global search

Search across saved notes quickly using the global spotlight-style search.

### Command palette

Use the command palette to quickly add blocks, change colors, trigger actions, or create templates without relying on the mouse.

---

## Download

Download the latest version from the [Releases](https://github.com/Mrudula-itsjuzme/Sticky/releases/latest) page.

Release builds may include Linux (`.deb` and `.AppImage`) and Windows (`.msi`) assets depending on the version.

> Sticky is a native desktop application. Do not use GitHub Packages, npm, or Docker to install it. Download the release asset for your platform.

---

## Getting Started

### Prerequisites

You need the Rust toolchain and GTK-related development libraries.

Install Rust using [rustup](https://rustup.rs/).

### Debian / Ubuntu

```bash
sudo apt install libgtk-4-dev libadwaita-1-dev libsqlite3-dev speech-dispatcher alsa-utils
```

### Fedora

```bash
sudo dnf install gtk4-devel libadwaita-devel sqlite-devel speech-dispatcher alsa-utils
```

### Arch Linux

```bash
sudo pacman -S gtk4 libadwaita sqlite speech-dispatcher alsa-utils
```

---

## Installation

Download the latest release from the **[Releases Page](https://github.com/Mrudula-itsjuzme/Sticky/releases/latest)**.

Each release includes separate downloads for each platform:

| File | Platform | How to install |
|---|---|---|
| `sticky_X.X.X_amd64.deb` | Debian / Ubuntu | Double-click or `sudo dpkg -i` |
| `sticky.AppImage` | Any Linux distro | `chmod +x` and run — no install needed |
| `sticky-X.X.X.msi` | Windows | Double-click the installer wizard |

### Linux — Install via `.deb`
```bash
sudo dpkg -i sticky_0.1.0-1_amd64.deb
```
Launch **Sticky** from your application menu.

### Linux — Run the `.AppImage` (portable, no install)
```bash
chmod +x sticky.AppImage
./sticky.AppImage
```
The first time you run the AppImage, it will automatically register itself in your Application Menu.

### Windows

> **Note:** The Windows `.msi` installer must be built on a Windows machine. A pre-built `.msi` may not be available in every release.

If a `sticky-X.X.X.msi` file is included in the release:
1. Download and double-click the `.msi` installer.
2. Follow the setup wizard.
3. Launch **Sticky** from your Start Menu or Desktop shortcut.

If no `.msi` is provided, see the [Building from Source](#for-developers-building-from-source) section below to compile on Windows.

---

## 🛠️ For Developers: Building from Source

If you want to compile this app yourself or contribute to the project, follow these steps:

### 1. Building the `.deb` file on Linux
```bash
cargo install cargo-deb
cargo deb
```

The generated package will be available at:

```text
target/debian/sticky_0.1.0-1_amd64.deb
```

### 2. Building the `.AppImage` on Linux
```bash
cargo install cargo-appimage
# Requires 'appimagetool' installed on your system
cargo appimage
```

The generated AppImage will be at `target/appimage/sticky.AppImage`.

### 3. Building the `.msi` file on Windows

> This step **must** be done on a Windows machine. Cross-compiling GTK4 apps for Windows from Linux is not supported.

**Prerequisites:** [Rust](https://rustup.rs/), [WiX Toolset v3](https://wixtoolset.org/), and GTK4 runtime libraries (via [gvsbuild](https://github.com/wingtk/gvsbuild) or [MSYS2](https://www.msys2.org/)).

```powershell
cargo install cargo-wix
cargo wix
```

The generated installer will be at:

```text
target\wix\sticky-0.1.0-x86_64.msi
```

---

## How to Use

1. Launch Sticky.
2. Create a new note from the app or system tray.
3. Double-click inside a note to add a block.
4. Use the command palette to add checklists, code blocks, timers, templates, or themes.
5. Expand a note into whiteboard mode for a larger visual workspace.
6. Drag blocks around to organize ideas.
7. Connect blocks using mind-map links.
8. Use global search to find saved notes.
9. Restore deleted notes from the recycle bin if needed.

---

## AI Summaries

Sticky can record meeting audio and generate summaries using OpenAI APIs.

To enable AI features, set your API key:

```bash
export OPENAI_API_KEY="your_api_key_here"
```

Then use the microphone button in the app to start and stop recording.

> Audio transcription and summaries depend on OpenAI API access. Do not use this feature for private or sensitive meetings unless you understand where your data is being sent.

---

## Keyboard Shortcuts

| Shortcut | Action |
|---|---|
| `Ctrl+Shift+F` | Global search |
| `Ctrl+K` | Command palette |
| `Ctrl+B` | Bold selected text |
| `Ctrl+I` | Italicize selected text |

---

## Data Storage

Sticky stores notes locally in a SQLite database.

| Platform | Storage path |
|---|---|
| Linux | `~/.local/share/sticky/notes.db` |
| Windows | `%APPDATA%\sticky\notes.db` |

Deleted notes are moved to the recycle bin before permanent deletion.

---

## Architecture

| File | Purpose |
|---|---|
| `src/main.rs` | App startup, system tray, global search, and database initialization |
| `src/db.rs` | SQLite data layer for notes, blocks, and links |
| `src/window.rs` | Sticky note window, controls, whiteboard toggles, and AI flow |
| `src/canvas.rs` | Canvas rendering, drag-and-drop behavior, and mind-map links |
| `src/text_block.rs` | Text, checklist, code, timer, and file block widgets |
| `src/portals.rs` | Flatpak/XDG portal integration |
| `src/style.css` | GTK styling, animations, and layout |

---

## Project Structure

```text
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
│   └── sticky.desktop
├── wix/
│   └── main.wxs
├── docs/
│   └── ROADMAP.md
├── install.sh
├── install.ps1
├── Cargo.toml
└── README.md
```

---

## Roadmap

Planned improvements include:

- Real screenshots and demo GIFs
- More export options
- Better search filters
- More note templates
- More theme presets
- Improved Windows packaging
- Local AI support
- Semantic search

See [`docs/ROADMAP.md`](docs/ROADMAP.md) for more details.

---

## Who is this for?

Sticky is designed for:

- Students organizing lectures, assignments, and study sessions
- Developers saving snippets, tasks, and project notes
- Writers mapping ideas visually
- Productivity users who prefer local-first desktop tools
- Anyone who wants sticky notes and a whiteboard in one app

---

## Tech Stack

- Rust
- GTK4
- gtk-rs
- SQLite
- libadwaita
- OpenAI APIs
- WiX for Windows installer generation
- cargo-deb for Debian package generation

---

## License

MIT
