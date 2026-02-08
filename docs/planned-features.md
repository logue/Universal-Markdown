# 実装予定機能リファレンス

**最終更新**: 2026年2月8日

このドキュメントは実装予定だが、まだ実装されていない機能を記載しています。

## 目次

- [メディアファイル自動検出](#メディアファイル自動検出)
- [ブロック装飾の追加機能](#ブロック装飾の追加機能)
- [テーブル拡張](#テーブル拡張)
- [Markdown拡張機能](#markdown拡張機能)
- [高度なUMD機能](#高度なumd機能)

---

## メディアファイル自動検出

### 概要

画像リンク構文 `![alt](url)` を拡張し、URLの拡張子に基づいて自動的に適切なHTMLメディアタグに変換します。

### 設計方針: HTML5メディアタグ統一

- 全てのメディアを最新のHTML5タグで処理（古いブラウザは考慮しない）
- 画像も`<picture>`タグで出力し、将来的な拡張性を確保
- `<video>`, `<audio>`, `<picture>`で統一的なメディア処理を実現

### 動画ファイル

**対応拡張子** (大文字小文字区別なし):
`.mp4`, `.webm`, `.ogv`, `.ogg`, `.mov`, `.avi`, `.mkv`, `.m4v`

**出力HTML**:

```html
<video controls>
  <source src="url" type="video/ext" />
  <track kind="captions" label="alt" />
  お使いのブラウザは動画タグをサポートしていません。
</video>
```

**特徴**:

- `<source>`タグで明示的にMIMEタイプを指定
- `alt`テキストは`<track>`タグのキャプションラベルとして使用
- `controls`属性をデフォルトで追加

**使用例**:

```markdown
![プレゼンテーション](video.mp4)
```

### 音声ファイル

**対応拡張子** (大文字小文字区別なし):
`.mp3`, `.wav`, `.ogg`, `.oga`, `.m4a`, `.aac`, `.flac`, `.opus`, `.weba`

**出力HTML**:

```html
<audio controls>
  <source src="url" type="audio/ext" />
  お使いのブラウザは音声タグをサポートしていません。
</audio>
```

**特徴**:

- `<source>`タグで明示的にMIMEタイプを指定
- `controls`属性をデフォルトで追加

**使用例**:

```markdown
![BGM](audio.mp3)
```

### 画像ファイル

**対応拡張子**:
`.jpg`, `.jpeg`, `.png`, `.gif`, `.svg`, `.webp`, `.avif`, `.bmp`, `.ico` 等

**出力HTML**:

```html
<picture>
  <source srcset="url" type="image/ext" />
  <img src="url" alt="alt" loading="lazy" />
</picture>
```

**特徴**:

- `<picture>`タグで統一的に出力
- `<source>`タグで明示的にMIMEタイプを指定
- `<img>`タグはフォールバック兼アクセシビリティ対応
- `loading="lazy"`属性を自動追加（遅延読み込み）

**インライン表示への配慮**:

- `<picture>`タグは`inline`要素として扱われる
- 段落内での使用が可能: `テキスト![画像](image.png)テキスト`
- CSSでブロック化も可能: `.picture { display: block; }`

**使用例**:

```markdown
![ロゴ](logo.png)
![最新画像](image.avif)
```

### MIMEタイプマッピング

#### 動画

- mp4→video/mp4, webm→video/webm, ogv→video/ogg, ogg→video/ogg
- mov→video/quicktime, avi→video/x-msvideo, mkv→video/x-matroska, m4v→video/x-m4v

#### 音声

- mp3→audio/mpeg, wav→audio/wav, ogg→audio/ogg, oga→audio/ogg
- m4a→audio/mp4, aac→audio/aac, flac→audio/flac, opus→audio/opus, weba→audio/webm

#### 画像

- jpg/jpeg→image/jpeg, png→image/png, gif→image/gif, svg→image/svg+xml
- webp→image/webp, avif→image/avif, bmp→image/bmp, ico→image/x-icon

### 実装方針

1. comrakのAST後処理で`Image`ノードを検出
2. URLの拡張子を正規表現で解析（クエリパラメータを除外）
3. 拡張子に応じて適切なHTMLタグとMIMEタイプを生成
4. 拡張子なし、または未知の拡張子: 画像として扱う（`<picture>`+`image/octet-stream`）

### セキュリティ

URL sanitizationは既存のsanitizer.rsで処理。以下のスキームをブロック:

- `javascript:` - JavaScript実行
- `data:` - Base64エンコードされたスクリプト
- `vbscript:` - VBScript実行
- `file:` - ローカルファイルアクセス

### 拡張オプション（将来的な検討事項）

`@media`プラグインで追加属性・複数ソースを指定可能にする:

**ブロック型**:

```markdown
@media(autoplay,loop,muted){![動画](video.mp4)}
@media(preload=metadata){![音声](audio.mp3)}
@media(mobile.jpg,tablet.jpg,desktop.jpg){![レスポンシブ](desktop.jpg)}
@media(avif,webp,fallback.png){![最新画像](image.avif)}
```

**インライン型**:

```markdown
テキスト&media(width=100){![アイコン](icon.png)};テキスト
説明文&media(autoplay,muted){![デモ](demo.mp4)};説明文
```

---

## ブロック装飾の追加機能

### JUSTIFY プレフィックス

#### テキストの両端揃え

```markdown
JUSTIFY: この文章は両端揃えで表示されます。
```

出力: `<p class="text-justify">この文章は両端揃えで表示されます。</p>`

**注**: Bootstrap 5では`text-justify`が非推奨だが、UMDでは明示的に対応。

#### UMDテーブルの幅指定

UMDテーブル（区切り行なし）の前に`JUSTIFY:`がある場合、テーブル全体の幅を100%に維持:

```markdown
JUSTIFY:
| Header1 | Header2 |
| Cell1 | Cell2 |
```

出力: `<table class="table umd-table">...</table>`

### UMDブロック要素の配置まとめ

適用対象: UMDテーブル（区切り行なし）、ブロック型プラグイン（`@function(...)`）

- `LEFT:`（改行）`<block>` → `w-auto`（コンテンツ幅、左寄せ）
- `CENTER:`（改行）`<block>` → `w-auto mx-auto`（コンテンツ幅、中央寄せ）
- `RIGHT:`（改行）`<block>` → `w-auto ms-auto me-0`（コンテンツ幅、右寄せ）
- `JUSTIFY:`（改行）`<block>` → `w-100`（100%幅）

**例（ブロック型プラグイン）**:

```markdown
CENTER:
@callout(info)
```

出力: `<div class="umd-plugin umd-plugin-callout w-auto mx-auto" ...></div>`

### TRUNCATE プレフィックス

テキスト省略:

```markdown
TRUNCATE: 長いテキストは省略されます...
```

出力: `<p class="text-truncate">長いテキストは省略されます...</p>`

**特徴**:

- `overflow: hidden; text-overflow: ellipsis; white-space: nowrap`を適用
- 幅指定はユーザーがCSSで指定する前提
- テーブルセルでは自動適用される

---

## テーブル拡張

### テーブルバリエーション

プラグインシステムで実装予定:

```markdown
@table(striped,hover){{
| Header1 | Header2 |
| Cell1   | Cell2   |
}}
```

**サポート予定のオプション**:

- `striped` → `table-striped`（縞模様）
- `hover` → `table-hover`（ホバー効果）
- `dark` → `table-dark`（ダークモード）
- `bordered` → `table-bordered`（ボーダー）
- `borderless` → `table-borderless`（ボーダーなし）
- `sm` → `table-sm`（コンパクト）

### レスポンシブ対応

```markdown
@table(responsive){{
| Header1 | Header2 |
| Cell1   | Cell2   |
}}
```

出力: `<div class="table-responsive"><table>...</table></div>`

### ブロック引用のデフォルトクラス

```html
<blockquote class="blockquote">...</blockquote>
```

Bootstrap標準の引用スタイルを自動適用。

---

## Markdown拡張機能

### Setext見出し

下線形式の見出し:

```markdown
# 見出し1

## 見出し2
```

### 参照スタイルリンク

```markdown
[テキスト][ref]

[ref]: https://example.com
```

### バックスラッシュエスケープ

```markdown
\*リテラルアスタリスク\*
```

### 自動URL検出

```markdown
http://example.com
```

自動的にリンク化されます。

---

## 高度なUMD機能

### リスト内ブロック要素

リスト項目内にテーブル、コードブロック等を配置:

```markdown
- リスト項目

  | Header |
  | ------ |
  | Cell   |
```

**実装方針**:

- インデント解析による親子関係判定
- CommonMark違反だが互換性のため必須

### タスクリスト拡張

```markdown
- [ ] 未完了タスク
- [x] 完了タスク
- [-] 不確定状態（UMD拡張）
```

### カスタムリンク属性

```markdown
[テキスト](url){id class}
```

リンクに任意の属性を追加。

### 添付ファイル構文

```markdown
PageName/FileName
```

### 相対パス

```markdown
./page
../page
/page
```

---

## 未実装機能（提案段階）

以下の機能は仕様書で提案されているが、MVP後の追加機能として保留:

- **ラジオボタン**: `( )`, `(x)`
- **トグルボタン**: `< >`, `<x>`
- **絵文字**: `::emoji_name::`
- **画像リンク**: `[![alt](image)](link)`

これらは需要と仕様確定後に実装を検討します。
