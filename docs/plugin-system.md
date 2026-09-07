# プラグインシステム

**最終更新**: 2026年7月6日

Universal Markdown のプラグイン構文と出力形式です。

## 構文

### インライン型

- `&function(arg1,arg2){content};`
- `&function(args);`
- `&function;`

### ブロック型

- `@function(args){{ ... }}`
- `@function(args){...}`
- `@function(args)`
- `@function()`

### ブロック型（`::: 記法`）

Qiita/GROWI 系の記法との互換性のための代替構文です。`@function(args){{ ... }}`
と同じプラグインシステムに統合されており、出力形式・標準プラグイン（`@table` /
`@math` / `@popover` / `@clear`）の扱いも共通です。

```umd
:::function args
content
:::
```

- **開始**: `:::function args` （`function` はプラグイン名、`args` は開始行の
  残り全体を引数文字列として使用。カンマ区切りで複数引数も指定可能）
- **終了**: `:::` のみからなる行
- **入れ子**: 非サポート。ブロックは最初に現れる `:::` のみの行で閉じるため、
  内側に別の `:::` ブロックや `@`/`&` プラグイン構文を書いても、それらは
  プラグインとして解釈されず、エスケープされた生テキストとして扱われます。

## 出力形式

プラグインは次の形式で出力されます。

- `<template class="umd-plugin umd-plugin-{name}">...</template>`
- 引数は `<data value="index">...</data>` で保持
- コンテンツはエスケープ済みテキストとして保持

バックエンド側（Nuxt/Laravel 等）で再パースして最終描画する設計です。

## 実際の出力例

### インラインプラグイン

入力:

```umd
&badge(primary){New};
&hint(info);
&clear;
```

出力例:

```html
<template class="umd-plugin umd-plugin-badge">
  <data value="0">primary</data>
  New
</template>
<template class="umd-plugin umd-plugin-hint">info</template>
<template class="umd-plugin umd-plugin-clear"></template>
```

### ブロックプラグイン

入力:

```umd
@card(info){{
  **Markdown** content
}}

@toc(2)
```

出力例:

```html
<template class="umd-plugin umd-plugin-card">
  <data value="0">info</data>
  **Markdown** content
</template>
<template class="umd-plugin umd-plugin-toc">2</template>
```

### 標準プラグイン（直接HTML出力）

入力:

```umd
@detail(詳細, open){{
  内容
}}
@clear()
```

出力例:

```html
<details open>
  <summary>詳細</summary>
  内容
</details>
<div class="clearfix"></div>
```

### ブロックプラグイン（`::: 記法`）

入力:

```umd
:::alert warning
Something went wrong
:::
```

出力例:

```html
<template class="umd-plugin umd-plugin-alert">
  <data value="0">warning</data>
  Something went wrong
</template>
```

## TypeScript でのパース例

以下は UMD の HTML 出力から `template.umd-plugin` を抽出し、
関数名・引数・コンテンツを取り出す最小実装例です。

```ts
type UmdPluginNode = {
  name: string;
  args: string[];
  content: string;
  rawClass: string;
};

export function parseUmdPlugins(html: string): UmdPluginNode[] {
  const doc = new DOMParser().parseFromString(html, "text/html");
  const templates = Array.from(
    doc.querySelectorAll<HTMLTemplateElement>("template.umd-plugin"),
  );

  return templates.map((tpl) => {
    const rawClass = tpl.getAttribute("class") ?? "";
    const classes = rawClass.split(/\s+/).filter(Boolean);
    const pluginClass = classes.find(
      (c) => c.startsWith("umd-plugin-") && c !== "umd-plugin",
    );
    const name = pluginClass
      ? pluginClass.replace("umd-plugin-", "")
      : "unknown";

    const argNodes = Array.from(tpl.content.querySelectorAll("data[value]"));
    const args = argNodes
      .sort(
        (a, b) =>
          Number(a.getAttribute("value")) - Number(b.getAttribute("value")),
      )
      .map((n) => n.textContent ?? "");

    const fragment = tpl.content.cloneNode(true) as DocumentFragment;
    fragment.querySelectorAll("data[value]").forEach((n) => n.remove());
    const content = (fragment.textContent ?? "").trim();

    return { name, args, content, rawClass };
  });
}
```

