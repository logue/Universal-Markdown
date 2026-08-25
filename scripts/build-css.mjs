import { cp, mkdir, rm } from "node:fs/promises";

const outputDirectory = "pkg";
const rsbuildDirectory = `${outputDirectory}/static`;
const sourceCss = `${rsbuildDirectory}/css/umd-reference.css`;
const targetCss = `${outputDirectory}/umd-reference.css`;

await mkdir(outputDirectory, { recursive: true });
await cp(sourceCss, targetCss);
await rm(rsbuildDirectory, { recursive: true, force: true });
await rm(`${outputDirectory}/umd-reference.html`, { force: true });
await rm(`${outputDirectory}/umd-reference.js`, { force: true });
