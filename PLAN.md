# LukiWiki Rustパーサー実装プラン

**プロジェクト概要**: Markdown上位互換を目指すLukiWikiのパース処理をRustで実装する。CommonMark仕様テストを合理的にパス(75%+目標)しつつ、レガシー構文を保持する。

**作成日**: 2026年1月23日
**最終更新**: 2026年1月31日
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
    - **Bootstrap固定スタイルシステム**: ✅ **完了**
      - デフォルトでBootstrap 5（Core UI互換）クラスを使用
      - text-align系はBootstrapクラスに完全置換
      - color/backgroundは任意色対応のためインラインスタイル維持
      - font-sizeはハイブリッド方式（単位なし→Bootstrapクラス、単位あり→インラインスタイル）
    - `COLOR(fg,bg): text` - 前景色・背景色指定（空白時は`inherit`） ✅
      - **✅ 実装完了**: Bootstrap変数のみサポート（案A採用）
      - **Bootstrap 5.3で利用可能な色（完全リスト）**:
        - **テーマカラー（8色）**: `primary`, `secondary`, `success`, `danger`, `warning`, `info`, `light`, `dark`
        - **基本カラー（14色）**: `blue`, `indigo`, `purple`, `pink`, `red`, `orange`, `yellow`, `green`, `teal`, `cyan`, `black`, `white`, `gray`, `gray-dark`
        - **グレースケール（9段階）**: `gray-100`, `gray-200`, `gray-300`, `gray-400`, `gray-500`, `gray-600`, `gray-700`, `gray-800`, `gray-900`
        - **各色の濃淡（9段階）**: `blue-100`～`blue-900`, `indigo-100`～`indigo-900`, など（基本カラー全てに対応）
        - **セマンティック色（v5.3追加）**: `body`, `body-secondary`, `body-tertiary`, `body-emphasis`, `border`, など
        - **サブトル色**: `primary-bg-subtle`, `success-border-subtle`, `danger-text-emphasis`, など
      - **合計**: 約200色のバリエーション（濃淡含む）
      - **十分性の評価**:
        - ✅ テーマカラー8色で基本的な用途はカバー可能
        - ✅ グレースケール9段階で微妙な濃淡表現が可能
        - ✅ 各色の100～900段階で細かい調整が可能
        - ✅ ダークモード対応のCSS変数が自動適用
        - ⚠️ 任意の中間色（例: `#FF6B35`）は表現不可
        - ⚠️ グラデーションやカスタムブランドカラーは不可
      - **判断**: Wiki用途では**Bootstrap色のみで十分と思われる**
        - Wikiはドキュメント中心で、デザイン自由度より読みやすさ優先
        - テーブルやコールアウトでの色分けは十分対応可能
        - ダークモード対応のメリットが大きい
      - **✅ 採用された実装: 案A（Bootstrap変数のみ）**
        - 前景色: `text-{color}`クラス（例: `text-primary`, `text-danger-emphasis`）
        - 背景色: `bg-{color}`クラス（例: `bg-warning`, `bg-success-subtle`）
        - 例: `COLOR(danger): エラー` → `<p class="text-danger">エラー</p>`
        - 例: `COLOR(,warning-subtle): 警告背景` → `<p class="bg-warning-subtle">警告背景</p>`
        - 例: `COLOR(primary,primary-subtle): 強調` → `<p class="text-primary bg-primary-subtle">強調</p>`
        - カスタムカラー（#FF0000など）: インラインスタイルとして出力
      - Bootstrap以外の色値はインラインスタイルにフォールバック
    - `SIZE(value): text` - フォントサイズ指定 ✅
      - **単位なし（数値のみ）**: Bootstrapクラスにマッピング
        - 例: `SIZE(2.5): 最大` → `<p class="fs-1">最大</p>` (2.5rem)
        - 例: `SIZE(2): 大きい` → `<p class="fs-2">大きい</p>` (2rem)
        - 例: `SIZE(1.75): やや大` → `<p class="fs-3">やや大</p>` (1.75rem)
        - 例: `SIZE(1.5): 標準より大` → `<p class="fs-4">標準より大</p>` (1.5rem)
        - 例: `SIZE(1.25): やや小` → `<p class="fs-5">やや小</p>` (1.25rem)
        - 例: `SIZE(0.875): 小さい` → `<p class="fs-6">小さい</p>` (0.875rem)
        - マッピング外の値: インラインスタイル出力（例: `SIZE(1.8):` → `style="font-size: 1.8rem"`）
      - **単位あり**: インラインスタイル出力
        - 例: `SIZE(1.5rem): 1.5rem` → `<p style="font-size: 1.5rem">1.5rem</p>`
        - 例: `SIZE(2em): 2em` → `<p style="font-size: 2em">2em</p>`
        - 例: `SIZE(16px): 16px` → `<p style="font-size: 16px">16px</p>`
      - 原則: LukiWikiはrem単位を標準とする
    - `RIGHT: text` - 右寄せ → `<p class="text-end">text</p>` (Bootstrap) ✅
    - `CENTER: text` - 中央寄せ → `<p class="text-center">text</p>` (Bootstrap) ✅
    - `LEFT: text` - 左寄せ → `<p class="text-start">text</p>` (Bootstrap) ✅
    - `JUSTIFY: text` - 両端揃え/ブロック幅指定 🚧 実装予定
      - **用途1: テキストの両端揃え（段落）**
        - 例: `JUSTIFY: この文章は両端揃えで表示されます。` → `<p class="text-justify">この文章は両端揃えで表示されます。</p>`
        - 注: Bootstrap 5では`text-justify`が非推奨だが、LukiWikiでは明示的に対応
        - ブラウザ対応: モダンブラウザでは`text-align: justify`で両端揃えが可能
      - **用途2: ブロック要素の幅指定（テーブル等）**
        - テーブル行の前に`JUSTIFY:`がある場合、テーブル全体の幅を100%に設定
        - 例:
          ```
          JUSTIFY:
          | Header1 | Header2 |
          |---------|---------|
          | Cell1   | Cell2   |
          ```
          → `<table class="table w-100">...</table>` （デフォルトで`table`クラスが追加される）
        - 用途: `CENTER:`でテーブル自体を中央寄せするのではなく、テーブルを画面幅いっぱいに広げた上でセル内のテキストを配置
        - 区別:
          - `CENTER: | Header |` → テーブル自体を中央寄せ（テーブルはコンテンツ幅）
          - `JUSTIFY: | Header |` → テーブルを100%幅に拡張（テーブルが画面幅いっぱい）
          - `JUSTIFY: CENTER: | Header |` → 100%幅のテーブルで、セル内テキストを中央揃え
      - **実装方針**:
        - 段落に対しては`text-justify`クラスを適用
        - テーブルに対しては`w-100`クラス（Bootstrap）または`style="width: 100%"`を適用
        - テーブル検出: `JUSTIFY:`の直後の行が`|`で始まる場合、次のブロック（テーブル）に適用
    - `TRUNCATE: text` - テキスト省略 → `<p class="text-truncate">text</p>` (Bootstrap) 🚧 実装予定
      - 長いテキストを`...`で省略（`overflow: hidden; text-overflow: ellipsis; white-space: nowrap`）
      - 幅指定はユーザーがCSSで指定する前提（テーブルセルでは自動適用）
    - **複合構文**: 複数のプレフィックスを組み合わせ可能 ✅ **完了**
      - **必須機能**: テーブルセル装飾で複数スタイルの同時適用が必要
      - **構文順序**: `SIZE(...): COLOR(...): TRUNCATE: JUSTIFY/RIGHT/CENTER/LEFT: テキスト`
        - サイズ → 色 → 省略 → 配置の順序を推奨（順不同でも動作）
      - **実装方針**:
        - 1つの正規表現で全プレフィックスをまとめて解析
        - パース結果を構造体に格納し、1つの`<p>`タグに統合
        - Bootstrapクラスとインラインスタイルを適切に分離
      - **出力例**:
        - `SIZE(1.5): COLOR(red): CENTER: テキスト` → `<p class="fs-4 text-center" style="color: red">テキスト</p>`
        - `TRUNCATE: CENTER: 長いテキスト` → `<p class="text-truncate text-center">長いテキスト</p>`
        - `COLOR(primary): RIGHT: 青い右寄せ` → `<p class="text-primary text-end">青い右寄せ</p>` (案A採用時)
        - `JUSTIFY: テキスト` → `<p class="text-justify">テキスト</p>`
        - `JUSTIFY: CENTER: テキスト` → `<p class="text-justify text-center">テキスト</p>` (両端揃え+中央寄せは矛盾するが、CSSの優先順位による)
      - **技術的課題**:
        - 現在の実装は各プレフィックスが個別に`<p>`タグを生成（不正なネスト発生）
        - 大幅な書き換えが必要（block_decorations.rsの再設計）
        - conflict_resolver.rsでの複合マーカー処理も対応必須
      - テーブルのセル装飾で特に有用
    - [src/lukiwiki/block_decorations.rs](src/lukiwiki/block_decorations.rs)実装完了（Bootstrap対応は未実装）
  - **インライン装飾関数** (プラグインのインライン型と同じ表記): ✅
    - `&color(fg,bg){text};` - 文字色・背景色指定（空白時は`inherit`） ✅
      - **注**: ブロック版`COLOR()`と同じ実装方針を採用（上記参照）
      - **✅ 実装完了: Bootstrap変数のみ**
        - 例: `&color(danger){エラー};` → `<span class="text-danger">エラー</span>`
        - 例: `&color(,warning-subtle){警告};` → `<span class="bg-warning-subtle">警告</span>`
        - 例: `&color(primary,primary-subtle){強調};` → `<span class="text-primary bg-primary-subtle">強調</span>`
      - 現在の実装: 任意色、インラインスタイル
    - `&size(value){text};` - フォントサイズ指定 ✅
      - **単位なし（数値のみ）**: Bootstrapクラスにマッピング ✅
        - 例: `&size(2){大きい};` → `<span class="fs-2">大きい</span>` (2rem)
        - 例: `&size(1.5){やや大};` → `<span class="fs-4">やや大</span>` (1.5rem)
        - マッピング外: インラインスタイル（例: `&size(1.8){text};` → `style="font-size: 1.8rem"`）
      - **単位あり**: インラインスタイル出力
        - 例: `&size(1.5rem){text};` → `<span style="font-size: 1.5rem">text</span>`
        - 例: `&size(2em){text};` → `<span style="font-size: 2em">text</span>`
    - `&badge(type){text};` - Bootstrapバッジ表示 ✅ **完了**
      - **Bootstrap Badge**: 小さなカウント表示やラベル表示用コンポーネント
      - **サポートするバッジタイプ（8種類）**:
        - `primary`, `secondary`, `success`, `danger`, `warning`, `info`, `light`, `dark`
        - 例: `&badge(primary){New};` → `<span class="badge bg-primary">New</span>`
        - 例: `&badge(danger){Error};` → `<span class="badge bg-danger">Error</span>`
        - 例: `&badge(success){4};` → `<span class="badge bg-success">4</span>` (カウント表示)
      - **ピルバッジ（丸みのある形状）**: `&badge(primary-pill){text};`
        - 例: `&badge(success-pill){Active};` → `<span class="badge rounded-pill bg-success">Active</span>`
      - **リンク複合パターン**: ✅ **実装完了**
        - **問題**: バッジがリンクになる場合の構文をどうするか
        - **案A: Markdown構文のネスト（採用予定）**: `&badge(primary){[New](/new)};`
          - メリット: Markdown標準構文を活用、学習コスト低い
          - デメリット: パース処理が複雑、後処理が必要
          - 出力: `<a href="/new" class="badge bg-primary">New</a>`
          - **実装方針（2つのアプローチ）**:
            - **アプローチ1: 後処理で変換**（シンプルだが処理コスト高い）
              1. `&badge(primary){[New](/new)};` → `<span class="badge bg-primary">[New](/new)</span>`
              2. comrakが `[New](/new)` → `<a href="/new">New</a>` に変換
              3. 結果: `<span class="badge bg-primary"><a href="/new">New</a></span>`
              4. **後処理**: 正規表現で `<span class="badge ([^"]+)"><a href="([^"]+)">([^<]+)</a></span>` を検出
              5. `<a href="$2" class="badge $1">$3</a>` に置換
              - ⚠️ **懸念点**:
                - 後処理が必要で処理コスト増加
                - 正規表現が複雑になる可能性
                - バッジ内に複数のリンクがある場合の対応が難しい
            - **アプローチ2: badge処理時に直接リンク検出（推奨）**
              1. `&badge(primary){[New](/new)};` をパース
              2. content部分 `[New](/new)` に対して**即座に**Markdownリンク正規表現を適用
              3. リンクパターン `\[([^\]]+)\]\(([^)]+)\)` を検出
              4. 検出成功 → `<a href="/new" class="badge bg-primary">New</a>` を直接生成
              5. 検出失敗 → 通常の `<span class="badge bg-primary">text</span>` を生成
              - ✅ **利点**:
                - 後処理不要、1パスで完結
                - 処理が明確で、comrakとの干渉なし
                - パフォーマンス良好（対象はbadge内のcontentのみ）
              - **実装**:

                ```rust
                // inline_decorations.rsでの実装例
                static BADGE_PATTERN: Lazy<Regex> =
                    Lazy::new(|| Regex::new(r"&badge\(([^)]+)\)\{([^}]+)\};").unwrap());
                static LINK_PATTERN: Lazy<Regex> =
                    Lazy::new(|| Regex::new(r"\[([^\]]+)\]\(([^)]+)\)").unwrap());

                // badge処理時
                if let Some(link_caps) = LINK_PATTERN.captures(content) {
                    let text = &link_caps[1];
                    let url = &link_caps[2];
                    format!("<a href=\"{}\" class=\"badge bg-{}\">{}</a>", url, badge_type, text)
                } else {
                    format!("<span class=\"badge bg-{}\">{}</span>", badge_type, content)
                }
                ```

          - **✅ 採用方針**: **アプローチ2（直接リンク検出）**を採用・実装完了
            - 理由: 処理が軽量、実装が明確、後処理不要
            - 処理負荷: 最小限（badgeパターンが検出された時のみ、content部分に対してリンク正規表現を1回実行）
            - バッジ+リンクは頻繁に使われないため、全体のパフォーマンスへの影響は軽微

        - **案B: 専用構文**: `&badge(primary,/new){New};`
          - メリット: シンプル、パース容易
          - デメリット: 新しい構文を覚える必要がある、他の装飾関数と一貫性がない
          - 出力: `<a href="/new" class="badge bg-primary">New</a>`
        - **案C: 別関数**: `&badge-link(primary,/new){New};`
          - メリット: 明示的、混乱が少ない
          - デメリット: 関数が増える、冗長
          - 出力: `<a href="/new" class="badge bg-primary">New</a>`

      - **使用例**:
        - ステータス表示: `Status: &badge(success){Active};` → `Status: <span class="badge bg-success">Active</span>`
        - 通知カウント: `Messages &badge(danger){99+};` → `Messages <span class="badge bg-danger">99+</span>`
        - タグ: `&badge(secondary){Tag1}; &badge(secondary){Tag2};` → `<span class="badge bg-secondary">Tag1</span> <span class="badge bg-secondary">Tag2</span>`
      - **ダークモード対応**: Bootstrap CSS変数で自動切替

    - `&sup(text);` - 上付き文字 → `<sup>text</sup>`
    - `&sub(text);` - 下付き文字 → `<sub>text</sub>`
    - `&lang(locale){text};` - 言語指定 → `<span lang="locale">text</span>`
      - 例: `&lang(en){Hello};` → `<span lang="en">Hello</span>`
    - `&abbr(text){description};` - 略語説明 → `<abbr title="description">text</abbr>`
    - `&ruby(reading){text};` - ルビ（ふりがな）表示 → `<ruby>text<rp>(</rp><rt>reading</rt><rp>)</rp></ruby>`
      - 例: `&ruby(Ashita){明日};` → `<ruby>明日<rp>(</rp><rt>Ashita</rt><rp>)</rp></ruby>`
      - 注: `<rp>`タグはルビ未対応ブラウザで括弧を表示するためのフォールバック
    - **セマンティックHTML要素**:
      - `&dfn(text);` - 定義される用語 → `<dfn>text</dfn>`
      - `&kbd(text);` - キーボード入力 → `<kbd>text</kbd>`
      - `&samp(text);` - サンプル出力 → `<samp>text</samp>`
      - `&var(text);` - 変数 → `<var>text</var>`
      - `&cite(text);` - 作品タイトル → `<cite>text</cite>`
      - `&q(text);` - 短い引用 → `<q>text</q>`
      - `&small(text);` - 細目・注釈 → `<small>text</small>`
      - `&u(text);` - 下線（非言語的注釈） → `<u>text</u>`
        - 注: Markdownに下線構文は存在しないため矛盾なし
      - `&time(datetime){text};` - 日時 → `<time datetime="datetime">text</time>`
        - 例: `&time(2026-01-26){今日};` → `<time datetime="2026-01-26">今日</time>`
      - `&data(value){text};` - 機械可読データ → `<data value="value">text</data>`
      - `&bdi(text);` - 双方向テキスト分離 → `<bdi>text</bdi>`
      - `&bdo(dir){text};` - 双方向テキスト上書き → `<bdo dir="dir">text</bdo>`
        - 例: `&bdo(rtl){right-to-left};` → `<bdo dir="rtl">right-to-left</bdo>`
      - `&wbr;` - 改行可能位置 → `<wbr />`
    - [src/lukiwiki/inline_decorations.rs](src/lukiwiki/inline_decorations.rs)実装完了

  - **取り消し線構文の分離**: ✅
    - **LukiWiki**: `%%text%%` → `<s>text</s>` (視覚的な取り消し線)
    - **Markdown/GFM**: `~~text~~` → `<del>text</del>` (削除を意味する取り消し線)
    - 注: 両方共取り消し線として表示されるが、HTMLの意味合いが異なる
      - `<s>`: 正確でなくなった内容や関連性のなくなった内容
      - `<del>`: ドキュメントから削除された内容
    - 実装: [src/lukiwiki/inline_decorations.rs](src/lukiwiki/inline_decorations.rs)でLukiWiki形式を処理後、comrakでMarkdown形式を処理
  - **プラグインシステム** (拡張可能なWiki機能): ✅
    - **インライン型（完全形）**: `&function(args){content};`
      - パース出力: `<span class="plugin-function" data-args='["arg1","arg2"]'>content</span>`
    - **インライン型（args-only）**: `&function(args);` ✅
      - パース出力: `<span class="plugin-function" data-args='["arg1","arg2"]' />`
      - 例: `&icon(mdi-pencil);` → `<span class="plugin-icon" data-args='["mdi-pencil"]' />`
    - **インライン型（no-args）**: `&function;` ✅
      - パース出力: `<span class="plugin-function" data-args='[]' />`
      - 例: `&br;` → `<span class="plugin-br" data-args='[]' />`
    - **ブロック型（複数行）**: `@function(args){{ content }}`
      - パース出力: `<div class="plugin-function" data-args='["arg1","arg2"]'>content</div>`
      - デフォルトタグ: `div`（SSR対応、SEO対応、アクセシビリティ対応）
      - カスタムタグ指定: プラグイン側で`data-tag`属性を使用して任意のタグを指定可能
        - セマンティックタグ: `aside`, `figure`, `section`, `nav`, `article`, `header`, `footer`, `main`
        - 遅延実行: `template`（JavaScript必須、SSR非対応）
      - 使い分け:
        - `div`/セマンティックタグ: サーバーサイドレンダリング、静的コンテンツ、SEO重要、アクセシビリティ必須
        - `template`: クライアントサイド専用、動的コンテンツ（API取得等）、遅延実行、JavaScript必須
    - **ブロック型（単行）**: `@function(args){content}`
      - パース出力: `<div class="plugin-function" data-args='["arg1","arg2"]'>content</div>`
      - デフォルトタグ: `div`
      - カスタムタグ指定: プラグイン側で`data-tag`属性を使用
    - **ブロック型（args-only）**: `@function(args)` ✅
      - パース出力: `<div class="plugin-function" data-args='["arg1","arg2"]' data-tag="div" />`
      - デフォルトタグ: `div`（SSR対応、SEO対応、アクセシビリティ対応）
      - カスタムタグ指定: プラグイン側で`data-tag`属性を読み取り、指定されたタグで置換
      - 例: `@callout(info)` → `<div class="plugin-callout" data-args='["info"]' data-tag="div" />`
      - 例（カスタムタグ）: `<aside class="plugin-callout" data-args='["info"]'>...</aside>`
      - **重要**: 括弧必須（`@function()`）で@mentionと区別
      - **URL保護**: argsをbase64エンコードしてMarkdownパーサーのautolink機能から保護
    - **ブロック型（no-args）**: `@function()` ✅
      - パース出力: `<div class="plugin-function" data-args='[]' data-tag="div" />`
      - デフォルトタグ: `div`
      - カスタムタグ指定: プラグイン側で`data-tag`属性を読み取り
      - 例: `@toc()` → `<div class="plugin-toc" data-args='[]' data-tag="div" />`
      - 例（カスタムタグ）: `<nav class="plugin-toc">...</nav>`
      - **重要**: 括弧必須で@mentionと区別
    - **カスタムタグの使用例**:
      - `@gallery(photos){{ ... }}` → JavaScript側で `<figure class="plugin-gallery">...</figure>` に変換
      - `@callout(warning){{ ... }}` → JavaScript側で `<aside class="plugin-callout">...</aside>` に変換
      - `@toc()` → JavaScript側で `<nav class="plugin-toc">...</nav>` に変換
      - `@feed(url)` → JavaScript側で `<template>`から動的コンテンツ生成（CSR専用）
    - **許可されるタグ**:
      - **セマンティックタグ**（推奨）: `div`, `aside`, `figure`, `section`, `nav`, `article`, `header`, `footer`, `main`
      - **遅延実行タグ**: `template`（JavaScript必須、SSR非対応）
    - **タグ選択ガイドライン**:
      - `div`/セマンティックタグ: サーバーサイドレンダリング、静的コンテンツ、SEO重要、アクセシビリティ必須
      - `template`: クライアントサイド専用、動的コンテンツ（API取得等）、遅延実行、JavaScript必須
    - **セキュリティ**: プラグイン側でタグ名のホワイトリストチェックを推奨
  - **プラグイン開発者向けガイドライン**: 🚧 ドキュメント作成予定
    - パーサーは`plugin-*`クラスのみ生成
    - フロントエンドJavaScriptでBootstrapコンポーネントを動的追加
    - 例: `<div class="plugin-alert" data-args='["info"]'>` → JavaScript側で`alert alert-info`クラス追加
    - Bootstrap 5（Core UI互換）のユーティリティクラス（`text-*`, `mb-*`, `d-*`, `fs-*`等）を活用可能
  - **GitHub Flavored Markdownアラート（Callouts）**: ✅ **完了** (注: sanitization制約あり)
    - GFM拡張機能として、ブロック引用ベースのアラート構文をサポート
    - **構文**: `> [!TYPE]`で始まるブロック引用
    - **サポートするアラートタイプ（5種類）**:
      - `[!NOTE]` - 補足情報 → Bootstrap `alert-info`（青）
        - 例: `> [!NOTE]\n> スキミング時にも知っておくべき有用な情報`
        - 出力: `<div class="alert alert-info" role="alert">スキミング時にも知っておくべき有用な情報</div>`
      - `[!TIP]` - ヒント → Bootstrap `alert-success`（緑）
        - 例: `> [!TIP]\n> より良く、より簡単に行うための役立つアドバイス`
        - 出力: `<div class="alert alert-success" role="alert">より良く、より簡単に行うための役立つアドバイス</div>`
      - `[!IMPORTANT]` - 重要 → Bootstrap `alert-primary`（青紫）
        - 例: `> [!IMPORTANT]\n> 目標達成のために知っておくべき重要情報`
        - 出力: `<div class="alert alert-primary" role="alert">目標達成のために知っておくべき重要情報</div>`
      - `[!WARNING]` - 警告 → Bootstrap `alert-warning`（黄）
        - 例: `> [!WARNING]\n> 問題を回避するために即座に注意が必要な緊急情報`
        - 出力: `<div class="alert alert-warning" role="alert">問題を回避するために即座に注意が必要な緊急情報</div>`
      - `[!CAUTION]` - 注意 → Bootstrap `alert-danger`（赤）
        - 例: `> [!CAUTION]\n> 特定の行動のリスクや否定的な結果についての助言`
        - 出力: `<div class="alert alert-danger" role="alert">特定の行動のリスクや否定的な結果についての助言</div>`
    - **実装方針**:
      - comrakのブロック引用処理後、カスタムポストプロセッサで`[!TYPE]`を検出
      - `<blockquote>`タグを`<div class="alert alert-{type}">`に変換
      - アイコン追加はフロントエンドJavaScript/CSSで対応（Bootstrap Iconsなど）
      - アクセシビリティ: `role="alert"`属性を自動追加
    - **既存LukiWiki引用構文との共存**:
      - LukiWiki形式: `> ... <` （閉じタグあり）→ 通常のブロック引用として処理
      - GFMアラート形式: `> [!TYPE]` → Bootstrapアラートに変換
      - Markdown標準: `> text` （閉じタグなし、`[!TYPE]`なし）→ 通常のブロック引用
    - **利点**:
      - GFMとの互換性向上（GitHub README等との相互運用性）
      - Bootstrapアラートを簡潔な構文で利用可能
      - プラグインシステムよりもシンプル（`@alert(type){{ content }}`より短い）
      - 標準的なMarkdown構文の拡張で学習コスト低い
  - **カスタムヘッダーID**: `# Header {#custom-id}` ✅
    - PukiWiki Advanceと同様の構文
    - ヘッダーに任意のIDを指定可能
    - 指定がない場合は`heading-1`, `heading-2`と自動採番
    - **メリット**:
      - URLセーフ（マルチバイト文字によるエンコード問題を回避）
      - 短いURL（SNSでの共有に最適）
      - 安定したリンク（ヘッダーテキスト変更に強い）
      - セキュリティ（同形異字による偽装攻撃を防止）
    - 実装: [src/lukiwiki/conflict_resolver.rs](src/lukiwiki/conflict_resolver.rs)でカスタムID抽出とHTML生成
    - [examples/test_header_id.rs](examples/test_header_id.rs)でデモンストレーション
  - **フロントマター**: YAML/TOML形式のメタデータ ✅
    - YAML形式: `---` で囲む
    - TOML形式: `+++` で囲む
    - HTML出力から除外され、`ParseResult.frontmatter`で取得可能
    - 実装: [src/frontmatter.rs](src/frontmatter.rs)
    - [examples/test_frontmatter.rs](examples/test_frontmatter.rs)でデモンストレーション
  - **フットノート（脚注）**: Markdown標準構文のサポート ✅
    - 構文: `[^1]`, `[^note]` で参照、`[^1]: content` で定義
    - HTML出力: `<section class="footnotes">` として生成
    - 本文から分離され、`ParseResult.footnotes`で取得可能
    - comrakの`extension.footnotes`を有効化
    - [examples/test_footnotes.rs](examples/test_footnotes.rs)でデモンストレーション
  - **テーブル**: Markdown標準形式 + LukiWiki拡張 🚧 実装予定
    - **基本構文**: Markdown標準のテーブル構文（GFM準拠）
      ```
      | Header1 | Header2 |
      |---------|---------|
      | Cell1   | Cell2   |
      ```
    - **デフォルトクラス**: `<table class="table">` （Bootstrap基本クラスを自動付与）
    - **テーブルバリエーション**: 🔮 今後の課題
      - 色（`table-striped`, `table-hover`, `table-dark`など）
      - ボーダー（`table-bordered`, `table-borderless`）
      - サイズ（`table-sm`）
      - 現状: プラグインシステムで対応予定（例: `@table(striped,hover){{ ... }}`）
    - **セル内装飾（水平配置）**: ブロック装飾プレフィックスと同様
      - `RIGHT:` → `text-end`（右寄せ）
      - `CENTER:` → `text-center`（中央寄せ）
      - `LEFT:` → `text-start`（左寄せ）
      - `JUSTIFY:` → `text-justify`（両端揃え）
      - 例: `| RIGHT: Cell1 | CENTER: Cell2 |`
    - **セル内装飾（垂直配置）**: 🚧 実装予定
      - `TOP:` → `align-top`（上揃え）
      - `MIDDLE:` → `align-middle`（中央揃え）
      - `BOTTOM:` → `align-bottom`（下揃え）
      - `BASELINE:` → `align-baseline`（ベースライン揃え）
      - 例: `| TOP: Cell1 | MIDDLE: Cell2 |`
      - 複合: `| TOP: RIGHT: Cell1 |` → `<td class="align-top text-end">Cell1</td>`
    - **セル内その他装飾**: `COLOR()`, `SIZE()`, `TRUNCATE`も使用可能
      - 例: `| COLOR(primary): SIZE(1.5): MIDDLE: CENTER: 強調セル |`
    - **テーブル幅指定**: `JUSTIFY:`でテーブル全体を100%幅に設定（上記参照）
    - **レスポンシブ対応**: プラグインで実装予定
      - 例: `@table(responsive){{ ... }}` → `<div class="table-responsive"><table>...</table></div>`
  - **ブロック引用**: LukiWiki形式 + Markdown標準形式 ✅
    - **LukiWiki形式**: `> ... <` （閉じタグあり）
    - **Markdown形式**: `> text` （行頭プレフィックス）
    - **デフォルトクラス**: 🚧 実装予定
      - `<blockquote class="blockquote">` （Bootstrap基本クラスを自動付与）
      - Bootstrap標準の引用スタイルを適用
    - **GFMアラート**: `> [!NOTE]`などは別途`<div class="alert">`に変換（上記参照）
  - **定義リスト**: 🚧 実装予定
    - **LukiWiki構文**: `:term|definition`
      ```
      :用語1|定義1
      :用語2|定義2の説明文
      ```
    - **Markdown拡張構文（オプション）**: CommonMarkには標準定義リスト構文がないが、一部の実装で対応

      ```
      用語1
      : 定義1

      用語2
      : 定義2の説明文
      ```

    - **HTML出力**: `<dl>`, `<dt>`, `<dd>`タグ
      ```html
      <dl>
        <dt>用語1</dt>
        <dd>定義1</dd>
        <dt>用語2</dt>
        <dd>定義2の説明文</dd>
      </dl>
      ```
    - **Bootstrapスタイリング**: デフォルトクラスなし（カスタムCSSまたはユーティリティクラスで調整）
      - オプション: `<dl class="row">`で2カラムレイアウト可能
        ```html
        <dl class="row">
          <dt class="col-sm-3">用語</dt>
          <dd class="col-sm-9">定義</dd>
        </dl>
        ```
    - **実装方針**:
      - LukiWiki構文`:term|definition`を優先実装
      - 行頭の`:`で定義リストを検出
      - 連続する定義リスト項目を1つの`<dl>`タグにグループ化
      - 定義は複数行対応（インデントで継続行を判定）
      - Markdown拡張構文は将来的な検討事項
    - **複数定義の対応**:
      ```
      :用語1|定義1-1
      :用語1|定義1-2
      ```
      →
      ```html
      <dl>
        <dt>用語1</dt>
        <dd>定義1-1</dd>
        <dd>定義1-2</dd>
      </dl>
      ```
    - **複数用語の対応**:
      ```
      :用語1|定義
      :用語2|定義
      ```
      →
      ```html
      <dl>
        <dt>用語1</dt>
        <dt>用語2</dt>
        <dd>定義</dd>
      </dl>
      ```
      （注: 同じ定義を持つ複数の用語）
    - **ネストされたコンテンツ**:
      - 定義内でMarkdown構文（強調、リンクなど）をサポート
      - 定義内でLukiWiki装飾関数（`&color()`, `&badge()`など）をサポート
    - **用途**: 用語集、FAQ、仕様書などでの使用を想定

