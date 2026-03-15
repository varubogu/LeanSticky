# AGENTS

- Keep the project lightweight.
- Use Rust with a shared `core`, plus `desktop` and `tui`.
- Desktop UI targets `egui`/`eframe`.
- Support Windows, macOS, and Linux.
- Store config and note data as YAML.
- Keep config local-only and support multiple note root folders.
- Use `messages.yml` for `ja` and `en` UI strings.
- Reload external file changes automatically.
- Limit configurable `cargo` command job counts to 4, for example `-j 4`.
- Prefer minimal dependencies and simple architecture.
- Update `docs/en`, `docs/ja`, and `docs/schema` when formats change.
