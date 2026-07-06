# Universal Markdown (UMD) 実装プラン

**プロジェクト概要**: Markdownを超える次世代マークアップ言語。CommonMark仕様テスト 75%+ パス、Bootstrap 5統合、セマンティックHTML、拡張可能なプラグインシステム提供。

**作成日**: 2026年1月23日  
**最終更新**: 2026年7月6日  
**Rustバージョン**: 1.96.1 (Edition 2024)  
**ライセンス**: Apache-2.0

### 主要ライブラリ

| ライブラリ          | バージョン | 用途                                     |
| ------------------- | ---------- | ---------------------------------------- |
| comrak              | 0.52.0     | Markdown パーサー (GFM)                  |
| ammonia             | 4.1.2      | HTML サニタイズ                          |
| maud                | 0.27.0     | 型安全 HTML 生成                         |
| regex               | 1.12.3     | パターンマッチング                       |
| wasm-bindgen        | 0.2.120    | WASM バインディング                      |
| syntect             | 5.3.0      | シンタックスハイライト（ネイティブのみ） |
| mermaid-rs-renderer | 0.2.2      | Mermaid SSR レンダリング                 |
| math-core           | 0.6.0      | LaTeX to MathML 変換                     |

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

---

## ブロック型プラグイン書式仕様（`::: 記法`）

### 概要

QiitaやGrowiで採用されている書式を新たにサポートします。

```
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

```markdown
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
- [x] ドキュメント更新（[docs/plugin-system.md](docs/plugin-system.md)）

---

## テスト結果

**総テスト数**: 308 tests (discoverable) ✅

> 注: テスト内訳は日次で変動するため、詳細な内訳は `cargo test -- --list` を基準とします。

**Phase 5 完了確認（2026-03-03）**:

- `cargo build --verbose` 成功
- `cargo test --verbose` 成功（ユニット・統合・doctest 全通過）

---

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

---

## 仕様確定事項

詳細は [docs/planned-features.md](docs/planned-features.md) を参照。

- ✅ **URL 自動リンク**: `<URL>` 形式のみサポート（裸 URL は非推奨）
- ✅ **URL スキーム**: `javascript:`, `data:`, `vbscript:`, `file:` ブロック
- ✅ **下線構文**: `__text__` → `<u>` (Discord 風)
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
