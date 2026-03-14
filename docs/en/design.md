# LeanSticky Design

## 1. Goal
- Build a lightweight sticky note app for desktop platforms.
- Prioritize low memory use, fast startup, and simple local persistence.
- Support Windows, macOS, and Linux from a shared Rust codebase.
- Provide both a GUI app and a TUI app backed by the same storage model.
- Support Japanese and English from the first release.

## 2. Product Scope
### Desktop GUI
- Built with Rust + egui/eframe.
- Frameless custom window UI instead of OS default chrome.
- Supports always-on-top toggle, minimize, maximize/restore, and close.
- Control buttons stay hidden by default and appear with a hover animation.
- Window can be moved and resized freely.
- Text is autosaved after 1.5 seconds of idle time.
- Each autosave adds one history snapshot.
- Tray/taskbar integration shows note list on click and menu on right click.
- Right-click menu includes at least Settings and Quit.
- All visible UI strings are loaded from localized message data.

### TUI
- Runs in terminal environments such as tmux.
- Shares note files, history behavior, and sync logic with the GUI app.
- Uses the same localized message keys as the GUI app.
- Focuses on quick note browsing and editing, not graphical window behavior.

## 3. Non-Functional Requirements
- Lightweight first: keep dependencies small and startup work minimal.
- Offline-first and file-based.
- Use YAML for both local config and note data.
- Store configuration locally only and keep it out of synced note folders.
- Safe for cloud-synced folders such as Dropbox, iCloud Drive, OneDrive, or Syncthing.
- Detect external file changes and reload notes automatically.
- Recover cleanly from partial writes or sync conflicts when possible.

## 4. Architecture
- `core`: shared domain logic, YAML persistence, history, file watching, config, i18n loading.
- `desktop`: egui desktop app with custom window chrome and tray support.
- `tui`: terminal UI app using the same `core`.

The `core` crate should not depend on GUI frameworks.

## 5. Localization
### Message source
Localized UI text is defined in `messages.yml`.

Format:
```yaml
sample_key:
  ja: サンプルメッセージ
  en: sample messages
```

### Rules
- Each message key maps to both `ja` and `en`.
- v1 requires Japanese and English only.
- Missing translations fall back to English.
- The selected UI language comes from local config, with optional system-locale fallback.
- `messages.yml` should be validated by the published schema.

## 6. Storage Model
### Local config
The app keeps its config in the platform-local config directory and does not sync it with note data.

Examples:
- Windows: `%AppData%/LeanSticky/config.yml`
- macOS: `~/Library/Application Support/LeanSticky/config.yml`
- Linux: `${XDG_CONFIG_HOME:-~/.config}/leansticky/config.yml`

Suggested fields:
- `schema_version`
- `locale`
- `note_roots`: list of folder paths to load
- `autosave_delay_ms`
- `history`

### Note roots
The config can contain any number of note root folders.

Each note root uses YAML files:
```text
<note-root>/
  notes/
    <note-id>.yml
  history/
    <note-id>/
      <timestamp>.yml
```

### Note file
Each note is stored as a single UTF-8 YAML file.

Suggested fields:
- `schema_version`
- `id`
- `title`
- `content`
- `created_at`
- `updated_at`
- `always_on_top`
- `window`: position, size, maximized state

### Rationale
- Local-only config keeps machine-specific settings separate from shared note data.
- Multiple note roots allow flexible folder-based organization.
- YAML is easy to inspect and edit manually.
- History snapshots are append-only and can be trimmed by policy.

## 7. Save and History Rules
- Start a 1.5 second idle timer after each edit.
- Reset the timer on further input.
- When the timer expires:
  - write the current note atomically as YAML
  - append one YAML history snapshot
  - update in-memory dirty state
- Use atomic write via temporary file + rename where supported.
- Keep a bounded history policy, for example:
  - dense recent history for the last hour
  - reduced history for older snapshots
  - max snapshot count per note

## 8. Sync and Reload Behavior
- Watch every configured note root for external changes.
- On external modification:
  - reload the changed note if the editor is clean
  - if the note is dirty locally, keep local text in memory and surface a conflict state
- On deleted files:
  - remove the note from the list after confirmation rules defined in config
- On newly added files:
  - load them into the note list automatically
- Config file changes should also be reloaded locally without restarting when practical.

Conflict handling should stay simple in v1:
- prefer non-destructive behavior
- keep local unsaved text
- expose "Reload from disk" and "Save as new version" actions

## 9. Desktop UI
### Main principles
- Minimal visual weight.
- Notes should feel like floating tools, not full document windows.
- Custom chrome must remain keyboard accessible.

### Window controls
- Pin icon for always-on-top toggle.
- Minimize button.
- Maximize/restore button.
- Close button.
- Buttons are hidden at rest and appear on hover with a subtle rise/fade animation.

### Note view
- Main area is a text editor with low-latency typing.
- Optional compact title field.
- Visual dirty state should be subtle because saving is automatic.
- Labels, menus, and status text come from `messages.yml`.

## 10. Tray Behavior
- Left click: open list of notes and quick actions.
- Right click: show menu with localized Settings and Quit entries.
- Tray should remain available even when all note windows are closed.

## 11. TUI Design
- Reuse the same note roots and local config.
- Support note list, open/edit, search, create, delete, and history browse.
- Avoid mouse-only assumptions.
- Use the same message keys and language selection as the GUI app.
- Work well inside tmux and other multiplexers.

## 12. Published Schema
Docs are intended for GitHub Pages publishing.

Publish versioned schemas under `docs/schema/` so they are accessible as:
- `/schema/v001/schema.json`
- `/schema/v001/config.schema.json`
- `/schema/v001/note.schema.json`
- `/schema/v001/messages.schema.json`

Recommended usage:
- `schema.json`: version entry point and shared definitions
- `config.schema.json`: local config YAML schema
- `note.schema.json`: note and history YAML schema
- `messages.schema.json`: localized message catalog schema

## 13. Suggested Dependencies
- `eframe` / `egui` for GUI
- `serde` + `serde_yaml` for persistence
- `notify` for filesystem watching
- `tray-icon` for system tray integration
- `directories` for platform-local config paths
- `tokio` only if async needs are proven necessary; avoid by default

## 14. Milestones
1. Shared `core` crate with YAML note model, YAML config, history, i18n loading, and atomic save.
2. Basic desktop note window with autosave.
3. Custom window chrome and always-on-top toggle.
4. Tray integration and localized note list.
5. External file reload, multi-root watching, and conflict handling.
6. TUI implementation on top of `core`.