**成果物**:

- LukiWiki構文パーサーモジュール群 ✅
  - emphasis.rs (5 tests)
  - block_decorations.rs (7 tests)
  - inline_decorations.rs (11 tests including strikethrough)
  - plugins.rs (20 tests including args-only/no-args patterns)
  - frontmatter.rs (5 tests)
  - conflict_resolver.rs (11 tests including custom header ID tests)
- レガシー構文互換性テスト ✅ (48 LukiWiki syntax tests passing)
- プラグインパターンデモ: [examples/test_plugin_extended.rs](examples/test_plugin_extended.rs) ✅

**テスト結果**: 123 tests passing

- 83 unit tests (including 5 frontmatter + 3 custom header ID + 9 new plugin tests + 2 strikethrough tests)
- 18 CommonMark tests
- 13 conflict resolution tests
- 9 doctests

---

### Step 4: 構文競合の解決 ✅ 完了

**目的**: MarkdownとLukiWiki構文の衝突を適切に処理

**ステータス**: ✅ **完了** (2025年版)

**作業内容**:

- [src/lukiwiki/conflict_resolver.rs](src/lukiwiki/conflict_resolver.rs)を作成 ✅
- マーカーベース前処理システム実装 ✅
  - プリプロセス: LukiWiki構文を`{{MARKER:...:MARKER}}`形式で保護
  - サニタイズーション: マーカーはHTMLエスケープされない
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
    - インライン: `&function(args){content};`, `&function(args);`, `&function;`
    - ブロック: `@function(args){{ content }}`, `@function(args){content}`, `@function(args)`, `@function()`
    - base64エンコーディングでコンテンツとargsを安全に保持
    - URL自動リンク化の防止: argsをbase64エンコードすることでMarkdownパーサーがURLをリンク化するのを防止
    - ネストされたプラグインと内部のWiki構文を完全保護
    - 処理順序: braces付きパターン → args-onlyパターン → no-argsパターン
  - **カスタムヘッダーID**: ✅
    - `# Header {#custom-id}` 構文のサポート
    - プリプロセスでカスタムIDを抽出・除去
    - ポストプロセスで`<h1><a href="#id" id="id"></a>Title</h1>`形式のHTMLを生成
    - カスタムIDがない場合は自動採番（`heading-1`, `heading-2`...）

