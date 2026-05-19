#!/bin/bash
# WASM Build Script for Universal Markdown
#
# This script builds the Rust library to WebAssembly
# and generates TypeScript bindings.
#
# Prerequisites:
#   - wasm-pack: cargo install wasm-pack
#
# Usage:
#   ./build.sh [dev|release]

set -e

BUILD_TYPE="${1:-release}"

echo "🔧 Building Universal Markdown for WASM (${BUILD_TYPE})..."

# Check if wasm-pack is installed
if ! command -v wasm-pack &> /dev/null; then
    echo "❌ wasm-pack is not installed"
    echo "Install it with: cargo install wasm-pack"
    exit 1
fi

# Build for web target
if [ "$BUILD_TYPE" = "dev" ]; then
    wasm-pack build --target web --dev --out-dir pkg
else
    wasm-pack build --target web --release --out-dir pkg
fi

# Normalize generated package metadata to the author's preferred npm format.
ruby <<'RUBY'
require "json"

path = "pkg/package.json"
unless File.exist?(path)
    warn "⚠️  package.json not found at #{path}; skipped metadata normalization."
    exit 0
end

pkg = JSON.parse(File.read(path))
pkg.delete("collaborators")
pkg["$schema"] = "https://json.schemastore.org/package.json"
pkg["author"] = {
    "name" => "Logue",
    "email" => "logue@hotmail.co.jp",
    "url" => "https://logue.dev/"
}
pkg["homepage"] = "https://github.com/logue/umd-core"
pkg["repository"] = {
    "type" => "git",
    "url" => "git+ssh://git@github.com/logue/umd-core.git"
}
pkg["bugs"] = {
    "url" => "https://github.com/logue/umd-core/issues"
}
pkg["sideEffects"] = false
pkg["name"] = "universal-markdown"

File.write(path, JSON.pretty_generate(pkg) + "\n")
RUBY

echo "✅ Build completed successfully!"
echo "📦 Output directory: pkg/"
echo ""
echo "Usage example:"
echo "  import init, { parse } from './pkg/umd.js';"
echo "  await init();"
echo "  const html = parse('# Hello World');"
