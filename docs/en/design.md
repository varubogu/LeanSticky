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
- Run as a single process with one management window and multiple note windows.
- The first implementation keeps standard OS window chrome and prioritizes multi-note display, autosave, and restore on relaunch.
- Frameless UI, tray support, and always-on-top behavior are later milestones.
- Text is autosaved after 1.5 seconds of idle time.
- Each autosave adds one history snapshot.
- All visible UI strings are loaded from localized message data.

### TUI
- Runs in terminal environments such as tmux.
- Shares note files, history behavior, and sync logic with the GUI app.
- Uses the same localized message keys as the GUI app.
- The first implementation only creates the crate skeleton; the usable terminal UI is a later milestone.

## 3. Non-Functional Requirements
- Lightweight first: keep dependencies small and startup work minimal.
- Offline-first and file-based.
- Use YAML for local config, local session state, and note data.
- Store config and session files locally only and keep them out of synced note folders.
- Safe for cloud-synced folders such as Dropbox, iCloud Drive, OneDrive, or Syncthing.
- Preserve the design for automatic reload of external file changes, with the implementation arriving in a later milestone.
- Recover cleanly from partial writes or sync conflicts when possible.

## 4. Architecture
- `core`: shared domain logic, YAML persistence, history, config, message loading, bootstrap logic.
- `desktop`: egui desktop app with a management window, multiple note windows, autosave, and single-instance behavior.
- `tui`: terminal UI app using the same `core`.

The `core` crate should not depend on GUI frameworks.

## 5. Localization
### Message source
Localized UI text is defined in `messages.yml`.

Format:
```yaml
sample_key:
  ja: サンプルメッセージ
  en: sample message
```

### Rules
- Each message key must provide `en`.
- v1 supports Japanese and English.
- Missing translations fall back to English.
- The selected UI language comes from local config, with optional system-locale fallback.
- `messages.yml` should be validated by the published schema.
- The first implementation uses the bundled `messages.yml`.

## 6. Storage Model
### Local config
The app keeps its config in the platform-local config directory and does not sync it with note data.

Examples:
- Windows: `%AppData%/LeanSticky/config.yml`
- macOS: `~/Library/Application Support/LeanSticky/config.yml`
- Linux: `${XDG_CONFIG_HOME:-~/.config}/leansticky/config.yml`

Fields:
- `schema_version`
- `locale`
- `note_roots`
- `autosave_delay_ms`
- `history.max_snapshots_per_note`

### Local session
Management-window placement, open note windows, and restore state live in `session.yml`.

Examples:
- Windows: `%AppData%/LeanSticky/session.yml`
- macOS: `~/Library/Application Support/LeanSticky/session.yml`
- Linux: `${XDG_CONFIG_HOME:-~/.config}/leansticky/session.yml`

Fields:
- `schema_version`
- `management_window`
- `open_windows[]`

Each `open_windows` entry stores:
- `root`
- `note_id`
- `position`
- `size`
- `maximized`

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

Fields:
- `schema_version`
- `id`
- `title`
- `content`
- `created_at`
- `updated_at`

Machine-local window placement and restore state are intentionally excluded from note files.

### Rationale
- Config and session state remain machine-local.
- Note content stays easy to sync safely across machines.
- Multiple note roots allow flexible folder-based organization.
- YAML is easy to inspect and edit manually.
- History snapshots stay append-only and easy to prune.

## 7. Save and History Rules
- Start an idle timer for `autosave_delay_ms` after each edit.
- Reset the timer on further input.
- When the timer expires:
  - write the current note atomically as YAML
  - append one YAML history snapshot
  - update in-memory dirty state
- The first implementation applies only `max_snapshots_per_note` pruning.
- Use atomic write via temporary file + rename where supported.

## 8. Startup and Restore
- On first launch, create a default note root under the platform data directory.
- Generate `config.yml` and `session.yml` automatically when missing.
- If there are no notes yet, create one empty note.
- On startup, restore note windows from `session.yml`.
- Missing restored notes are skipped.
- If no restorable windows remain, open the first existing note, or create a new one if needed.
- The `open_windows` array order is the restore order.

## 9. Desktop UI
### Main principles
- Keep the interface visually light.
- Notes should feel like floating tools, not heavy document windows.
- Separate management and note windows to support multi-note workflows.

### First implementation
- The management window provides note listing, create, and open actions.
- Each note opens in a separate window inside the same process.
- A second launch sends an activate request to the existing process and exits.
- The existing process brings the management window to the front.
- Note windows allow editing title and content.
- Save state is shown with subtle status text.

### Later milestones
- Frameless custom chrome
- Always-on-top toggle
- Tray integration
- External file watching

## 10. TUI Design
- Reuse the same note roots and local config.
- Eventually support note list, open/edit, search, create, delete, and history browse.
- Avoid mouse-only assumptions.
- Use the same message keys and language selection as the GUI app.
- Work well inside tmux and other multiplexers.

## 11. Published Schema
Docs are intended for GitHub Pages publishing.

Publish versioned schemas under `docs/schema/` so they are accessible as:
- `/schema/v001/schema.json`
- `/schema/v001/config.schema.json`
- `/schema/v001/session.schema.json`
- `/schema/v001/note.schema.json`
- `/schema/v001/messages.schema.json`

Recommended usage:
- `schema.json`: version entry point and shared definitions
- `config.schema.json`: local config YAML schema
- `session.schema.json`: local restore-state YAML schema
- `note.schema.json`: note and history YAML schema
- `messages.schema.json`: localized message catalog schema

## 12. Suggested Dependencies
- `eframe` / `egui` for GUI
- `serde` + `serde_yaml` for persistence
- `directories` for platform-local paths
- `single-instance` for single-process startup
- `ulid` for note IDs
- Avoid `tokio` until async needs are clearly justified

## 13. Milestones
1. Shared `core` crate with YAML note model, config, session, history, i18n loading, and atomic save.
2. Basic desktop note windows with autosave and a management window.
3. Single-instance startup and session restore.
4. Custom window chrome and always-on-top toggle.
5. Tray integration and localized note list.
6. External file reload, multi-root watching, and conflict handling.
7. TUI implementation on top of `core`.
