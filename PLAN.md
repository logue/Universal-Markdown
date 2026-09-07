# Universal Markdown (UMD) 実装プラン

**プロジェクト概要**: Markdownを超える次世代マークアップ言語。CommonMark仕様テスト 75%+ パス、Bootstrap 5統合、セマンティックHTML、拡張可能なプラグインシステム提供。

**作成日**: 2026年1月23日  
**最終更新**: 2026年9月07日  
**Rustバージョン**: 1.98.1 (Edition 2024)  
**ライセンス**: Apache-2.0

### 主要ライブラリ

| ライブラリ          | バージョン | 用途                                     |
| ------------------- | ---------- | ---------------------------------------- |
| ammonia             | 4.1.4      | HTML サニタイズ                          |
| comrak              | 0.55.0     | Markdown パーサー (GFM)                  |
| math-core           | 0.8.2      | LaTeX to MathML 変換                     |
| maud                | 0.27.0     | 型安全 HTML 生成                         |
| mermaid-rs-renderer | 0.3.1      | Mermaid SSR レンダリング                 |
| regex               | 1.13.1     | パターンマッチング                       |
| syntect             | 5.3.0      | シンタックスハイライト（ネイティブのみ） |
| wasm-bindgen        | 0.2.127    | WASM バインディング                      |

---

## ドキュメント構成

このPLAN.mdは**実装状況とロードマップ**を記載しています。詳細は以下を参照：

- **[docs/README.md](docs/README.md)** - ドキュメント索引（入口）
- **[docs/architecture.md](docs/architecture.md)** - システムアーキテクチャ、開発者ガイド、技術設計
- **[docs/implemented-features.md](docs/implemented-features.md)** - 実装済み機能の完全リファレンス
- **[docs/planned-features.md](docs/planned-features.md)** - 実装予定機能の詳細仕様
- **[README.md](README.md)** - プロジェクト概要とユーザー向けドキュメント

---

## プロジェクト現状サマリー

### ✅ 達成済み

- ✅ ビルドスクリプトのメタデータ正規化処理を改善
- ✅ ドキュメントを最新の状態に更新
- ✅ CommonMark 75%+ 準拠達成
- ✅ Bootstrap 5 統合（Core UI互換）
- ✅ セマンティックHTML生成
- ✅ メディア自動検出（動画・音声・画像・ダウンロード）
- ✅ プラグインシステム（インライン & ブロック型）
- ✅ テーブル拡張（セル連結、配置、装飾）
- ✅ UMD独自構文全体実装
- ✅ 数式サポート（`&math(...)` / `@math(...)`）
- ✅ ポップオーバー（`&popover` / `@popover`）
- ✅ インラインコード色サンプル（HEX/RGB/RGBA/HSL/HSLA）
- ✅ セキュリティ対策完備（XSS/URL sanitization）
- ✅ ブロック型プラグイン書式の拡張（`::: 記法`）

### 🚧 進行中

- 🚧 WASM サイズ最適化と配布改善
- 🚧 Mermaid レンダリングのキャッシュ最適化

### 🔮 計画中

- 🔮 Bootstrap依存の削減（CSS Layerの活用による脱Bootstrap化）
- 🔮 リファレンスCSS提供（スタイル定義の標準化）
- 🔮 テンプレートエンジン機能の検討と仕様策定
- 🔮 フロントエンド向けのシンタックスハイライト改善
- 🔮 テキスト装飾記法の追加（`^^` / `~~` / `==` / `&outline()`）
- 🔮 挿入・削除記法（`{+ +}` / `{- -}`）の追加
- 🔮 パーサーレベルの定義（チャット運用時、コメント運用時、ドキュメント作成時）
- 🔮 リンク・画像の `integrity` 属性対応（SRI）
- 🔮 フロントマターのTSON対応（区切り文字 `***`）
- 🔮 ボトムマター仕様策定
- 🔮 AAプラグイン（決め打ちフォント指定によるアスキーアート表示、例: MS Pゴシック）— 文字幅依存が強くリファレンスCSS/コアの責務にできないためプラグインとして分離
- 🔮 Mermaid SVGの色トークン対応 — `mermaid-rs-renderer`は各要素に`fill="#hex"`等のリテラル色を焼き込むため、`.umd-color-*`のようなCSSクラスでは上書きできない。現状は`src/extensions/code_block.rs`の`inject_bootstrap_colors`がBootstrap既定6色のHEXのみ`var(--bs-*, #hex)`に後置換する場当たり的な対応。恒久対応は (1) この置換をUMDの色トークン・全色相に拡張するか、(2) `mermaid-rs-renderer`のTheme/ThemeVariables設定に`var(...)`文字列を直接渡してレンダリングさせる（SVGシリアライザが素通しするか要検証）

