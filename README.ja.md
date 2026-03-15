# LeanSticky

Windows、macOS、Linux 向けの軽量なクロスプラットフォーム付箋アプリです。

## ステータス

プロトタイプ実装を進めています。

## 目標

- Rust で構築する軽量なデスクトップアプリ
- `egui` / `eframe` を使う GUI
- デスクトップ版と TUI 版で共有する `core`
- 外部変更の自動再読み込みに対応するファイルベースの付箋
- YAML ベースの付箋データとローカル設定
- 日本語と英語の UI メッセージを標準搭載

## ドキュメント

- 日本語設計書: [docs/ja/design.md](docs/ja/design.md)
- 英語設計書: [docs/en/design.md](docs/en/design.md)
- スキーマ: [docs/schema/v001/](docs/schema/v001/)

## ワークスペース構成

```text
.
├── AGENTS.md
├── core/
├── desktop/
├── docs/
├── messages.yml
├── tui/
├── README.ja.md
├── README.md
└── ...
```

## 補足

- ローカル設定は同期対象のノートフォルダとは分離して保持します。
- ローカルの復元状態は同期対象ノートとは別の `session.yml` に保持します。
- 公開スキーマは GitHub Pages 向けに `docs/schema/` 配下でバージョン管理します。
