# Sticky

Sticky is a GTK4-based sticky note application written in Rust. It provides resizable, color-customizable floating notes with persistent storage using SQLite. Notes can contain editable text blocks, and the app saves note position, size, color, and contents automatically.

## Features

- Borderless sticky-note windows
- Persistent storage using SQLite (`notes.db` in user data directory)
- Add and remove notes
- Change note background color via system color picker
- Notes can remain always on top
- Resizable notes with saved size and position
- Editable text blocks inside notes
- Shortcut support: `Ctrl+N` to create a new note

## Architecture

- `src/main.rs` — Application bootstrap and startup logic
- `src/db.rs` — SQLite database layer for notes and text blocks
- `src/window.rs` — Sticky note window implementation and controls
- `src/canvas.rs` — Note canvas for rendering and managing text blocks
- `src/text_block.rs` — Editable text block widget
- `src/portals.rs` — Portal integration for color picker and shortcut setup
- `src/style.css` — Theme and styling rules for the note UI

## Installation

### Prerequisites

- Rust toolchain (recommended via `rustup`)
- GTK4 development libraries
- libadwaita development libraries
- SQLite development libraries

On Debian/Ubuntu, install dependencies with:

```bash
sudo apt install libgtk-4-dev libadwaita-1-dev libsqlite3-dev
```

On Fedora:

```bash
sudo dnf install gtk4-devel libadwaita-devel sqlite-devel
```

## Build and Run

From the project root:

```bash
cargo build
cargo run
```

For a release build:

```bash
cargo build --release
cargo run --release
```

## Data Storage

Sticky stores its data in the system user data directory returned by `glib::user_data_dir()` in a dedicated application folder.

Example path on Linux:

```bash
~/.local/share/<app-name>/notes.db
```

## Usage

- Launch the app to restore existing notes.
- Use `Ctrl+N` or the new note button to create a new sticky note.
- Double-click inside a note canvas to add a new text block.
- Drag text blocks to reposition them.
- Click the color button to choose a new note background color.
- Use the close button to delete a note.
- Toggle the pin button to mark a note always on top.

## Roadmap

Sticky is actively improving. See the full roadmap in [docs/ROADMAP.md](docs/ROADMAP.md).

### Planned enhancements

- Add a note title bar and improved drag handles
- Improve text block editing and resizing
- Add search, export/import, and backup support
- Add more theme options and visual polish

## Notes

- The app uses `ashpd` to integrate with the desktop portal for color selection and shortcuts.
- Note content and position are automatically persisted in the database.
- Deleting a note removes its corresponding database records.

## License

This repository does not currently include a license file.