---

## ブロック型プラグイン書式仕様（`::: 記法`）

### 概要

QiitaやGrowiで採用されている書式を新たにサポートします。

```markdown
:::function args
content
:::
```

### 基本ルール

1. **開始**: `:::function args` で開始（`function` はプラグイン名、`args` はオプション引数）
2. **終了**: `:::` で終了
3. **入れ子構造**: **非サポート** - 入れ子構造内のプラグインマークアップは無視またはエラーとして扱う

### 非対応パターン（入れ子禁止）

以下のパターンはいずれも入れ子を含むため非対応です：

```umd
❌ ブロック型プラグイン内にブロック型プラグイン
:::function args
:::function2 args2
content
:::
:::

❌ ブロック型プラグイン内にインライン型プラグイン
:::function args
@function2(args2){{
    content
  }}
:::

❌ インライン型プラグイン内にブロック型プラグイン
@function(args){{
  :::function2 args2
    content
  :::
}}
```

### 実装計画

- [x] `::: 記法` の字句解析・構文解析実装（`src/extensions/plugin_markers.rs` の `protect_colon_block_plugins`）
- [x] 既存プラグインシステムとの統合（`@table` / `@math` / `@popover` / `@clear` と同一の後処理ロジックを共有、`src/extensions/conflict_resolver.rs`）
- [x] 入れ子検出・エラーハンドリング実装（最初に現れる `:::` のみの行で閉じることで、入れ子部分は生テキストとして無害化）
- [x] テスト suite 追加（`plugin_markers` 単体テスト、`tests/conflict_resolution.rs` の統合テスト）
- [x] ドキュメント更新（[docs/plugin-system.md](docs/plugin-system.md) / [docs/block-plugins.md](docs/block-plugins.md)）

---

## テキスト装飾記法

### 概要（テキスト装飾）

CommonMark の `~~strikethrough~~` を廃止し、テキスト装飾記法を再設計します。各記号がCSS `text-decoration` の値に1対1で対応する「文法の直交性」を重視した設計です。

| 記法         | 出力要素                           | CSS                                   | 備考                                   |
| ------------ | ---------------------------------- | ------------------------------------- | -------------------------------------- |
| `__text__`   | `<u>`                              | `text-decoration: underline`          | Discord互換                            |
| `^^text^^`   | `<span class="umd-overline">`      | `text-decoration: overline`           | `^`は「上」のニーモニック、ASCII範囲内 |
| `~~text~~`   | `<span class="umd-wavy">`          | `text-decoration: underline wavy`     | `~~`の形が波線に対応                   |
| `==text==`   | `<span class="umd-overunderline">` | `text-decoration: underline overline` | 上下ライン                             |
| `{- text -}` | `<del>`                            | ブラウザデフォルト                    | diff記法の`-`に対応、スペース必須      |

> ⚠️ CommonMarkの `~~strikethrough~~` との非互換あり。UMDは意図的にこの仕様を変更しています。

### 袋文字プラグイン（`&outline()`）

背景色変更の代替として、より視覚的に目立つテキスト装飾手段を提供します。

```umd
&outline(){text}                 ← デフォルト（白文字・黒縁）
&outline(red){text}              ← fill=red、stroke=デフォルト
&outline(white, blue){text}      ← fill=white、stroke=blue
```

**パラメータ**:

- `fill`（第1引数）: 文字の塗り色。デフォルト = `--umd-body-bg`（背景色）
- `stroke`（第2引数）: 輪郭色。デフォルト = `--umd-body-color`（前景色）

