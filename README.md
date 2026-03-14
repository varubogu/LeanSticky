# LeanSticky

Lightweight cross-platform sticky notes for Windows, macOS, and Linux.

## Status

Planning stage.

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

## Planned Structure

```text
.
├── AGENTS.md
├── docs/
├── README.md
└── ...
```

## Notes

- Local config is stored outside synced note folders.
- Public schemas are versioned under `docs/schema/` for GitHub Pages.

