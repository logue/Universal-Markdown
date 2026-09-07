# UMD拡張構文

**最終更新**: 2026年5月18日

Universal Markdown 独自の構文拡張をまとめた仕様です。

## 主要機能

- UMD 強調構文
  - `''太字''` -> `<b>`
  - `'''斜体'''` -> `<i>`
  - `__下線__` -> `<u>`
- UMD 取り消し線
  - `%%...%%` -> `<s>`
- Spoiler
  - `||...||`
  - `&spoiler{...};`
- 定義リスト
  - `:term|definition`
- UMD ブロック引用
  - `> ... <`

## ブロック装飾プレフィックス

行頭プレフィックスで段落/ブロックを装飾します。

- 配置: `LEFT:`, `CENTER:`, `RIGHT:`, `JUSTIFY:`, `TRUNCATE:`
- 色: `COLOR(...)`
- サイズ: `SIZE(...)`
- 複合指定: `SIZE(...): COLOR(...): CENTER: ...`

## インライン装飾関数

- 見た目: `&color`, `&size`（既定はキーワードサイズ `xs`/`sm`/`lg`/`xl` のみ。詳細は [inline-plugins.md](inline-plugins.md)）
- セマンティック: `&abbr`, `&ruby`, `&time`, `&kbd`, `&cite` など
- 改行/折返し: `&br;`, `&wbr;`

## ネスト深度制限

インライン装飾関数の再帰展開には上限があります。

- 設定: `ParserOptions.max_inline_nesting`
- 既定値: `Some(5)`
- 超過時: 該当部分をエラー表示クラスで無効化

## ノート / コールアウトブロック

GFM Alert 風の `> [!TYPE]` ブロック引用を、`<aside class="umd-note umd-note-{type}">` の
ノート/コールアウトブロックに変換します。

構文:

```markdown
> [!NOTE]
> 補足情報。
```

対応タイプ（エイリアスは正規タイプに解決されます）:

- `NOTE`（エイリアス: `SUCCESS`）
- `TIP`（エイリアス: `INFO`）
- `IMPORTANT`
- `WARNING`（エイリアス: `WARN`）
- `CAUTION`（エイリアス: `DANGER`）
- `MUST` / `RECOMMEND` / `DONT` / `NEVER`（RFC 2119 に倣った推奨度セット）
- `EXAMPLE`

`NOTE`・`TIP`・`EXAMPLE` は Digital Publishing WAI-ARIA（DPUB-ARIA）の
`doc-notice` / `doc-tip` / `doc-example` ロールを部分的に付与します。

### アイコン

各タイプのタイトル（`<p class="umd-note-title">`）先頭には `ParserOptions.icons`
（`note` / `tip` / `important` / `warning` / `caution` / `must` / `recommend` /
`dont` / `never` / `example` フィールド）のアイコン HTML が挿入されます。CSS の
`content` によるアイコン表示ではなく、この HTML が実際に挿入される点に注意してください
（`scss/components/note.scss` は配色のみを担当）。

アイコンの既定値・差し替え方法・Iconify スクリプトの埋め込み要否については
[media-tags.md の `icons` オプション](media-tags.md#icons)を参照してください
（`ParserOptions.icons` は媒体アイコンとノートアイコンの両方をまとめて設定します）。

## Step 6: 高度なUMD機能

- 数式
  - `&math(...)`
  - `@math(...)`
- Popover
  - `&popover(...)`
  - `@popover(...)`
- ネストブロック補正（リスト直下のブロック要素）
- タスクリスト拡張（`[-]` の indeterminate）
- カスタムリンク属性（`{#id .class}`）

## 実装の主担当

- `src/extensions/inline_decorations.rs`
- `src/extensions/block_decorations.rs`
- `src/extensions/conflict_resolver.rs`
- `src/extensions/nested_blocks.rs`

## 主なテスト

- `tests/bootstrap_integration.rs`
- `tests/conflict_resolution.rs`
- `tests/test_semantic_integration.rs`
- `examples/test_bootstrap_integration.rs`