**実装**: `-webkit-text-stroke` + `paint-order: stroke fill`（`text-shadow` 多重描画は使用しない）

```html
<!-- &outline(){text} -->
<span class="umd-outline">text</span>

<!-- &outline(white, blue){text} -->
<span
  class="umd-outline"
  style="--umd-outline-fill: white; --umd-outline-stroke: blue;"
  >text</span
>
```

### 実装計画（テキスト装飾）

- [ ] `^^`、`~~`（波線）、`==` の字句解析・構文解析実装
- [ ] 対応CSSクラスの生成（`umd-overline` / `umd-wavy` / `umd-overunderline`）
- [ ] `&outline()` プラグイン実装（CSS変数インライン注入）
- [x] `components/outline.scss` 追加済み
- [ ] テスト suite 追加
- [ ] ドキュメント更新

---

## 挿入・削除記法仕様（`{+ +}` / `{- -}`）

### 概要（挿入・削除記法）

`<ins>` / `<del>` に対応するインライン記法を追加します。

```umd
{+ 追加されたテキスト +}
{- 削除されたテキスト -}
```

### 基本ルール（挿入・削除記法）

1. **挿入**: `{+ text +}` → `<ins>text</ins>`
2. **削除**: `{- text -}` → `<del>text</del>`
3. **区切り記号に隣接する半角スペースは必須**: `{+`/`{-` の直後、および `+}`/`-}`
   の直前に半角スペースが1つ必要。スペースがない表記（例: `{+hoge+}`、
   `{-hoge-}`）は本記法として処理せず、プレーンテキストのまま出力する。
4. **前後スペースの除去**: 変換後のHTMLには区切り記号に隣接する1つのスペース
   を含めない（例: `{- bar -}` → `<del>bar</del>`）。テキスト内部の空白は
   そのまま保持する。
5. インライン記法（`&function;` 系と同様、複数行content や入れ子には対応しない）

### 実装計画（挿入・削除記法）

- [ ] `{+ +}` / `{- -}` の字句解析・構文解析実装
- [ ] 前後スペース必須ルールのバリデーション実装
- [ ] 既存のインライン装飾・プラグイン記法（`{id class}` 属性記法や `{{ }}`
      ブロックプラグインマーカー）との衝突確認
- [ ] テスト suite 追加
- [ ] ドキュメント更新

## パーサーレベルの定義

### 概要

利用コンテキストに応じて、許容する構文の幅を分けるための考え方です。

- Chat
- Comment（フォーラムや記事のコメント欄など）
- Document（フル仕様）

### 現状の前提

現時点で実装しているコードには、このレベルの概念はまだ存在しておらず、表の意味では「Document」として扱います。

### 基本ルール

| 機能                            | Chat | Comment | Document |
| ------------------------------- | ---- | ------- | -------- |
| 太字・斜体・取り消し線          | ✅   | ✅      | ✅       |
| 下線（`__`）                    | ✅   | ✅      | ✅       |
| スポイラー                      | ✅   | ✅      | ✅       |
| コードブロック                  | ✅   | ✅      | ✅       |
| メンション（`@user`）           | ✅   | ✅      | ✅       |
| 文字色（プリセットカラーのみ）  | ✅   | ✅      | ✅       |
| 数式（`&math();`）              | ❌   | ✅      | ✅       |
| 文字色（HEXによる指定）         | ❌   | ❌      | ✅       |
| テーブル（Markdown）            | ❌   | ✅      | ✅       |
| Mermaid                         | ❌   | ✅      | ✅       |
| コメント（`//`等）              | ❌   | ❌      | ✅       |
| 文字サイズ                      | ❌   | ❌      | ✅       |
| レイアウト（`START:`/`END:`等） | ❌   | ❌      | ✅       |
| テーブル（PukiWiki形式）        | ❌   | ❌      | ✅       |
| ブロックプラグイン              | ❌   | ❌      | ✅       |

### 検討事項

