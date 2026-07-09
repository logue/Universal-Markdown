# インライン型プラグイン（実装済み）

**最終更新**: 2026年7月9日

この文書は、UMD で現在実装済みの「インライン型プラグイン」形式を、Markdown の構文シュガーとは分けて一覧化したものです。

> ここで扱うのは `&...` で始まる明示的なプラグイン構文です。`**hoge**` や `__hoge__` のような Markdown 系のシンタックスシュガーは含めません。

## 主要な構文

### 文字装飾・見た目

- `&color(fg,bg){text};`
  - 文字色・背景色を指定するインライン装飾です。
  - `fg` / `bg` は Bootstrap の色名または HEX 形式を受け付けます。
- `&size(value){text};`
  - 文字サイズを指定するインライン装飾です。
  - Bootstrap の `fs-*` クラスまたはインラインスタイルに変換されます。
- `&badge(type){text};`
  - バッジ表示用のインライン装飾です。
  - Bootstrap の `badge` クラスに変換されます。

### 置換・要素化

- `&sup(text);`
  - `<sup>` で囲みます。
- `&sub(text);`
  - `<sub>` で囲みます。
- `&lang(locale){text};`
  - `<span lang="...">` で囲みます。
- `&abbr(text){description};`
  - `<abbr title="...">` で囲みます。
- `&ruby(reading){text};`
  - `<ruby>` で囲み、ルビ表示を行います。
- `&spoiler(text);`
  - `&spoiler{text};`
  - `<span class="spoiler">` で囲みます。

### セマンティック HTML

- `&dfn(text);`
  - `<dfn>` で囲みます。
- `&kbd(text);`
  - `<kbd>` で囲みます。
- `&samp(text);`
  - `<samp>` で囲みます。
- `&var(text);`
  - `<var>` で囲みます。
- `&cite(text);`
  - `<cite>` で囲みます。
- `&q(text);`
  - `<q>` で囲みます。
- `&small(text);`
  - `<small>` で囲みます。
- `&time(datetime){text};`
  - `<time datetime="...">` で囲みます。
- `&data(value){text};`
  - `<data value="...">` で囲みます。
- `&bdi(text);`
  - `<bdi>` で囲みます。
- `&bdo(dir){text};`
  - `<bdo dir="...">` で囲みます。

### 改行・分割

- `&wbr;`
  - `<wbr>` を挿入します。
- `&br;`
  - `<br />` を挿入します。

## 形式の傾向

実装済みのインライン構文は、次の 3 パターンに整理できます。

- `&function(args){content};`
- `&function(args);`
- `&function;`

ただし、この文書では実際に現在の実装で処理対象となっているものを中心に列挙しています。

## 対象外

次のようなものは、この文書の対象外です。

- `**bold**` のような Markdown の強調構文
- `__underline__` のような underline 系シンタックスシュガー
- `%%text%%` や `||text||` のような別系統のショートハンド
