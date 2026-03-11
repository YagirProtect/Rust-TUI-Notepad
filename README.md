# Rust-TUI-Notepad

A terminal text editor written in Rust on top of `crossterm`.

This project is primarily a practical playground to learn:
- building a custom TUI layout and render pipeline
- editing text with selections, history, and search
- working with system clipboard, file dialogs, and recent files
- keeping a terminal app usable across different terminal hosts

Some features helped by OpenAI Codex.

---

## Features

### Editor

- Multi-line text editing
- Keyboard navigation with arrows, `Home`, `End`, `PgUp`, `PgDn`
- Mouse cursor placement
- Mouse text selection with auto-scroll on viewport edges
- Vertical mouse wheel scrolling
- Horizontal scrolling with `Alt + wheel`
- Undo / redo
- Real system clipboard support
- Clickable links inside the document

### File management

- Open file through native file dialog
- Save / Save As
- New virtual document with reserved path in app data
- Recent files panel on the left
- Current file highlight in recent files
- Dirty state marker (`*`) for unsaved changes
- Broken recent-file entries are marked with `x` and removed on click
- "Open in Explorer" for the current file
- Unsaved-changes confirmation before switching files or exiting

### Search and replace

- Side popup for `Find`
- Side popup for `Replace`
- Multi-line search query
- Match count and current match index
- `Next` / `Prev` navigation
- Replace current match
- Replace all matches
- Match highlighting inside the visible viewport
- Selected text is pushed into `Find` when opened

### Syntax highlighting

- Toggleable from `View -> Highlight Keywords`
- Generic language-agnostic keyword highlighting
- Separate colors for:
  - keywords
  - control-flow keywords
  - collection / vector-like types
  - numbers
  - strings
  - brackets
  - comments
- Supported comments:
  - `// ...`
  - `/* ... */`
  - `<!-- ... -->`

### Configurable shortcuts

- Hotkeys are stored in config as string bindings
- Users can remap shortcuts without recompiling
- Multiple shortcuts can be assigned to the same action

---

## Quick start

### Run from source

```bash
cargo run
```

### Windows Terminal helper

If you run inside Windows Terminal and want a cleaner `Ctrl+V` experience:

```bat
run_notepad_wt.cmd
```

What it does:
- temporarily unbinds `Ctrl+V` in Windows Terminal
- temporarily applies the project color scheme
- restores the original `settings.json` after exit

If Windows Terminal is not installed, the launcher now falls back to a normal run without touching terminal settings.

### Launcher files

The repository also contains helper launcher scripts:

- `run_notepad_wt.cmd`
- `run_notepad_wt.bat`
- `run_notepad_wt.ps1`

What they are for:

- `run_notepad_wt.cmd` - main Windows entry point
- `run_notepad_wt.bat` - same wrapper as `.cmd`, kept for convenience
- `run_notepad_wt.ps1` - actual launcher logic used by both wrappers

What the PowerShell launcher does:

- tries to find Windows Terminal settings
- temporarily unbinds `Ctrl+V` from the terminal host
- temporarily injects the NOTEPAD color scheme into Windows Terminal
- starts `NOTEPAD.exe` or falls back to `cargo run`
- restores original Windows Terminal settings after exit

Why this exists:

- many terminal hosts intercept `Ctrl+V` themselves
- in Windows Terminal that can turn paste into streamed text instead of a clean app-level paste
- the launcher gives the editor a better chance to receive `Ctrl+V` the way the app expects

Important behavior:

- if Windows Terminal is missing, the launcher does not fail anymore
- in that case it simply runs the app without editing any terminal settings
- if you do not need this behavior, you can ignore these files and just use `cargo run`

---

## Default hotkeys

- `Ctrl+N` - New file
- `Ctrl+O` - Open file
- `Ctrl+S` - Save file
- `Ctrl+Shift+S` - Save file as
- `Ctrl+E` - Open current file in Explorer / file manager
- `Ctrl+F` - Find
- `Ctrl+H` - Replace
- `Ctrl+Z` - Undo
- `Ctrl+Shift+Z` / `Ctrl+Y` - Redo
- `Ctrl+A` - Select all
- `Ctrl+C` - Copy
- `Ctrl+X` - Cut
- `Ctrl+V` / `Alt+V` / `Shift+Insert` - Paste
- `Enter` in Find - Next match
- `Shift+Enter` in Find / Replace - New line in the query
- `Esc` - Close Find / Replace
- `Ctrl+Click` or `Shift+Click` on a link - Open link

---

## Config and storage

The app stores its data in the standard application config location:

- Windows: `%APPDATA%\\notepad`
- Linux: `$XDG_CONFIG_HOME/notepad` or `~/.config/notepad`
- macOS: `~/Library/Application Support/notepad`

Files used by the app:

- `.config` - editor config, recent files, hotkeys, highlight toggle
- `documents/` - generated paths for new unsaved documents
- `log.txt` - local log file

Example hotkey section:

```json
[
  { "action": "find", "shortcut": "Ctrl+F" },
  { "action": "replace", "shortcut": "Ctrl+H" },
  { "action": "paste", "shortcut": "Alt+V" }
]
```

---

## Notes / limitations

- This is a custom TUI editor, not a full parser-based IDE.
- Syntax highlighting is intentionally generic and rule-based.
- `Ctrl+V` behavior depends on terminal host; `Alt+V` and `Shift+Insert` are safer fallbacks.
- Control characters are sanitized on render to avoid breaking the terminal output.
- The project currently has no tabs, splits, or plugin system.

---

## Project structure

- `src/app.rs` - event loop, layout orchestration, drawing
- `src/app_actions.rs` - app-level actions
- `src/app_dialogs.rs` - native dialogs
- `src/text_buffer.rs` - editor buffer, search, history, selection, links
- `src/input.rs` - input state and command routing
- `src/shortcuts.rs` - configurable hotkey parser
- `src/syntax_highlight.rs` - generic syntax highlighting
- `src/panels/text_editor_panel.rs` - main editor panel
- `src/panels/search_panel.rs` - find / replace popup
- `src/panels/files_panel.rs` - recent files panel
- `src/panels/menu_panel.rs` - top menu

---

## Tech

- Rust
- `crossterm`
- `serde` / `serde_json`
- `cli-clipboard`
- `rfd`
