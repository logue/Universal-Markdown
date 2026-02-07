# 実装済み機能リファレンス

**最終更新**: 2026年2月7日

このドキュメントはUniversal Markdownで実装済みの機能を記載しています。

## 目次

- [基本Markdown機能](#基本markdown機能)
- [コメント構文](#コメント構文)
- [UMD拡張構文](#umd拡張構文)
- [プラグインシステム](#プラグインシステム)
- [テーブル機能](#テーブル機能)
- [その他の機能](#その他の機能)

---

## 基本Markdown機能

### 見出し

ATX形式の見出しをサポート:

```markdown
# 見出し1

## 見出し2

### 見出し3

#### 見出し4

##### 見出し5

###### 見出し6
```

### 段落と改行

段落は空行で区切ります。行末に2つのスペースまたは`\`を追加することで改行できます。

### フェンスコードブロック

````markdown
```rust
fn main() {
    println!("Hello, world!");
}
```
````

### リスト

**順序なしリスト**:

```markdown
- アイテム1
- アイテム2

* アイテム3
```

**順序付きリスト**:

```markdown
1. 最初
2. 次

- UMD形式もサポート
```

### リンクと画像

```markdown
[リンクテキスト](https://example.com)
![画像の代替テキスト](image.png)
```

### 強調

```markdown
_斜体_ または _斜体_
**太字** (Markdown標準)
```

**注意**: `__text__` はCommonMark標準では `<strong>` ですが、Universal Markdownでは Discord風に `<u>` (アンダーライン) に変換されます。

```markdown
**アンダーライン** → <u>アンダーライン</u> (Discord風)
```

---

## UMD拡張構文

### 強調構文

**UMD形式の強調** (視覚的な装飾):

```markdown
''太字'' → <b>太字</b>
'''斜体''' → <i>斜体</i>
**アンダーライン** → <u>アンダーライン</u> (Discord風)
```

**注**:

- Markdown形式（`*`, `**`）はセマンティックタグ（`<em>`, `<strong>`）を生成
- UMD形式は視覚的タグを生成
- `__text__` は Discord風にアンダーライン（`<u>`）として扱われます（CommonMark標準の`<strong>`ではありません）

### 取り消し線

```markdown
%%取り消し線%% → <s>取り消し線</s> (UMD: 視覚的)
~~取り消し線~~ → <del>取り消し線</del> (GFM: セマンティック)
```

---

## コメント構文

**実装日**: 2026年2月4日

プログラミング言語と同様のコメント構文。レンダリングされない開発用コメントをドキュメント内に記載できます。

### 単一行コメント

`//` から行末までがコメントとして扱われます。

```markdown
// この行は出力されません
これは表示される // ここは表示されない
```

**出力**:

```html
<p>これは表示される</p>
```

### 複数行コメント

`/*` から `*/` までがコメントとして扱われます。

```markdown
これは表示される
/_
このブロック内は
すべてコメント
_/
これも表示される
```

**出力**:

```html
<p>これは表示される</p>
<p>これも表示される</p>
```

### インラインコメント

行の途中にもコメントを配置できます。

```markdown
前部分/_ コメント _/後部分
```

**出力**:

```html
<p>前部分後部分</p>
```

### コードブロック内での扱い

コードブロック（` ``` `）、インラインコード（`` ` ``）内のコメント構文は**そのまま保持**されます。

````markdown
```rust
// Rustのコメント
fn main() {}
```
````

**出力**:

```html
<pre><code class="language-rust">// Rustのコメント
fn main() {}
</code></pre>
```

### URLとの区別

URLスキーム（`https://`等）の `//` はコメントとして扱われません。

```markdown
リンク: https://example.com/path
```

**出力**: URLはそのまま保持されます。

### 処理タイミング

コメント除去は**Conflict Resolver段階**（Markdown解析前）で実行されます。

### 利点

1. **開発用メモ**: ドキュメント内にTODOや注意事項を記載
2. **一時的な無効化**: 構文を削除せずにコメントアウトで無効化
3. **コード例の説明**: コメント構文を使って例示の説明を追加

---

## ブロック引用

**UMD形式** (開始・終了タグ):

```markdown
> 引用文
> 複数行の引用
> <
```

**Markdown標準形式**:

```markdown
> 引用文
> 複数行の引用
```

### ブロック装飾プレフィックス

行頭に配置して段落やブロックをスタイリング:

#### 配置

```markdown
LEFT: 左寄せテキスト → <p class="text-start">...</p>
CENTER: 中央寄せテキスト → <p class="text-center">...</p>
RIGHT: 右寄せテキスト → <p class="text-end">...</p>
```

#### 色指定

```markdown
COLOR(primary): 青いテキスト → Bootstrap text-primary
COLOR(,warning): 黄色の背景 → Bootstrap bg-warning
COLOR(danger,danger-subtle): 赤い強調テキスト → text-danger + bg-danger-subtle
COLOR(#FF0000): カスタム色 → インラインスタイル
```

**サポートされるBootstrap色**:

- テーマカラー: `primary`, `secondary`, `success`, `danger`, `warning`, `info`, `light`, `dark`
- グレースケール: `gray-100`～`gray-900`
- サブトル色: `primary-subtle`, `success-border-subtle`等

#### サイズ指定

```markdown
SIZE(2): 大きいテキスト → <p class="fs-2">...</p>
SIZE(1.5rem): カスタムサイズ → <p style="font-size: 1.5rem">...</p>
```

**Bootstrapクラスマッピング**:

- `2.5` → `fs-1` (2.5rem)
- `2` → `fs-2` (2rem)
- `1.75` → `fs-3`
- `1.5` → `fs-4`
- `1.25` → `fs-5`
- `0.875` → `fs-6`

#### 複合構文

複数のプレフィックスを組み合わせ可能:

```markdown
SIZE(1.5): COLOR(primary): CENTER: 強調テキスト
```

出力: `<p class="fs-4 text-primary text-center">強調テキスト</p>`

### インライン装飾関数

段落内でテキストをスタイリング:

#### 基本装飾

```markdown
&color(danger){エラー}; → <span class="text-danger">エラー</span>
&size(2){大きい文字}; → <span class="fs-2">大きい文字</span>
&badge(success){Active}; → <span class="badge bg-success">Active</span>
&badge(primary){[New](/new)}; → <a href="/new" class="badge bg-primary">New</a>
```

#### セマンティック要素

```markdown
&sup{上付き}; → <sup>上付き</sup>
&sub{下付き}; → <sub>下付き</sub>
&lang(en){Hello}; → <span lang="en">Hello</span>
&abbr(HTML){HyperText Markup Language}; → <abbr title="HyperText...">HTML</abbr>
&ruby(あした){明日}; → <ruby>明日<rp>(</rp><rt>あした</rt><rp>)</rp></ruby>
&kbd{Ctrl}; → <kbd>Ctrl</kbd>
&samp{output}; → <samp>output</samp>
&var{x}; → <var>x</var>
&cite{Book Title}; → <cite>Book Title</cite>
&q{Quote}; → <q>Quote</q>
&small{Small text}; → <small>Small text</small>
&u{Underline}; → <u>Underline</u>
&time(2026-01-26){今日}; → <time datetime="2026-01-26">今日</time>
&dfn{Definition}; → <dfn>Definition</dfn>
&bdi{Text}; → <bdi>Text</bdi>
&bdo(rtl){Text}; → <bdo dir="rtl">Text</bdo>
&wbr; → <wbr />
```

---

## プラグインシステム

Universal Markdownは拡張可能なプラグインシステムを提供します。

### プラグイン構文

#### インライン型

```markdown
&function(arg1,arg2){content}; 完全形
&function(args); 引数のみ
&function; 引数なし
```

#### ブロック型

```markdown
@function(args){{ multi-line content }} 複数行
@function(args){single-line content} 単行
@function(args) 引数のみ
@function() 引数なし
```

### HTML出力形式

プラグインは`<template>`タグと`<data>`要素で出力されます:

```html
<template class="umd-plugin umd-plugin-function">
  <data value="0">arg1</data>
  <data value="1">arg2</data>
  content
</template>
```

バックエンド（Nuxt/Laravel等）でこのHTMLをパースし、最終的なHTML出力を生成します。

---

## テーブル機能

### Markdown標準テーブル

GFM準拠のテーブル（ソート可能）:

```markdown
| Header1 | Header2 |
| ------- | ------- |
| Cell1   | Cell2   |
```

出力: `<table class="table">...</table>`

### UMDテーブル

セル連結対応の拡張テーブル:

```markdown
| Header1 |> | Header3 |
| Cell1 | Cell2 |
```

出力: `<table class="table umd-table">...</table>`

#### セル連結

**横方向連結 (colspan)**: `|>`

```markdown
| Header1 |> |
| Cell1 | Cell2 |
```

**縦方向連結 (rowspan)**: `|^`

```markdown
| Header1 | Header2 |
| Cell1 | Cell2 |
| |^ | Cell3 |
```

#### セル装飾

セル内でブロック装飾プレフィックスを使用可能:

```markdown
| RIGHT: 右寄せ | CENTER: 中央 |
| TOP: 上揃え | MIDDLE: 中央揃え |
| COLOR(primary): 青い文字 | SIZE(1.5): 大きい文字 |
```

---

## その他の機能

### フロントマター

YAMLまたはTOML形式のメタデータをサポート:

**YAML形式**:

```yaml
---
title: ページタイトル
author: 著者名
date: 2026-01-26
---
```

**TOML形式**:

```toml
+++
title = "ページタイトル"
author = "著者名"
date = 2026-01-26
+++
```

フロントマターはHTML出力から除外され、`ParseResult.frontmatter`で取得できます。

### フットノート

Markdown標準の脚注構文:

```markdown
本文中の参照[^1]です。

[^1]: 脚注の内容
```

出力: `<section class="footnotes">...</section>`

### カスタムヘッダーID

見出しに任意のIDを指定:

```markdown
# カスタムID付き見出し {#custom-id}
```

出力: `<h1 id="custom-id">カスタムID付き見出し</h1>`

指定がない場合は自動で`heading-1`, `heading-2`...と採番されます。

### GFM Callouts (アラート)

GitHub Flavored Markdown互換のアラート:

```markdown
> [!NOTE]
> 補足情報

> [!TIP]
> 役立つヒント

> [!IMPORTANT]
> 重要な情報

> [!WARNING]
> 警告

> [!CAUTION]
> 注意事項
```

それぞれBootstrapのalertクラスに変換されます。

---

## セキュリティ

### HTML入力制限

- 直接のHTML入力は**完全禁止**
- すべてのHTMLタグは自動エスケープ
- HTMLエンティティ（`&nbsp;`, `&lt;`等）のみ保持
- パーサー生成のHTMLのみ出力に使用

### URL Sanitization

以下のスキームをブロック:

- `javascript:` - XSS攻撃
- `data:` - Base64エンコードスクリプト
- `vbscript:` - VBScript実行
- `file:` - ローカルファイルアクセス

その他のスキーム（http, https, mailto, tel, カスタムアプリスキーム等）は許可されます。

---

## テスト結果

**総テスト数**: 184 tests passing

- 121 unit tests
- 22 bootstrap integration tests
- 18 CommonMark compliance tests
- 13 conflict resolution tests
- 10 other tests