- 色の仕様はまだまとまっておらず、レベルとは関係なく、オプションでプリセットカラー（現時点では Bootstrap の色）だけを許可するか、HEX・RGBA・HSL なども許可するかを検討中です。
- 文字サイズは `&size(xs/sm/lg/xl){text}` の記法で確定。CSSキーワード値（`x-small` / `small` / `large` / `x-large`）のみ使用し、ピクセル指定はオプション（デフォルト無効）。
- 標準プラグイン以外のプラグイン使用可否は、このライブラリを使用するホストプログラムの責務とし、このレベルには定義を含めません。

---

## リンク・画像の `integrity` 属性対応

### 概要（`integrity` 属性）

Subresource Integrity (SRI) 相当のハッシュ検証をリンク・画像に付与するための属性拡張です。

```umd
[リンク](url){integrity=sha256:x9y8z7..., class=...};
![画像](url){50%, integirity=sha256:x9y8z7...}
```

### 基本ルール（`integrity` 属性）

1. 既存の属性記法 `{key=value, ...}` に `integrity` キーを追加
2. 値は `アルゴリズム名:ハッシュ値` 形式（例: `sha256:x9y8z7...`）を想定 — 標準SRI仕様の `sha256-...`（ハイフン区切り）との対応関係は要検討
3. `class` など既存の属性キーと併記可能（カンマ区切り）
4. 出力HTMLでは `integrity` 属性（および必要に応じ `crossorigin` 属性）としてそのまま付与

### 検討事項（`integrity` 属性）

- 区切り文字を `:`（本仕様案）にするか、標準SRIに合わせて `-` にするかは未確定
- 対応アルゴリズム（`sha256` / `sha384` / `sha512`）の範囲
- 値のフォーマットバリデーション（Base64判定）の要否
- クロスオリジンリソースのみ許可するか、ローカルリソースも含めるか

### 実装計画（`integrity` 属性）

- [ ] 属性パーサへの `integrity` キー追加
- [ ] 値フォーマットの確定・バリデーション実装
- [ ] HTML出力時の `integrity` / `crossorigin` 属性付与
- [ ] テスト suite 追加
- [ ] ドキュメント更新

---

## フロントマターのTSON対応

### 概要（TSONフロントマター）

型注釈付きJSON（TSON）形式のフロントマターをサポートします。区切り文字は `***` とします。

```umd
***
{
  "id": int 101,
  "name": string "Alice",
  "isActive": bool true
}
***
```

### 基本ルール（TSONフロントマター）

1. 文書先頭の `***` 行から次の `***` 行までをフロントマターとして扱う
2. 中身はTSON形式（キーごとに型名 + 値を明記するJSON類似構文）
3. 既存のYAML/TOML等のフロントマター記法とは別記法として扱う（区切り文字で判別）
4. パース結果はメタデータとしてホスト側に渡す（用途は既存フロントマターと同様）

### 検討事項（TSONフロントマター）

- TSONの正式な文法（対応する型の一覧、配列・ネストオブジェクトの扱い）の確定
- 既存のフロントマター（YAML `---` 等）との共存可否
- 型注釈と実際の値の不一致時のエラーハンドリング

### 実装計画（TSONフロントマター）

- [ ] TSON文法の確定
- [ ] `***` 区切りの検出・パース実装
- [ ] 型注釈の検証ロジック実装
- [ ] テスト suite 追加
- [ ] ドキュメント更新

---

## ボトムマター仕様

### 概要（ボトムマター）

フロントマターのボトム版。文書末尾に付与するメタデータブロックです。現時点ではJSON形式で出力する以外の仕様は未確定です。

### 検討事項（ボトムマター）

- 区切り文字（フロントマターと同じ `***` を使うか、別の記号にするか）
- 記述フォーマット（TSON形式にするか、通常のJSONにするか）
- フロントマターとの併用可否・優先順位
- 用途（ページ末尾のメタ情報、コメント欄への引き渡し情報など）の具体化

### 実装計画（ボトムマター）

- [ ] 仕様策定（区切り文字・フォーマット確定）
- [ ] パーサー実装
- [ ] テスト suite 追加
- [ ] ドキュメント更新

---

## Bootstrap依存の削減とリファレンスCSS

### 概要（脱Bootstrap化）

現在Bootstrap 5のユーティリティクラス（`d-block`、`text-primary`など）に依存して出力しているHTMLを、Bootstrap本体への依存から切り離し、CSS Layer（`@layer`）を使ったミニマムなリファレンスCSSで置き換える計画です。

