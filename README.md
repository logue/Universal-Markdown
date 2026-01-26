# LukiWiki-rs

LukiWikiのWikiパーサーをRustで再実装したプロジェクトです。CommonMark準拠のMarkdownパーサーをベースに、LukiWiki独自の構文拡張をサポートします。

## 特徴

- **CommonMark準拠**: 標準Markdown構文の高い互換性
- **LukiWiki構文サポート**: レガシーPHP実装との互換性
- **フロントマターサポート**: YAML/TOML形式のメタデータ
- **フットノート**: 標準的な脚注構文のサポート
- **セキュリティ**: HTMLサニタイゼーションによるXSS対策
- **WASM対応**: WebAssembly出力によるブラウザ実行
- **拡張性**: プラグインシステムによる機能拡張

## フロントマター

ドキュメントの先頭にYAMLまたはTOML形式のメタデータを配置できます。

### YAML形式

```markdown
---
title: ドキュメントタイトル
author: 著者名
date: 2024-01-23
tags:
  - rust
  - wiki
---

# コンテンツ
```

### TOML形式

```markdown
+++
title = "ドキュメントタイトル"
author = "著者名"
date = 2024-01-23
+++

# コンテンツ
```

フロントマターはHTML出力から除外され、`parse_with_frontmatter()`関数を使用することで別途取得できます。

```rust
use lukiwiki_parser::parse_with_frontmatter;

let input = "---\ntitle: Test\n---\n\n# Content";
let result = parse_with_frontmatter(input);

if let Some(fm) = result.frontmatter {
    println!("Format: {:?}", fm.format); // Yaml
    println!("Content: {}", fm.content);
}
println!("HTML: {}", result.html);
```

## フットノート（脚注）

Markdownの標準的なフットノート構文をサポートしています：

```markdown
本文にフットノート[^1]を含めます。

別の段落で別のフットノート[^note2]を参照。

[^1]: これが最初のフットノートです。

[^note2]: 名前付きフットノートも使えます。
```

フットノートは本文から分離され、`ParseResult`の`footnotes`フィールドで取得できます：

```rust
use lukiwiki_parser::parse_with_frontmatter;

let input = "Text with footnote[^1].\n\n[^1]: Footnote content.";
let result = parse_with_frontmatter(input);

println!("Body: {}", result.html);
if let Some(footnotes) = result.footnotes {
    println!("Footnotes: {}", footnotes);
}
```

フットノートは`<section class="footnotes">`として生成され、適切にスタイリングできます。

## プラグインシステム

LukiWiki-rsは、拡張可能なプラグインシステムを提供します。プラグインは3つのパターンをサポートします。

### インライン型プラグイン

```
&function(args){content};
```

**出力HTML**:

```html
<span class="plugin-function" data-args="args">content</span>
```

**使用例**:

```
&highlight(yellow){重要なテキスト};
```

### ブロック型プラグイン（複数行）

```
@function(args){{ content }}
```

**出力HTML**:

```html
<div class="plugin-function" data-args="args">content</div>
```

**使用例**:

```
@toc(2){{
}}
```

### ブロック型プラグイン（単行）

```
@function(args){content}
```

**出力HTML**:

```html
<div class="plugin-function" data-args="args">content</div>
```

**使用例**:

```
@include(file.txt){デフォルトコンテンツ}
```

### プラグインのネストと再パース

プラグインは**ネスト可能**で、コンテンツ内にさらにプラグインを含めることができます：

```
&outer(arg1){text &inner(arg2){nested}; more};
```

**元のWiki構文がタグのテキストコンテンツとしてそのまま保持**されます。これにより、JavaScript側でプラグイン実行時に再パースが可能です：

```
@box(){{ **bold** and *italic* text }}
```

↓

```html
<div class="plugin-box" data-args="">**bold** and *italic* text</div>
```

プラグイン実装側でDOMのテキストコンテンツを取得し、再度LukiWikiパーサーに渡すことで、ネストされた構文も正しく処理できます。

**重要な特徴：**

- ブロック型プラグインは独立した`<div>`要素として出力され、`<p>`タグで括られません
- コンテンツ内の`&`文字は保持されるため、ネストされたプラグイン構文も再パース可能
- コンテンツ内のWiki構文（`**bold**`など）も生のまま保持されます

### 組み込み装飾との違い

LukiWiki-rsには、プラグインと同じ表記を使う**組み込み装飾関数**があります：

- `&color(fg,bg){text};` - 文字色・背景色
- `&size(rem){text};` - フォントサイズ
- `&sup(text);` - 上付き文字
- `&sub(text);` - 下付き文字
- `&lang(locale){text};` - 言語指定
- `&abbr(text){description};` - 略語説明

これらはパーサー内で直接HTMLに変換されます。組み込み装飾以外の名前は、すべて汎用プラグインとして処理されます。

## LukiWiki構文

## LukiWiki構文

### ヘッダーID

ヘッダーには自動的にURLセーフなIDが付与されます：

**自動採番（デフォルト）**:

```markdown
# Introduction

## Details
```

生成されるHTML:

```html
<h1><a href="#heading-1" id="heading-1"></a>Introduction</h1>
<h2><a href="#heading-2" id="heading-2"></a>Details</h2>
```

**カスタムID（推奨）**:

```markdown
# Introduction {#intro}

## Details {#details}
```

生成されるHTML:

```html
<h1><a href="#intro" id="intro"></a>Introduction</h1>
<h2><a href="#details" id="details"></a>Details</h2>
```

**メリット**:

- ✅ URLセーフ（マルチバイト文字を避ける）
- ✅ 短いURL（SNSでの共有に最適）
- ✅ 安定したリンク（ヘッダーテキスト変更に強い）
- ✅ セキュリティ（同形異字による偽装を防止）

カスタムIDは`{#custom-id}`構文で指定します。指定がない場合は`heading-1`, `heading-2`のように自動採番されます。

### 強調表現

LukiWiki独自の視覚的強調：

```
''太字'' → <b>太字</b>
'''斜体''' → <i>斜体</i>
```

Markdownのセマンティック強調も利用可能：

```
**強調** → <strong>強調</strong>
*強調* → <em>強調</em>
```

### ブロック装飾

行頭にプレフィックスを配置：

```
COLOR(red): 赤い文字
SIZE(1.5): 大きな文字
RIGHT: 右寄せ
CENTER: 中央寄せ
```

### ブロック引用

LukiWiki形式（開始・終了タグ）：

```
> 引用文
> 複数行対応 <
```

Markdown形式（行頭プレフィックス）も使用可能：

```
> Markdownスタイルの引用
```

## ビルド

### 通常ビルド

```bash
cargo build --release
```

### WASM ビルド

```bash
wasm-pack build --target web
```

## テスト

```bash
cargo test
```

**テスト結果**: 112 tests passing

- 72 unit tests (including 5 frontmatter + 3 custom header ID tests)
- 18 CommonMark compliance tests
- 13 conflict resolution tests
- 9 doctests

## ライセンス

MIT License

## 作者

Masashi Yoshikawa

## リポジトリ

https://github.com/logue/LukiWiki-rs
