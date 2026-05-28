# Sticky — Feature Management & Architecture Guide

This document is intended for maintainers and developers to understand the internal workings of every core feature within **Sticky**.

---

## 1. Floating Sticky Notes
**How it works:** 
- The application spawns borderless GTK4 Windows (`adw::ApplicationWindow`) utilizing Wayland/X11 protocols. 
- Window decorations are removed by setting `.property("decorated", false)`. 
- Dragging is implemented manually using a `gtk::GestureClick` controller bound to the window header. When `GestureClick::connect_pressed` fires, it grabs the pointer device and delegates movement to the window manager using `gdk::Toplevel::begin_move`.
- **File:** `src/window.rs`

## 2. Infinite Whiteboard Mode
**How it works:** 
- Each note contains a `Canvas` widget (`gtk::Fixed`) that holds text blocks.
- When the expand `[ ]` button is clicked, the window calls `.maximize()` and applies a CSS class `.whiteboard-mode`. 
- Crucially, the `Canvas` is wrapped dynamically inside a `gtk::ScrolledWindow` at runtime, enabling infinite scrolling across an expansive dot-grid background drawn via Cairo.
- **File:** `src/window.rs`, `src/canvas.rs`

## 3. WYSIWYG Markdown Editing
**How it works:**
- Instead of using a split-pane "edit/preview" view, Markdown syntax is hidden dynamically.
- `TextBlock` binds to the `gtk::EventControllerFocus`.
- When the user clicks *into* a block (focus acquired), it strips GTK/Pango HTML tags and displays the raw markdown (e.g., `**bold**`).
- When the user clicks *away* (focus lost), a custom parser runs over the `GtkTextBuffer`, applies hidden tags (via `Pango::AttrList`), and renders the markdown visually (making the `**` invisible while bolding the word).
- **File:** `src/text_block.rs`

## 4. Mind-Map Connections (Bezier Arrows)
**How it works:**
- Handled at the database layer via a many-to-many table `block_links (source_id, target_id)`.
- When a user clicks the 🔗 icon on two different blocks, `Db::link_blocks()` is called.
- During the GTK drawing loop, `Canvas::snapshot()` intercepts the Cairo context. It iterates through all linked blocks, calculates their center points, and draws a cubic Bezier curve bridging them.
- **File:** `src/canvas.rs`, `src/db.rs`

## 5. AI Meeting Summaries
**How it works:**
- Audio capture is natively handled via `arecord` spawned as a child process. The file is temporarily stored in `~/.local/share/sticky/recording.wav` (avoiding `/tmp/` pollution).
- Upon stopping the recording, a background `tokio::spawn` task reads the environment for `OPENAI_API_KEY`.
- It performs a two-step API call using `reqwest`:
  1. **Transcription:** Sent to OpenAI Whisper API.
  2. **Summarization:** The transcript is forwarded to GPT-4o-mini, specifically instructed to return a JSON object containing `summary` and `action_items`.
- The async task then safely dispatches UI creation commands back to the main GTK thread to spawn new text blocks containing the results.
- **File:** `src/window.rs`

## 6. Global Spotlight Search
**How it works:**
- Triggered by `Ctrl+Shift+F` (or tray menu), bound globally via `gio::SimpleAction`.
- The app spawns a modal `SearchEntry` overlay.
- As the user types, `Db::search_blocks()` is invoked on every keystroke, performing a full-text `LIKE %query%` SQL search.
- When a result is clicked, the app loops over all active `adw::ApplicationWindow` instances, matches the DB ID, calls `.present()` to bring the window to the front, and briefly triggers a CSS pulse animation (`.highlight`).
- **File:** `src/main.rs`, `src/db.rs`

## 7. Soft-Delete Recycle Bin
**How it works:**
- Sticky does not actually delete data when the Trash icon is clicked.
- Instead, `Db::delete_note()` updates the `deleted` column in SQLite to `1`.
- The System Tray process (`ksni`) reads this column. If the user selects "Restore", the flag is flipped back to `0` and the note is re-rendered.
- "Empty Trash" executes a permanent SQL `DELETE CASCADE`, wiping the blocks and notes entirely.
- **File:** `src/db.rs`, `src/main.rs`

## 8. Command Palette, Templates & Themes
**How it works:**
- Bound to `Ctrl+K` on the note header via `gtk::EventControllerKey`.
- Spawns a floating `gtk::Popover` anchored to the header.
- Themes apply CSS dynamically using `gtk::CssProvider::load_from_data` injected with the selected hex color.
- Templates instantly dispatch hard-coded markdown strings to `Canvas::create_block_with_content()`.
- **File:** `src/window.rs`

## 9. Backup and Restore
**How it works:**
- Leveraging SQLite makes this trivial.
- Triggered from the `ksni` system tray loop, a `gtk::FileDialog` is spawned.
- **Export:** Uses `std::fs::copy` to duplicate `~/.local/share/sticky/notes.db` to the user's selected path.
- **Import:** Uses `std::fs::copy` to overwrite the live `notes.db` from a user-provided file.
- **File:** `src/main.rs`
