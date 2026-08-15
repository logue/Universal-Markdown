# Security Policy

## Reporting a Vulnerability

Please report security issues through GitHub Security Advisory:

- <https://github.com/logue/umd-core/security/advisories/new>

Do not open public issues for unpatched vulnerabilities.

## What to include

Please include:

- Affected version(s)
- Reproduction steps or proof of concept
- Impact assessment
- Suggested mitigation (if available)

## Response process

Maintainers will:

1. Acknowledge receipt.
2. Assess severity and impact.
3. Prepare and release a fix.
4. Publish coordinated disclosure details after remediation.

## Scope

This policy covers the Rust crate in this repository. External integrations and downstream deployments are handled by their respective maintainers.

## Security controls (current behavior)

- Input HTML is always escaped before parsing.
- Dangerous URL schemes are blocked: `javascript:`, `data:`, `vbscript:`, `file:`.
- Suspicious invisible characters (including control and BiDi control characters) are removed from text and URL inputs, including inside fenced code blocks:
  - `U+200B`, `U+200C`, `U+200D`, `U+FEFF`, `U+3164`
  - `U+202A`-`U+202E` (LRE, RLE, PDF, LRO, RLO)
  - `U+2066`-`U+2069` (LRI, RLI, FSI, PDI)
- `ParserOptions.allow_bidi_in_code_blocks` (default: `false`) can opt into preserving BiDi control characters (`U+202A`-`U+202E`, `U+2066`-`U+2069`) inside fenced code blocks (` ``` ` / `~~~`), e.g. to show RTL code samples or Trojan Source-style attack demonstrations verbatim. All other invisible characters remain stripped everywhere, and BiDi characters outside code blocks are always removed regardless of this setting. Enable only when the source content is trusted.
- For directional text use-cases, use UMD inline syntax instead of raw BiDi control characters:
  - `&bdi(ltr){content};` (or `rtl`) for bidirectional isolation with explicit direction
- Allowed blank characters are only:
  - `U+0020` (half-width space)
  - `U+3000` (full-width space)
- Homograph-risk mitigation for external links:
  - For `http/https` links with non-ASCII hostnames or `xn--` punycode labels, UMD adds a visual warning marker (`class="umd-idn-warning-link"`, `data-idn-warning="true"`) and a warning icon element.
  - This is a visual warning and does not block the link.
- ASCII control characters are stripped from document text before parsing:
  - Removed: `U+0000`–`U+0008`, `U+000B`, `U+000C`, `U+000E`–`U+001F`, `U+007F` (DEL)
  - Preserved: `U+0009` (TAB), `U+000A` (LF), `U+000D` (CR) — required for Markdown formatting
  - Content inside fenced code blocks (` ``` ` / `~~~`) is **exempt** from this removal, as control characters may be intentional (e.g., terminal escape sequences in examples)
  - Plugin content is base64-encoded by the conflict resolver before this step, so literal control characters in plugin content are inherently safe from reaching the output unsanitized. However, **plugin authors are responsible for sanitizing their plugin's content** before rendering it on the server side.