### クラス名改名一覧（2026年8月実施済み）

**display.scss**

| 旧                | 新                   | CSS                                      |
| ----------------- | -------------------- | ---------------------------------------- |
| `.d-block`        | `.umd-block`         | `display: block`                         |
| `.d-inline-block` | `.umd-inline-block`  | `display: inline-block`                  |
| `.d-none`         | `.umd-hidden`        | `display: none`                          |
| `.w-100`          | `.umd-block-justify` | `inline-size: 100%`                      |
| `.w-auto`         | `.umd-block-center`  | `inline-size: auto; margin-inline: auto` |

**text.scss**

| 旧             | 新                           | CSS                                      |
| -------------- | ---------------------------- | ---------------------------------------- |
| `.text-center` | `.umd-center`                | `text-align: center`                     |
| `.text-end`    | `.umd-end`                   | `text-align: end`                        |
| （新規）       | `.umd-start`                 | `text-align: start`                      |
| （新規）       | `.umd-justify`               | `text-align: justify`                    |
| （新規）       | `.umd-v-start/center/end`    | `vertical-align: top/middle/bottom`      |
| （新規）       | `.umd-text-size-xs/sm/lg/xl` | `font-size: x-small/small/large/x-large` |
| `.fs-4`        | （削除）                     | —                                        |

**spacing.scss**

| 旧         | 新                   | CSS                         |
| ---------- | -------------------- | --------------------------- |
| `.mx-auto` | `.umd-inline-center` | `margin-inline: auto`       |
| `.ms-auto` | `.umd-block-end`     | `margin-inline-start: auto` |
| `.me-auto` | `.umd-block-start`   | `margin-inline-end: auto`   |
| `.me-0`    | （削除）             | —                           |

**components/content.scss**

| 旧                          | 新                             |
| --------------------------- | ------------------------------ |
| `.blockquote`（エイリアス） | 削除（`.umd-blockquote` のみ） |
| `.spoiler`                  | `.umd-spoiler`                 |
| `.inline-code-color`        | `.umd-color-swatch`            |

**components/code-block.scss**

| 旧            | 新                |
| ------------- | ----------------- |
| `.code-block` | `.umd-code-block` |
| `.code-title` | `.umd-code-title` |

**base.scss**

| 旧           | 新               |
| ------------ | ---------------- |
| `.footnotes` | `.umd-footnotes` |

### 基本ルール（脱Bootstrap化）

1. CSS Layerを用いたリファレンスCSSを新設し、Bootstrap本体を必須依存から外す
2. Bootstrapのユーティリティクラス名は、UMD独自のプレフィックスへ改名する（例: `d-block` → `umd-block` など）。具体的な命名規則は未定
3. **意味を持たない色指定**（`blue` / `red` / `green` など、Bootstrapの標準カラー名）はそのままのクラス名・命名を踏襲する
4. **意味を持つ色指定**（`primary` / `danger` / `success` など、役割に基づく色）はCSSにハードコーディングせず、ホスト側がオプションで実際の色（CSS変数やクラス名）を指定できるようにする
5. **CSS論理プロパティを使用する**（後述）

### CSS 論理プロパティ方針

UMD が出力する HTML および提供する CSS は、物理的な方向指定（`left` / `right`）を使用せず、**CSS 論理プロパティ**（CSS Logical Properties & Values）を使用します。

これにより以下を担保します：

- **RTL 言語**（アラビア語・ヘブライ語等）: `dir="rtl"` を付けるだけで `start`/`end` が自動反転する
- **縦書き**（`writing-mode: vertical-rl`）: `inline`/`block` の軸が入れ替わっても崩れない

主な対応表：

| 物理プロパティ（使用禁止） | 論理プロパティ（使用すること） |
| -------------------------- | ------------------------------ |
| `text-align: left/right`   | `text-align: start/end`        |
| `margin-left/right`        | `margin-inline-start/end`      |
| `padding-left/right`       | `padding-inline-start/end`     |
| `border-left/right`        | `border-inline-start/end`      |
| `left/right`（position）   | `inset-inline-start/end`       |
| `top/bottom`               | `inset-block-start/end`        |
| `width/height`             | `inline-size/block-size`       |
| `float: left/right`        | `float: inline-start/end`      |