補足:

- `content` は UMD 出力時にエスケープされているため、`textContent` 取得で元のテキスト表現を扱えます。
- 標準プラグイン（`@detail`, `@clear`, `@table`）は `template` を経由しないケースがあるため、別ルートで処理します。

## PHP でのパース例

以下は `DOMDocument + DOMXPath` で `template.umd-plugin` を抽出する例です。

```php
<?php

function parseUmdPlugins(string $html): array
{
    $doc = new DOMDocument('1.0', 'UTF-8');
    libxml_use_internal_errors(true);
    $doc->loadHTML('<?xml encoding="UTF-8">' . $html, LIBXML_HTML_NOIMPLIED | LIBXML_HTML_NODEFDTD);
    libxml_clear_errors();

    $xpath = new DOMXPath($doc);
    $nodes = $xpath->query("//template[contains(concat(' ', normalize-space(@class), ' '), ' umd-plugin ')]");

    $result = [];
    foreach ($nodes as $template) {
        $class = $template->attributes?->getNamedItem('class')?->nodeValue ?? '';
        $classes = preg_split('/\s+/', trim($class));

        $name = 'unknown';
        foreach ($classes as $c) {
            if (str_starts_with($c, 'umd-plugin-') && $c !== 'umd-plugin') {
                $name = substr($c, strlen('umd-plugin-'));
                break;
            }
        }

        $args = [];
        foreach ($template->childNodes as $child) {
            if ($child->nodeName === 'data' && $child->attributes?->getNamedItem('value')) {
                $idx = (int)$child->attributes->getNamedItem('value')->nodeValue;
                $args[$idx] = $child->textContent ?? '';
            }
        }
        ksort($args);
        $args = array_values($args);

        $contentParts = [];
        foreach ($template->childNodes as $child) {
            if ($child->nodeName === 'data' && $child->attributes?->getNamedItem('value')) {
                continue;
            }
            $contentParts[] = $doc->saveHTML($child);
        }

        $content = html_entity_decode(trim(implode('', $contentParts)), ENT_QUOTES | ENT_HTML5, 'UTF-8');

        $result[] = [
            'name' => $name,
            'args' => $args,
            'content' => $content,
            'rawClass' => $class,
        ];
    }

    return $result;
}
```

補足:

- 配列インデックスは `<data value="index">` を優先して復元します。
- 実運用では、`name` ごとにハンドラを分岐し、許可されたプラグインのみ実行してください。

## 関連ドキュメント

- ブロック型プラグインの概要は [block-plugins.md](block-plugins.md) を参照してください。
- インライン型プラグインの一覧は [inline-plugins.md](inline-plugins.md) を参照してください。

## 標準プラグイン

ブロック型:

- `@detail(summary[, open])`
  - `<details><summary>...</summary>...</details>`
- `@clear()`
  - `<div class="clearfix"></div>`
- `@table(...)`
  - テーブルへの Bootstrap バリエーション適用（詳細は [table-features.md](table-features.md)）
- `@math(...)` / `@popover(...)`
  - [umd-extensions.md](umd-extensions.md) 参照

インライン型（`&color()`, `&size()`, `&ruby()`, `&spoiler()` など）の標準プラグイン一覧は
[inline-plugins.md](inline-plugins.md) を参照してください。いずれも未知の関数名と異なり、
汎用の `<template class="umd-plugin-{name}">` を経由せず直接HTMLを出力します。

## 実装の主担当

- `src/extensions/plugins.rs`
- `src/extensions/plugin_markers.rs`
- `src/extensions/conflict_resolver.rs`

## 主なテスト

- `tests/bootstrap_integration.rs`
- `tests/conflict_resolution.rs`（`::: 記法` の統合テストを含む）
- `examples/test_plugin_extended.rs`
- `examples/test_plugin_table.rs`
- `src/extensions/plugin_markers.rs` の単体テスト（`::: 記法` の字句解析）
