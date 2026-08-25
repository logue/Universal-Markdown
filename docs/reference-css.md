# Reference CSS

UMDの参照CSSは、Rust/WASMコードとは分離した `scss/` を編集元にします。RsbuildとSassで `pkg/umd-reference.css` を生成します。

## Build

```bash
pnpm install
pnpm run build:css
```

WASMとCSSをまとめてビルドする場合は、リポジトリルートの `./build.sh release` を実行します。

## Host colors

意味付きカラーは固定色ではなく、ホスト側の変数で上書きできます。

```css
.my-theme {
  --umd-host-primary: #6750a4;
  --umd-host-success: #217a4b;
  --umd-host-danger: #b42318;
  --umd-host-warning: #a15c00;
}
```

色相そのものを表す `text-red` や `bg-yellow` などは、既存のUMDクラス名を維持します。