UMD 文法上の配置記法の正式名称は以下の通りです：

**インライン方向（text-align 相当）**

| 記法       | 対応 CSS              | 備考        |
| ---------- | --------------------- | ----------- |
| `START:`   | `text-align: start`   | 旧 `LEFT:`  |
| `END:`     | `text-align: end`     | 旧 `RIGHT:` |
| `CENTER:`  | `text-align: center`  | 変更なし    |
| `JUSTIFY:` | `text-align: justify` | 変更なし    |

**ブロック方向（vertical-align 相当）**

CSS 仕様に `vertical-align` の論理的代替が存在しないため、`V-` プレフィックスを付けた独自記法を採用します。

| 記法        | 対応 CSS                 | 備考         |
| ----------- | ------------------------ | ------------ |
| `V-START:`  | `vertical-align: top`    | 旧 `TOP:`    |
| `V-END:`    | `vertical-align: bottom` | 旧 `BOTTOM:` |
| `V-CENTER:` | `vertical-align: middle` | 旧 `MIDDLE:` |

> ⚠️ 既存コード（`src/extensions/block_decorations.rs` 等）に物理プロパティのクラス名が残っている場合は、CSS抽象化層の実装と合わせて順次移行する。

### 検討事項（脱Bootstrap化）

- 意味を持つ色（`primary`/`danger`等）のオプション指定方法（CSS変数、テーマ設定オブジェクト、ビルド時設定など）の具体化
- 既存のBootstrap前提ドキュメント（`docs/architecture.md`等）との整合、移行パス（Bootstrap版との共存可否）
- リファレンスCSSの配布方法（npm パッケージ、CDN、生成物としてのみ提供 等）

### 実装計画（脱Bootstrap化）

- [x] CSS Layer構成の設計（`@layer umd.reset, umd.base, umd.components, umd.utilities, umd.overrides`）
- [x] クラス名改名マッピングの確定・実施
- [x] セマンティックトークン（`primary`/`danger`等）の除去、ホスト側責務として明記
- [x] リファレンスCSS実装（`scss/` 配下の全ファイル整備）
- [x] `tokens.scss`: `light-dark()` によるダークモード一本化、パレット変数参照に移行
- [ ] Bootstrap依存コードの置き換え（`src/extensions/` 配下のクラス生成箇所）
- [ ] 既存テスト（`bootstrap_integration.rs` 等）の移行方針検討
- [ ] ドキュメント更新

---

## テスト結果

**総テスト数**: 308 tests (discoverable) ✅

> 注: テスト内訳は日次で変動するため、詳細な内訳は `cargo test -- --list` を基準とします。

**Phase 5 完了確認（2026-03-03）**:

- `cargo build --verbose` 成功
- `cargo test --verbose` 成功（ユニット・統合・doctest 全通過）

---

## 最近の実装（2026年8月）

### 2026年8月26日

#### リファレンスCSS整備・クラス名umd-\*統一

- 全SCSSファイルのBootstrapクラス名を `umd-*` プレフィックスへ改名（詳細は上表）
- `tokens.scss`: セマンティックトークン（`--umd-color-primary/success/danger/warning`）を削除、`light-dark()` に一本化、ハードコーディングされたhex値をOKLCHパレット変数参照へ置換
- `base.scss`: 全方向プロパティをCSS論理プロパティへ置換、`abbr[title]` スタイル追加（点線下線・緑・`cursor: help`）
- `components/outline.scss` 新規追加: 袋文字実装（`-webkit-text-stroke` + `paint-order: stroke fill`）
- `utilities/text.scss`: 縦方向アライメント（`umd-v-*`）・テキストサイズ（`umd-text-size-*`）クラス追加
- テキスト装飾記法の確定（`^^`/`~~`/`==`、`&outline()`、`&size()`）

### 2026年8月15日

#### Bidi対策のコードブロック向けオプション化

