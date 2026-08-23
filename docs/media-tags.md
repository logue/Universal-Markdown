# メディアタグ自動検出

**最終更新**: 2026年8月23日

画像記法 `![alt](url)` から、拡張子ベースでメディア HTML に変換する仕様です。

## 概要

- 実装: `src/extensions/media.rs`
- 適用フェーズ: 拡張処理（HTML 後処理）
- 主な入力: `<img src="..." alt="..." title="..." />`
- 主なテスト: `src/extensions/media.rs` 内テスト、`tests/bootstrap_integration.rs`

## 変換先

### 動画

対象拡張子:
`.mp4`, `.webm`, `.ogv`, `.mov`, `.avi`, `.mkv`, `.m4v`

出力:

```html
<video controls title="...">
  <source src="..." type="video/..." />
  <track kind="captions" label="..." />
  <a href="..." download class="download-link video-fallback">...</a>
</video>
```

### 音声

対象拡張子:
`.mp3`, `.wav`, `.ogg`, `.oga`, `.m4a`, `.aac`, `.flac`, `.opus`, `.weba`

出力:

```html
<audio controls title="...">
  <source src="..." type="audio/..." />
  <a href="..." download class="download-link audio-fallback">...</a>
</audio>
```

### 画像

対象拡張子:
`.jpg`, `.jpeg`, `.png`, `.gif`, `.svg`, `.webp`, `.avif`, `.bmp`, `.ico`, `.jxl`, `.tif`, `.tiff`

出力:

```html
<picture title="...">
  <source srcset="..." type="image/..." />
  <img src="..." alt="..." loading="lazy" class="img-fluid" title="..." />
</picture>
```

### ダウンロードリンク

対象拡張子:

- アーカイブ: `.zip`, `.tar`, `.gz`, `.7z`, `.rar`, `.bz2`, `.xz`
- ドキュメント: `.pdf`, `.doc`, `.docx`, `.xls`, `.xlsx`, `.ppt`, `.pptx`, `.odt`, `.ods`, `.odp`
- テキスト系: `.txt`, `.md`, `.csv`, `.json`, `.xml`, `.yaml`, `.yml`, `.toml`
- 実行ファイル系: `.exe`, `.dmg`, `.deb`, `.rpm`, `.app`, `.apk`, `.msi`

出力:

```html
<a href="..." download class="download-link" title="...">...</a>
```

## 拡張パラメータ（`{}` 構文）

画像記法の URL の直後に `{...}` ブロックを付加することで、サイズと整合性ハッシュを指定できます。

```
![alt](url){パラメータ, ...}
```

パラメータはカンマ区切りで複数指定できます。順序は問いません。

### トークン種別と字句規則

`{}` 内のトークンは字句的に型が決まります。文法の曖昧さはありません。

| トークン形式      | 種別                | 例                 |
| ----------------- | ------------------- | ------------------ |
| `[0-9]+%`         | 幅（パーセント）    | `75%`              |
| `[0-9]+x[0-9]+`   | 幅×高さ（ピクセル） | `320x240`          |
| `[0-9]+`          | 幅（ピクセル）      | `320`              |
| `sha256-<base64>` | SRI ハッシュ        | `sha256-abc123...` |
| `sha384-<base64>` | SRI ハッシュ        | `sha384-def456...` |
| `sha512-<base64>` | SRI ハッシュ        | `sha512-ghi789...` |

- `sha256-`・`sha384-`・`sha512-` で始まるトークン → 整合性ハッシュ（SRI）
- 数値・パーセント・`x` 区切りのトークン → サイズ指定
- サイズ指定は最大 1 つ。SRI ハッシュは複数指定可能。

### サイズ指定

```markdown
![動画](movie.mp4){75%}
![画像](photo.jpg){320x240}
![音声](track.mp3){400}
```

出力への影響:

- パーセント・ピクセル幅のみ → `style="width: 75%"` または `width="320"` を外側要素に付加
- `幅x高さ` → `width="320" height="240"` を外側要素に付加
- 動画・音声・`<picture>` の直接の親要素（`<video>`, `<audio>`, `<picture>`）に適用

### SRI（Subresource Integrity）

```markdown
![動画](movie.mp4){sha256-abc123==, sha384-def456==}
![音声](track.mp3){sha512-ghi789==}
![画像](photo.png){sha256-abc123==}
```

出力への影響:

- `<source>` タグに `integrity="sha256-abc123== sha384-def456=="` を付加（スペース区切りで結合）
- フォールバックの `<a download>` タグにも同じ `integrity` 属性を付加
- `<picture>` の `<source>` にも同様に付加

動画の出力例:

```html
<video controls title="...">
  <source
    src="movie.mp4"
    type="video/mp4"
    integrity="sha256-abc123== sha384-def456=="
  />
  <track kind="captions" label="..." />
  <a
    href="movie.mp4"
    download
    class="download-link video-fallback"
    integrity="sha256-abc123== sha384-def456=="
    >...</a
  >
</video>
```

### サイズと SRI の組み合わせ

```markdown
![動画](movie.mp4){75%, sha256-abc123==, sha384-def456==}
```

出力例:

```html
<video controls title="..." style="width: 75%">
  <source
    src="movie.mp4"
    type="video/mp4"
    integrity="sha256-abc123== sha384-def456=="
  />
  <track kind="captions" label="..." />
  <a
    href="movie.mp4"
    download
    class="download-link video-fallback"
    integrity="sha256-abc123== sha384-def456=="
  >
    ...
  </a>
</video>
```

### メディア種別ごとの適用範囲

| メディア種別 | サイズ             | SRI（`<source>`） | SRI（`<a download>`） |
| ------------ | ------------------ | ----------------- | --------------------- |
| 動画         | `<video>` に付加   | ✅                | ✅                    |
| 音声         | `<audio>` に付加   | ✅                | ✅                    |
| 画像         | `<picture>` に付加 | ✅                | —                     |
| ダウンロード | `<a>` に付加       | —                 | ✅                    |

### 不正トークンの扱い

- 認識できないトークンは無視し、警告は出力しません。
- サイズトークンが複数あった場合、最初のものを採用します。

---

## 表示ルール

- 段落がメディア要素だけで構成される場合、`<figure class="w-100">...</figure>` にラップします。
- 段落内のインラインメディアはそのままインライン扱いです。

## オプション

### `allow_fragment_extension_hint`

`ParserOptions.allow_fragment_extension_hint`（既定: `false`）を有効にすると、拡張子なし URL の `#.png` のようなフラグメントヒントをメディア判定に使います。

例:

- `/assets/image#.png` → 画像として判定（`true` の場合）
- `/assets/media#t=10` → 拡張子ヒントではないため判定しない

### `icons`

`ParserOptions.icons` で、動画・音声・ダウンロード・カラーサンプルのアイコン HTML を差し替え可能です。

既定値は Bootstrap Icons ベースです。

## 補足

- クエリ文字列とフラグメントを除いたパス拡張子で判定します（ヒント有効時を除く）。
- 未知拡張子はメディア変換せず、通常の画像 `<picture><img ...></picture>` として扱います。
- alt が空のときは、フォールバックリンクの表示文字に URL を使用します。
