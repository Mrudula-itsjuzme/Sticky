# Sticky - Roadmap & Pending Work

This document outlines the pending work and recommended features to transition the application from its current state into a truly polished, world-class productivity environment.

## 🚧 Pending Work (Currently Unfinished)

These are features that were discussed or planned but have not yet been fully implemented:

- **Edge Peeking / Auto-Hide** (Wayland Blocked)
  - *Current State*: Notes can be dragged freely.
  - *Pending*: When a note is dragged to the edge of the monitor, it should auto-hide into a small tab. Hovering over the tab should slide the note back out. (Note: Requires Window Manager extensions like KWin scripts on Wayland).

- **Window Snapping (Magnetic Edges)** (Wayland Blocked)
  - *Current State*: Windows float freely.
  - *Pending*: Make windows "snap" to each other or align to a grid when dragged close to another note's edge, creating a tidy mosaic on the desktop. (Note: Also requires WM extensions on modern Wayland).

## ✅ Recently Completed
- **Version History / Recycle Bin**: Soft deletion logic with System Tray integration for restoration.
- **WYSIWYG Markdown Editing**: Focus-based dynamic Pango tag injection to hide syntax when not editing.
- **Mind-Map Connecting Arrows**: SQLite-backed block linking with native Cairo cubic bezier drawing.
- **Command Palette**: `Ctrl+K` integration for rapid keyboard-driven productivity.
- **Action Item Extraction**: AI integration automatically parsing and creating checklist blocks.

## 💡 New Feature Recommendations

To further elevate the ASMR, spatial computing, and AI-centric goals of the application:

### 1. Offline Local AI Mode
Right now, the AI Meeting Summary relies on the OpenAI API via `OPENAI_API_KEY`. 
- **Recommendation**: Integrate **Whisper.cpp** or **Ollama** natively (or via local API calls) to process audio and text entirely on-device, offering extreme privacy for sensitive work.

### 2. Live Collaboration (Multiplayer)
- **Recommendation**: Since the backend is just SQLite, an optional sync server (using websockets or something like Yjs/CRDTs) could allow multiple users to edit the same whiteboard in real-time.

### 3. Canvas "Portals" (Note Linking)
- **Recommendation**: Allow a block in Note A to be a clickable "Portal" to Note B. When double-clicked, the window seamlessly morphs and loads the contents of Note B.

### 4. Advanced Block Types
- **Kanban Boards**: A block that acts as a container for other sub-blocks, rendering them in columns.
- **Embedded Web Views**: Using WebKitGTK to embed a live webpage (like a YouTube video or Figma file) directly on the whiteboard.
- **Math / LaTeX Blocks**: A block that renders equations beautifully via MathML or an embedded renderer.

### 5. Context-Aware AI Command Palette
- **Recommendation**: Upgrade the `Ctrl+K` palette so you can type things like: `> "Make this note sound more professional"` or `> "Translate to Spanish"`. The app reads the current canvas text, pings the AI, and replaces the text blocks with the result.

### 6. Semantic Global Search
- **Recommendation**: The current `Ctrl+Shift+F` Spotlight search uses standard SQLite `LIKE` queries. By generating embeddings for note text (using a local model like `all-MiniLM-L6-v2`) and storing them in SQLite (via `sqlite-vec` or `vss`), the search bar could understand the *meaning* of your query, not just exact keywords.
