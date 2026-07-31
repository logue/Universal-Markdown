# 基本Markdown機能

**最終更新**: 2026年5月18日

Universal Markdown が提供する基本 Markdown 機能の実装一覧です。

## 対応範囲

- ATX 見出し (`#` から `######`)
- 段落・改行
- リスト（順序あり/なし）
- 強調（`*text*` → `<em>`, `**text**` → `<strong>`）
- リンク・画像
- 明示的自動リンク（`<https://...>`）
- GFM テーブル
- GFM タスクリスト
- GFM 取り消し線（`~~...~~`）

> [!NOTE]
> 標準CommonMarkでは `__text__` も `**text**` と同じ `<strong>` になりますが、
> UMDでは「複数の書き方に同じ意味を持たせない」という設計方針により
> `__text__` をDiscord風の下線（`<u>`）専用構文として再定義しています。
> 詳細は [architecture.md の設計方針](architecture.md#設計方針)を参照してください。

## コードブロック拡張

コードブロックの詳細は以下を参照してください。

- [code-block-extensions.md](code-block-extensions.md)

主な機能:

- Mermaid ブロックの SVG 変換
- syntect によるシンタックスハイライト
- `lang:filename` 形式のファイル名付きコードブロック
- インラインコードのカラーサンプル

## 実装の主担当

- `src/parser.rs`
- `src/extensions/code_block.rs`
- `src/extensions/conflict_resolver.rs`
- `src/extensions/preprocessor.rs`

## 主なテスト

- `tests/commonmark.rs`
- `tests/bootstrap_integration.rs`
- `examples/test_output.rs`
- `examples/code_block_extensions.rs`