**成果物**:

- 構文曖昧性解消モジュール ✅
- プリプロセス/ポストプロセスパイプライン ✅
- 競合検出診断ツール ✅
- カスタムヘッダーID実装 ✅

**テスト結果**: 16 conflict resolution tests passing (including 3 custom header ID tests)

- 競合ケースの網羅的テスト
- 優先順位ドキュメント（コード内コメント）
- カスタムヘッダーID抽出・適用テスト

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

2.5. **取り消し線**: ✅

- 両スタイルサポート（共存）
- Markdown/GFM → セマンティックタグ (`<del>`) - 削除された内容
- LukiWiki → 視覚的タグ (`<s>`) - 正確でなくなった内容
- 違い: HTMLの意味合いが異なる（視覚的 vs セマンティック）
- **矛盾なし**: 構文が明確に異なる (`%%` vs `~~`)

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

8. **プラグイン構文と@mention**: ✅
   - プラグイン: `@function()` - 括弧必須
   - @mention: `@username` - 括弧なし
   - **矛盾なし**: 括弧の有無で明確に区別可能

### Markdown仕様との矛盾箇所まとめ

| LukiWiki構文  | Markdown構文        | 矛盾度 | 解決策                   |
| ------------- | ------------------- | ------ | ------------------------ |
| `'''text'''`  | `***text***`        | 中     | 3連続クォートを優先検出  |
| `+ item`      | `+ item` (一部方言) | 低     | 順序付きリストとして統一 |
| `COLOR(...):` | `: definition`      | 低     | 大文字キーワードで判別   |
| `> ... <`     | `> quote`           | 低     | 閉じタグで判別           |
| `%%text%%`    | `~~text~~`          | 低     | 異なる構文で明確に区別   |
| `@function()` | `@mention`          | 低     | 括弧の有無で区別         |

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