- `ParserOptions.allow_bidi_in_code_blocks`（デフォルト: `false`）を追加
- 無効時（デフォルト）: BiDi制御文字（`U+202A`-`U+202E` / `U+2066`-`U+2069`）はコードブロック内外を問わず除去
- 有効時: フェンス付きコードブロック（` ``` ` / `~~~`）内に限り BiDi制御文字を保持（RTLコードサンプルやTrojan Source型攻撃のデモ表示用途）。コードブロック外のBiDi制御文字、およびゼロ幅文字等の他の不可視文字はオプションの値に関わらず常に除去
- `src/sanitizer.rs` に `sanitize_opts` / `remove_disallowed_blank_chars_opts` を追加し、既存の `remove_ascii_control_chars_from_markup` と同じフェンス検出ロジックを共有
- `SECURITY.md` の記載を実際の挙動（コードブロック内も含めデフォルトで全除去）に合わせて修正

**テスト**: `sanitizer` 単体テスト7件追加、`tests/bidi_code_block_option.rs` 統合テスト3件追加

## 最近の実装（2026年4月〜5月）

- Rust バージョンを 1.95.0 にアップデート
- Punycode ドメインの視覚的脆弱性対策を実装
- Bidirectional (Bidi) テキストの脆弱性対策を追加
- 空文字の除去を実装（コードブロック以外の全要素に適用）
- npm パッケージ名を `universal-markdown` に変更

## 最近の実装（2026年2月〜3月）

### 2026年3月3日

#### 高度なUMD機能・ドキュメント同期

- 数式サポート（MathML出力）を実装済みとして反映
- ポップオーバー（インライン/ブロック）を実装済みとして反映
- インラインコード色サンプル（`#hex` / `rgb()` / `rgba()` / `hsl()` / `hsla()`）を実装
- 実装済み/予定ドキュメント間の整合を更新
- Phase 5（HTML生成・テスト整備）の完了条件を満たし、フェーズ状態を更新

**テスト**: `bootstrap_integration` 46/46 passing

### 2026年2月24日

#### ドキュメント体系の統一

- `.github/copilot-instructions.md` を簡潔なリファレンスに再設計
- `docs/architecture.md` に開発者向けガイドを統合
- 重複情報を排除し、単一の情報源を確立

### 2026年2月20日-24日

#### メディア機能の最終化

- ブロック vs インライン自動判別実装
- Bootstrap 5 マージン クラス（`ms/me`）採用
- 幅制御を figure レベルに集約
- 右揃え・中央揃え・左揃え・両端揃えプレフィックス対応

**テスト**: 24/24 integration tests passing

### 2026年2月20日

#### コードブロック強化

- Mermaid SVG レンダリング
- 複数行コンテンツ対応
- Bootstrap CSS 変数自動注入
- 言語別シンタックスハイライト対応

**テスト**: 12 code block tests passing

---

## 実装フェーズ

| Phase                   | 状態      | 期間           | 目標                           |
| ----------------------- | --------- | -------------- | ------------------------------ |
| 1: MVP (基本機能)       | ✅ 完了   | 1月-2月初      | Markdown + UMD基本             |
| 2: 準拠性向上           | ✅ 完了   | 2月初中        | CommonMark 75%+                |
| 3: 拡張機能             | ✅ 完了   | 2月中          | テーブル・プラグイン完成       |
| 4: 高度なUMD機能        | ✅ 完了   | 2月18日        | リスト内ブロック・タスク・属性 |
| 5: HTML生成・テスト整備 | ✅ 完了   | 2月24日-3月3日 | プロダクション品質             |
| 6: リリース準備         | 🚧 進行中 | 3月+           | 最適化・ドキュメント・配布     |

---

## 実装予定（次フェーズ）

### 短期（1-2週間）

1. **パフォーマンス最適化**
   - 正規表現パターンのキャッシング
   - 大規模ドキュメント処理の並列化検討
   - ベンチマーク測定

2. **ドキュメント完成**
   - Copilot Instructions の言語別バリアント（英語版）
   - API リファレンス自動生成
   - デモサイトの充実

### 中期（3-6週間）

