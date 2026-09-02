import { cp, mkdir, readdir, readFile, rm, writeFile } from "node:fs/promises";

const outputDirectory = "pkg";
const rsbuildDirectory = `${outputDirectory}/static`;
const sourceCss = `${rsbuildDirectory}/css/umd-reference.css`;
const targetCss = `${outputDirectory}/umd-reference.css`;
const sourceFontDirectory = `${rsbuildDirectory}/font`;
const targetFontDirectory = `${outputDirectory}/fonts`;

await mkdir(outputDirectory, { recursive: true });

let css = await readFile(sourceCss, "utf8");

// Font assets (e.g. the bundled Twemoji webfont) are emitted under
// static/font/ by rsbuild with an absolute /static/font/ URL baked into
// the CSS. static/ is discarded below, so relocate the fonts next to the
// flattened CSS and rewrite the reference to match.
const fontFiles = await readdir(sourceFontDirectory).catch(() => []);
if (fontFiles.length > 0) {
  await mkdir(targetFontDirectory, { recursive: true });
  await Promise.all(
    fontFiles.map((file) =>
      cp(`${sourceFontDirectory}/${file}`, `${targetFontDirectory}/${file}`),
    ),
  );
  css = css.replaceAll("/static/font/", "./fonts/");
}

await writeFile(targetCss, css);
await rm(rsbuildDirectory, { recursive: true, force: true });
await rm(`${outputDirectory}/umd-reference.html`, { force: true });
await rm(`${outputDirectory}/umd-reference.js`, { force: true });
