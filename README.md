# LeanSticky

Lightweight cross-platform sticky notes for Windows, macOS, and Linux.

## Status

Prototype implementation in progress.

## Goals

- Lightweight desktop app built with Rust
- GUI with `egui` / `eframe`
- Shared `core` for desktop and TUI
- File-based notes with automatic reload on external changes
- YAML-based note data and local config
- Built-in Japanese and English UI messages

## Docs

- Japanese design: [docs/ja/design.md](docs/ja/design.md)
- English design: [docs/en/design.md](docs/en/design.md)
- Schemas: [docs/schema/v001/](docs/schema/v001/)

## Workspace

```text
.
├── AGENTS.md
├── core/
├── desktop/
├── docs/
├── messages.yml
├── tui/
├── README.md
└── ...
```

## Notes

- Local config is stored outside synced note folders.
- Local session restore state is stored separately from synced notes.
- Public schemas are versioned under `docs/schema/` for GitHub Pages.
