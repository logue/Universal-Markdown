# LukiWiki Rustパーサー実装プラン

**プロジェクト概要**: Markdown上位互換を目指すLukiWikiのパース処理をRustで実装する。CommonMark仕様テストを合理的にパス(75%+目標)しつつ、レガシー構文を保持する。

**作成日**: 2026年1月23日
**Rustバージョン**: 1.93 (最新安定版)
**ライセンス**: MIT

## 目標

- CommonMark仕様テストで75%以上のパス率を達成
- 既存LukiWikiコンテンツとの後方互換性を維持
- HTML直接入力を禁止し、セキュアなHTML生成のみ許可
- 既存Markdownパーサー（`pulldown-cmark`/`comrak`）を基盤として活用

## 実装ステップ

### Step 1: プロジェクト初期化とHTML安全化層

**目的**: プロジェクト基盤とセキュリティ層の構築

**作業内容**:

- `Cargo.toml`に以下の依存関係を追加:
  - `comrak` (GFM対応ASTベースMarkdownパーサー、推奨)
  - `html-escape` (HTML安全化)
  - `maud` または `markup` (型安全HTML生成)
- [src/sanitizer.rs](src/sanitizer.rs)を作成
  - 入力テキストの完全HTMLエスケープ処理
  - HTMLエンティティ(`&nbsp;`, `&lt;`等)の保持ロジック
  - `<tag>`形式の完全除去
  - XSS脆弱性の防止

**成果物**:

- Cargoプロジェクト構造
- HTML安全化モジュール
- 単体テスト（悪意あるHTML入力のテスト）

---

### Step 2: コアMarkdown基盤の構築

**目的**: 標準Markdown機能の実装とCommonMark準拠

**作業内容**:

- [src/parser.rs](src/parser.rs)に`comrak`ベースのパーサーを実装
- サポート機能:
  - ATX見出し (`#` ~ `#####`)
  - 段落と改行
  - フェンスコードブロック (` ``` `)
  - 基本リスト（順序なし `-`、順序付き `1.`）
  - リンク `[text](url)`
  - 画像 `![alt](url)`
  - 強調 `*italic*`、`**bold**`
- [tests/commonmark.rs](tests/commonmark.rs)でCommonMark仕様テスト統合
- 初期目標: コア機能で85%+パス率

**成果物**:

- 基本Markdownパーサー
- CommonMark統合テスト環境
- パース→HTML変換パイプライン

---

### Step 3: LukiWiki構文拡張の実装 ✅ 完了

**目的**: LukiWiki独自構文のサポート

**ステータス**: ✅ **完了** (2025年版)

**作業内容**:

- [src/lukiwiki/](src/lukiwiki/)ディレクトリ作成 ✅
- 実装する構文:
  - **ブロック引用**: `> ... <` (開始・終了タグ形式) ✅
    - [src/lukiwiki/conflict_resolver.rs](src/lukiwiki/conflict_resolver.rs)でマーカー方式実装
  - **LukiWiki強調**: ✅
    - `''text''` → `<b>text</b>` (視覚的な太字)
    - `'''text'''` → `<i>text</i>` (視覚的な斜体)
    - [src/lukiwiki/emphasis.rs](src/lukiwiki/emphasis.rs)実装完了
  - **Markdown強調**: ✅
    - `**text**` → `<strong>text</strong>` (セマンティックな強調)
    - `*text*` → `<em>text</em>` (セマンティックな強調)
    - 注: 表示は同じだが、意味合いが異なる（視覚的 vs セマンティック）
  - **ブロック装飾プレフィックス** (行頭に配置): ✅
    - `COLOR(fg,bg): text` - 前景色・背景色指定（空白時は`inherit`）
      - 例: `COLOR(grey): 灰色の文` → `<p style="color: grey">灰色の文</p>`
      - 例: `COLOR(,#CCCCFF): 背景色のみ` → `<p style="background-color: #CCCCFF">背景色のみ</p>`
    - `SIZE(rem): text` - フォントサイズ指定
      - 例: `SIZE(1.5): 大きな文` → `<p style="font-size: 1.5rem">大きな文</p>`
    - `RIGHT: text` - 右寄せ
    - `CENTER: text` - 中央寄せ
    - `LEFT: text` - 左寄せ（デフォルト）
    - [src/lukiwiki/block_decorations.rs](src/lukiwiki/block_decorations.rs)実装完了
  - **インライン装飾関数** (プラグインのインライン型と同じ表記): ✅
    - `&color(fg,bg){text};` - 文字色・背景色指定（空白時は`inherit`）
      - 例: `&color(red){赤い文字};` → `<span style="color: red">赤い文字</span>`
      - 例: `&color(,yellow){黄色背景};` → `<span style="background-color: yellow">黄色背景</span>`
    - `&size(rem){text};` - フォントサイズ指定
    - `&sup(text);` - 上付き文字 → `<sup>text</sup>`
    - `&sub(text);` - 下付き文字 → `<sub>text</sub>`
    - `&lang(locale){text};` - 言語指定 → `<span lang="locale">text</span>`
      - 例: `&lang(en){Hello};` → `<span lang="en">Hello</span>`
    - `&abbr(text){description};` - 略語説明 → `<abbr title="description">text</abbr>`
    - [src/lukiwiki/inline_decorations.rs](src/lukiwiki/inline_decorations.rs)実装完了
  - **プラグインシステム** (拡張可能なWiki機能): ✅
    - **インライン型**: `&function(args){content};`
      - パース出力: `<span class="plugin-function" data-args="args" data-content="content">&#38;function</span>`
    - **ブロック型（複数行）**: `@function(args){{ content }}`
      - パース出力: `<div class="plugin-function" data-args="args" data-content="content">@function</div>`
    - **ブロック型（単行）**: `@function(args){content}`
      - パース出力: `<div class="plugin-function" data-args="args" data-content="content">@function</div>`
    - **ネストされたプラグイン呼び出しをサポート**:
      - 例: `&outer(arg1){text &inner(arg2){nested}; more};`
      - data-content属性内で内側のプラグイン構文を生のまま保持
    - **Wiki構文の保持**:
      - プラグインコンテンツ内のWiki構文（`**bold**`など）はそのまま保持
      - JavaScript側で再パース可能な状態で出力
    - 実装方法: [src/lukiwiki/conflict_resolver.rs](src/lukiwiki/conflict_resolver.rs)でbase64エンコーディングによる保護
    - 注: プラグインの実際の実行処理は別レイヤー(JavaScript/フロントエンド)で処理
    - パーサーはプラグイン呼び出しの検出とメタデータ保持のみを担当
    - **重要**: プラグインが生成するHTML出力は許可される（プラグイン側で安全性を保証）
  - **テーブル**: `|~Header|h` 形式 ⏸️ 保留
    - 行修飾子: `h`(ヘッダー), `f`(フッター), `c`(キャプション)
    - セル内色/配置: `COLOR(fg,bg):`, `RIGHT:`, `CENTER:`, `LEFT:`
  - **定義リスト**: `:term|definition` ⏸️ 保留

**成果物**:

- LukiWiki構文パーサーモジュール群 ✅
  - emphasis.rs (5 tests)
  - block_decorations.rs (7 tests)
  - inline_decorations.rs (9 tests)
  - plugins.rs (11 tests)
- レガシー構文互換性テスト ✅ (31 LukiWiki syntax tests passing)

**テスト結果**: 101 tests passing

- 63 unit tests
- 18 CommonMark tests
- 13 conflict resolution tests
- 7 doctests

---

### Step 4: 構文競合の解決 ✅ 完了

**目的**: MarkdownとLukiWiki構文の衝突を適切に処理

**ステータス**: ✅ **完了** (2025年版)

**作業内容**:

- [src/lukiwiki/conflict_resolver.rs](src/lukiwiki/conflict_resolver.rs)を作成 ✅
- マーカーベース前処理システム実装 ✅
  - プリプロセス: LukiWiki構文を`{{MARKER:...:MARKER}}`形式で保護
  - サニタイゼーション: マーカーはHTMLエスケープされない
  - ポストプロセス: マーカーを適切なHTMLに復元
- 競合解決ルール: ✅
  - **ブロック引用**:
    - LukiWiki形式 `> ... <` 優先
    - 閉じタグ `<` の検出により判定
    - 閉じタグなしの場合はMarkdown `>` 行頭プレフィックスとして処理
  - **リストマーカー**:
    - 順序なし: `-` (LukiWiki) と `*` (Markdown) 両対応
    - 順序付き: `+` (LukiWiki) と `1.` (Markdown) 両対応
  - **水平線**:
    - `----` (4文字以上のハイフン) を優先
    - `***`, `___` も対応（CommonMark準拠）
  - **強調表現**: ✅
    - Markdown: `*em*`, `**strong**` → セマンティックタグ (`<em>`, `<strong>`)
    - LukiWiki: `'''italic'''`, `''bold''` → 視覚的タグ (`<i>`, `<b>`)
    - 両方サポート、ネスト時の優先順位を定義
    - 表示は同一だが、HTMLの意味合いが異なる
  - **プラグイン構文の保護**: ✅
    - インライン: `&function(args){content};`
    - ブロック: `@function(args){{ content }}` / `@function(args){content}`
    - base64エンコーディングでコンテンツを安全に保持
    - ネストされたプラグインと内部のWiki構文を完全保護

**成果物**:

- 構文曖昧性解消モジュール ✅
- プリプロセス/ポストプロセスパイプライン ✅
- 競合検出診断ツール ✅

**テスト結果**: 13 conflict resolution tests passing

- 競合ケースの網羅的テスト
- 優先順位ドキュメント（コード内コメント）

---

### Step 5: Markdown拡張機能の追加

**目的**: CommonMark準拠率向上と現代的Markdown機能

**作業内容**:

- [src/markdown/tables.rs](src/markdown/tables.rs):
  - Markdown形式テーブル `| Header |` 構文
  - ソート可能テーブル生成
  - アライメント指定 (`:--`, `:-:`, `--:`)
- [src/markdown/extras.rs](src/markdown/extras.rs):
  - **Setext見出し** (下線形式)
  - **参照スタイルリンク**: `[text][ref]` + `[ref]: url`
  - **バックスラッシュエスケープ**: `\*` → リテラル `*`
  - **GFM打ち消し線**: `~~text~~` (PukiWiki `%%text%%` も保持)
  - **自動URL検出**: `http://example.com` → リンク化
  - **ハード改行**: 行末2スペースまたは `\`

**成果物**:

- Markdown拡張機能モジュール群
- CommonMark準拠率75%+達成
- GFM互換性

---

### Step 6: 高度なLukiWiki機能

**目的**: LukiWiki固有の複雑な機能をサポート

**作業内容**:

- [src/lukiwiki/nested_blocks.rs](src/lukiwiki/nested_blocks.rs):
  - **リスト内ブロック要素**
    - リスト項目内にテーブル、コードブロック等を許可
    - CommonMark違反だが互換性のため必須
    - インデント解析による親子関係判定
- その他高度機能:
  - **タスクリスト拡張**: `[ ]`, `[x]`, `[-]` (不確定状態)
  - **カスタムリンク属性**: `[text](url){id class}`
  - **添付ファイル構文**: `PageName/FileName`
  - **相対パス**: `./page`, `../page`, `/page`

**成果物**:

- 複雑なネスト構造のパース実装
- 既存LukiWikiコンテンツ互換性テスト
- パフォーマンステスト（深いネスト）

---

### Step 7: HTML生成とテスト

**目的**: 安全なHTML出力と包括的テスト

**作業内容**:

- [src/renderer.rs](src/renderer.rs):
  - `maud`または`markup`クレートで型安全HTML生成
  - ユーザー入力の直接埋め込み禁止
  - HTMLエンティティの適切な処理
- テストスイート:
  - [tests/commonmark.rs](tests/commonmark.rs): CommonMark仕様テスト、目標75%+
  - [tests/legacy_compat.rs](tests/legacy_compat.rs): LukiWiki互換性
  - [tests/php_comparison.rs](tests/php_comparison.rs): PHP実装との差分検証
  - [tests/security.rs](tests/security.rs): XSS等のセキュリティテスト
- ベンチマーク:
  - 大規模ドキュメントのパース速度
  - メモリ使用量

**成果物**:

- 完成したHTMLレンダラー
- 包括的テストスイート
- パフォーマンスベンチマーク結果
- セキュリティ監査レポート

---

## 技術仕様

### アーキテクチャ

```
Input Text
    ↓
[HTML Sanitizer] - HTMLエスケープ、エンティティ保持
    ↓
[Lexer/Tokenizer] - LukiWiki/Markdown構文検出
    ↓
[Parser] - comrakベースAST構築
    ↓
[LukiWiki Extensions] - 独自ノード追加
    ↓
[Disambiguator] - 構文競合解決
    ↓
[AST Transformer] - 最適化・検証
    ↓
[HTML Renderer] - 型安全HTML生成
    ↓
[Plugin Processor] - プラグイン実行（HTML出力許可）
    ↓
Output HTML
```

### 主要な依存クレート

```toml
[package]
name = "lukiwiki-parser"
version = "0.1.0"
edition = "2024"
rust-version = "1.93"

[dependencies]
comrak = "0.28"                    # Markdown parser (GFM)
ammonia = "4.0"                    # HTML sanitization (html-escapeの後継)
maud = "0.26"                      # Type-safe HTML (alternative: markup)
regex = "1.11"                     # Pattern matching
once_cell = "1.20"                 # Lazy static initialization (lazy_staticの後継)
unicode-segmentation = "1.12"      # Grapheme cluster handling

[dev-dependencies]
insta = "1.41"                     # Snapshot testing
criterion = "0.5"                  # Benchmarking
```

**注1**: Rust 1.93 + Edition 2024の最新機能（改善された型推論、パターンマッチング拡張等）を活用します。
**注2**: シンタックスハイライトはJavaScript側（Codemirror）で動的に実装するため、Rust側では言語情報のみをHTML属性として出力します。

### ディレクトリ構造

```
lukiwiki-parser/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── parser.rs           # エントリーポイント
│   ├── sanitizer.rs        # HTML安全化
│   ├── disambiguator.rs    # 構文競合解決
│   ├── renderer.rs         # HTML生成
│   ├── ast.rs              # AST定義
│   ├── markdown/           # Markdown拡張
│   │   ├── mod.rs
│   │   ├── tables.rs
│   │   └── extras.rs
│   └── lukiwiki/           # LukiWiki拡張
│       ├── mod.rs
│       ├── table.rs
│       ├── definition_list.rs
│       ├── blockquote.rs
│       ├── text_decorations.rs
│       └── nested_blocks.rs
├── tests/
│   ├── commonmark.rs       # CommonMark仕様テスト
│   ├── lukiwiki_compat.rs  # LukiWiki互換性
│   ├── php_comparison.rs   # PHP実装比較
│   └── security.rs         # セキュリティテスト
└── benches/
    └── parse_bench.rs      # パフォーマンステスト
```

---

## 構文優先順位ポリシー

### 競合時の解決ルール

1. **ブロック引用**:
   - LukiWiki `> ... <` 優先（閉じタグ検出）
   - 閉じタグなし → Markdown `>` 行頭プレフィックス

2. **強調表現**:
   - 両スタイルサポート（共存）
   - Markdown → セマンティックタグ (`<strong>`, `<em>`) - 意味的な強調
   - LukiWiki → 視覚的タグ (`<b>`, `<i>`) - 見た目の装飾
   - 違い: アクセシビリティやSEOへの影響が異なる
   - **潜在的矛盾**: `'''text'''` (3個) がMarkdownの太字 `***text***` と視覚的に類似

3. **リストマーカー**:
   - 両スタイルサポート
   - `-`, `*` → 順序なしリスト
   - `+`, `1.` → 順序付きリスト
   - **潜在的矛盾**: LukiWikiの `+` がMarkdownでは順序なしリストに使用される場合がある

4. **水平線**:
   - `----` (4+文字) 優先
   - `***`, `___` も対応
   - **矛盾なし**: CommonMark準拠

5. **テーブル**:
   - LukiWiki形式とMarkdown形式を構文で判別
   - LukiWiki: `|cell|h` (行修飾子あり)
   - Markdown: `| header |\n|---|` (区切り行あり)
   - **矛盾なし**: 構文が明確に異なる

6. **インライン装飾関数**:
   - `&color(...)`, `&size(...)` 等
   - **矛盾なし**: Markdownにこの構文は存在しない

7. **ブロック装飾プレフィックス**:
   - `COLOR(...): text`, `SIZE(...): text` 等
   - **潜在的矛盾**: コロン `:` がMarkdownの定義リストと競合する可能性

### Markdown仕様との矛盾箇所まとめ

| LukiWiki構文  | Markdown構文        | 矛盾度 | 解決策                   |
| ------------- | ------------------- | ------ | ------------------------ |
| `'''text'''`  | `***text***`        | 中     | 3連続クォートを優先検出  |
| `+ item`      | `+ item` (一部方言) | 低     | 順序付きリストとして統一 |
| `COLOR(...):` | `: definition`      | 低     | 大文字キーワードで判別   |
| `> ... <`     | `> quote`           | 低     | 閉じタグで判別           |

**対策**:

- パーサーの優先順位で明示的に処理
- Step 4（構文競合解決）で包括的にテスト
- 曖昧な入力に対する警告メッセージの実装

---

## CommonMark準拠目標

### 目標パス率

- **コア機能** (見出し、リスト、コード、リンク、強調): **85%+**
- **拡張機能** (テーブル、参照リンク、Setext): **70%+**
- **全体**: **75%+**

### 許容される失敗

- LukiWiki構文と競合するケース
- HTML出力が要求されるテスト（HTML入力禁止のため）
- 極端に複雑なネスト構造の一部エッジケース

---

## 実装フェーズ

### Phase 1: MVP（基本機能）

- Step 1-3: 基盤 + Markdown + LukiWiki基本
- 目標期間: 2-3週間
- 成果: 基本的なWiki記法のパース・変換

### Phase 2: 準拠性向上

- Step 4-5: 競合解決 + Markdown拡張
- 目標期間: 2週間
- 成果: CommonMark 75%+達成

### Phase 3: 高度機能

- Step 6: LukiWiki複雑機能
- 目標期間: 1-2週間
- 成果: 完全なレガシー構文互換性

### Phase 4: 完成・最適化

- Step 7: テスト・最適化
- 目標期間: 1週間
- 成果: プロダクション品質

**総計**: 6-8週間

---

## セキュリティ方針

### HTML入力制限

**原則**: 直接HTML入力は**完全禁止**

**実装**:

1. 入力時に全てのHTMLタグをエスケープ
2. HTMLエンティティ（`&nbsp;`, `&lt;`等）のみ保持
3. パーサー生成HTMLのみ出力に使用
4. XSS攻撃ベクトルの完全遮断

**例外**: プラグイン出力のHTMLは許可

- プラグインが生成するHTMLは信頼されたコードとして扱う
- プラグイン側でサニタイズ責任を負う
- ユーザー入力をプラグインに渡す場合は、プラグイン内でエスケープ必須

---

## 未実装機能（提案段階）

以下の機能は仕様書で提案されているが、MVP後の追加機能として保留:

- ラジオボタン: `( )`, `(x)`
- トグルボタン: `< >`, `<x>`
- 絵文字: `::emoji_name::`
- 画像リンク: `[![alt](image)](link)`

これらは需要と仕様確定後に実装を検討。

---

## 参考リソース

- **PHP実装**: https://github.com/logue/LukiWiki/tree/master/app/LukiWiki
- **仕様書**: https://github.com/logue/LukiWiki-core/blob/master/docs/rules.md
- **CommonMark仕様**: https://spec.commonmark.org/
- **GFM仕様**: https://github.github.com/gfm/

---

## リスク管理

### 高リスク

- 構文曖昧性によるパース失敗 → 包括的テストで対処
- セキュリティ脆弱性 → 入力サニタイズ徹底
- パフォーマンス問題 → 早期ベンチマーク

### 中リスク

- CommonMark準拠困難 → 目標を75%に設定（現実的）
- レガシー構文互換性不足 → PHP実装との比較テスト

### 低リスク

- Rustクレートエコシステム → 実績あるクレート使用
- チーム習熟度 → 段階的実装で学習時間確保

---

## 成功基準

1. ✅ CommonMark仕様テスト75%以上パス
2. ✅ 既存LukiWikiコンテンツが正常変換
3. ✅ HTML直接入力の完全ブロック
4. ✅ XSS等セキュリティテスト全パス
5. ✅ 大規模ドキュメント（10000行）が1秒以内にパース

---

**プラン策定**: 2026年1月23日  
**ライセンス**: MIT License  
**次のステップ**: Step 1（プロジェクト初期化）の開始
