import { defineConfig } from "@rsbuild/core";
import { pluginSass } from "@rsbuild/plugin-sass";

export default defineConfig({
	plugins: [pluginSass()],
	source: {
		entry: {
			"umd-reference": "./scss/umd-reference.scss",
		},
	},
	output: {
		target: "web",
		distPath: {
			root: "./pkg",
		},
		cleanDistPath: false,
		sourceMap: false,
		filename: {
			css: "[name].css",
		},
	},
});