1. **シンタックスハイライト** (ハイブリッド)
   - サーバー側: HTML 属性付与
   - フロントエンド: JavaScript オプション
   - Bootstrap CSS 変数カスタムテーマ

2. **Mermaidレンダリング最適化**
   - SVGキャッシュ戦略の整理
   - ダークモード時の可読性検証
   - 大規模ドキュメントでの描画コスト評価

### 長期（2ヶ月以降）

1. **テンプレートエンジン機能（将来構想）**
   - 仕様の段階的確定
   - バックエンド連携モデルの検証
   - 実装可否・優先度の再評価

---

## 技術的負債・改善項目

### 優先度：高

1. **ブロック装飾の複合処理最適化**
   - 現状: 各プレフィックスが個別に `<p>` タグ生成
   - 目標: 統一正規表現で1つのタグに統合
   - ファイル: `src/extensions/block_decorations.rs`, `conflict_resolver.rs`

2. **テーブル装飾の統一**
   - セル装飾関数の標準化
   - Bootstrap クラス マッピング効率化

### 優先度：中

1. **WASM バイナリサイズ最適化**
   - LTO (Link Time Optimization) 有効化
   - 不使用機能の削除検討
   - 目標: pkg/ < 200KB

2. **エラーメッセージの改善**
   - ユーザー向けエラーログ実装
   - デバッグモード（verbose）オプション

3. **syntect・mermaid-rs-renderer のオプショナル化**
   - 現状: シンタックスハイライト（syntect）とMermaid SSR（mermaid-rs-renderer）がコアのWASMバイナリに含まれており、バイナリサイズを圧迫している
   - 方針: 将来的に `umd-highlight` / `umd-mermaid` として別パッケージへ分離し、コアをミニマムに保つ
   - 背景:
     - **CSS Custom Highlight API** がChrome/Edge/Safari で実装済み・Firefox対応中。軽量クライアントサイドライブラリへの移行が現実的になりつつある
     - **Mermaid** は仕様が流動的で本家の破壊的変更に引きずられるリスクが高い。ホスト側の責務とする方がアーキテクチャ的に正しい
     - SEOへの影響はGoogleがJSを実行するため、コードブロックのSSRハイライトの優位性は限定的
   - 移行タイミング: CSS Custom Highlight APIのブラウザサポートが揃い、軽量ライブラリが成熟した段階

---

## 仕様確定事項

詳細は [docs/planned-features.md](docs/planned-features.md) を参照。

- ✅ **URL 自動リンク**: `<URL>` 形式のみサポート（裸 URL は非推奨）
- ✅ **URL スキーム**: `javascript:`, `data:`, `vbscript:`, `file:` ブロック
- ✅ **テキスト装飾記法**: 以下の記法を確定
  - `__text__` → `<u>` アンダーライン
  - `^^text^^` → `<span>` オーバーライン（`^`は「上」のニーモニック、ASCII範囲内）
  - `~~text~~` → `<span>` 波線アンダーライン（`~~`の形が波線に対応）
  - `==text==` → `<span>` アンダーライン + オーバーライン
  - `{- text -}` → `<del>` 削除（diff記法の`-`に対応）
- ✅ **袋文字プラグイン**: `&outline(fill, stroke){text}` — `-webkit-text-stroke` + `paint-order: stroke fill`
- ✅ **文字サイズ記法**: `&size(xs/sm/lg/xl){text}` — CSSキーワード値のみ（ピクセル指定はオプション、デフォルト無効）
- ✅ **数式構文**: `&math(LaTeX);` ($ 記号非採用)
- ✅ **フットノート**: JSON 構造化データ出力
- ✅ **絵文字**: Unicode 直接入力推奨、ショートコード非サポート
- ✅ **改行**: `&br;` 明示的タグ（テーブルセル対応）

---

## 参考リソース

- **仕様**: [LukiWiki Rules](https://github.com/logue/LukiWiki-core/blob/master/docs/rules.md)
- **CommonMark**: [仕様書](https://spec.commonmark.org/)
- **GFM**: [GitHub Flavored Markdown](https://github.github.com/gfm/)
- **Bootstrap 5**: [Documentation](https://getbootstrap.com/docs/5.3/)

---

## ライセンス

Apache License 2.0
