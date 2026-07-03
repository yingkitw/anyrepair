# Troubleshooting Guide

Common issues, error messages, and fixes when using anyrepair.

## Table of Contents

- [CLI Issues](#cli-issues)
- [Format Detection Problems](#format-detection-problems)
- [Repair Quality Issues](#repair-quality-issues)
- [Library API Issues](#library-api-issues)
- [Streaming Issues](#streaming-issues)
- [MCP Server Issues](#mcp-server-issues)

---

## CLI Issues

### "Unknown format: X"

**Cause:** The `--format` value doesn't match any supported format.

**Fix:** Use one of: `json`, `yaml`, `yml`, `markdown`, `md`, `xml`, `toml`, `csv`, `ini`, `diff`, `properties`, `env`. Format names are case-insensitive.

### "Confidence X% below minimum threshold Y%"

**Cause:** The `--min-confidence` flag was set and the repair confidence fell below the threshold.

**Fix:** Lower the threshold or inspect the input for severe damage. Use `--explain` to see which strategies were applied. Use `--confidence` to check the score without thresholding.

### No output when running `repair`

**Cause:** You may have `--dry-run` set, or output is going to a file via `--output`.

**Fix:** Remove `--dry-run` if you want output written. Check `--output` path. Use `--verbose` for diagnostic messages.

### Output differs from expected (whitespace changes)

**Cause:** Some repair strategies normalize whitespace (e.g., `FixWhitespaceAroundEquals` in INI/properties/env removes spaces around `=`).

**Fix:** This is expected behavior. The repairer prioritizes structural validity over format preservation. If the input is already valid, it passes through unchanged.

---

## Format Detection Problems

### JSON detected as YAML

**Cause:** JSON content that uses YAML-like constructs (e.g., unquoted strings, colons without braces) may trigger YAML detection first. The detection order is: JSON → Diff → YAML → XML → TOML → CSV → Env → Properties → INI → Markdown.

**Fix:** Use `--format json` to force JSON repair mode.

### Env file detected as YAML

**Cause:** Env values containing `:` (e.g., `DATABASE_URL=postgres://localhost`) trigger YAML detection because `is_env_like` excludes content with colons.

**Fix:** Use `--format env` to force env repair mode.

### Properties file detected as INI

**Cause:** Properties files with `[section]` headers can match INI detection. Properties files with dot-notation keys (`app.name=value`) are correctly detected.

**Fix:** Use `--format properties` to force properties repair mode.

### "No format detected" — falls back to Markdown

**Cause:** The content doesn't match any format heuristic. `repair()` falls back to the Markdown repairer.

**Fix:** Use `--format <fmt>` to specify the format explicitly.

---

## Repair Quality Issues

### Repair doesn't fix my broken JSON

**Cause:** The JSON damage may be beyond what heuristic strategies can handle. The JSON repairer handles: trailing commas, unquoted keys, single quotes, missing brackets, JS-style comments.

**Fix:** Check `--explain` output to see which strategies ran. If the input has structural issues beyond the supported strategies, manual intervention may be needed.

### Repair changes valid content

**Cause:** The validator may consider the content invalid due to strict checks, triggering strategies that modify it.

**Fix:** Use `--dry-run --diff` to preview changes before writing. If the content is already valid, the validator should return it unchanged. Report false positives as bugs.

### Confidence is 0% but content looks repaired

**Cause:** The `GenericRepairer::confidence()` returns 1.0 if the validator says valid, 0.0 otherwise. Some format-specific repairers override `confidence()` with heuristic scoring. If the repaired content still fails validation, confidence will be low.

**Fix:** Check `--explain` to see if strategies were applied. The content may be partially repaired but still invalid.

### Idempotency violation — repairing twice gives different output

**Cause:** A strategy may not be idempotent (applying it twice changes the result). This is a bug.

**Fix:** Report it. Golden master tests include idempotency checks for key formats.

---

## Library API Issues

### `create_repairer("yml")` works but `create_repairer("YAML")` doesn't

**Cause:** Both should work — `normalize_format` handles case-insensitive matching and aliases (`yml` → `yaml`, `md` → `markdown`).

**Fix:** If you encounter this, it's a bug. Ensure you're using the latest version.

### `repair()` returns the same content unchanged

**Cause:** The content is either already valid (validator returns `true`) or no format was detected and the Markdown repairer found nothing to fix.

**Fix:** Use `detect_format()` to check which format was detected. Use `create_validator(fmt)?.is_valid(content)` to check validity.

### `repair_with_explanations()` returns empty strategy list

**Cause:** The content was already valid, so no strategies were applied. Or the content was empty.

**Fix:** Check `needs_repair()` before calling repair. Empty input returns `("", [])`.

---

## Streaming Issues

### Streaming produces different output than non-streaming repair

**Cause:** Streaming repair processes content line-by-line, which may miss multi-line issues (e.g., unclosed JSON brackets spanning multiple lines). Non-streaming repair sees the full content.

**Fix:** For files that fit in memory, use `repair()` instead of streaming. For large files, ensure the damage is line-local (e.g., trailing commas, unquoted keys).

### "Buffer too small" or corrupted output

**Cause:** The buffer size is smaller than the longest line, causing lines to be split across reads.

**Fix:** Increase `--buffer-size` (default: 8192 bytes). Set it to at least the size of the longest line in your file.

---

## MCP Server Issues

### MCP client can't connect to the server

**Cause:** The MCP server expects newline-delimited JSON on stdin/stdout. Ensure your client is sending properly formatted JSON-RPC messages.

**Fix:** Test with: `echo '{"tool":"repair","input":{"content":"{\"key\":\"value\"}"}}' | anyrepair mcp`

### MCP repair returns "missing 'content' field"

**Cause:** The input JSON object doesn't have a `content` field.

**Fix:** Ensure the input JSON has `"content"` as a string field: `{"content": "...", "format": "json"}`.

---

## Getting Help

- Check the [SPEC.md](SPEC.md) for protocol and API details
- Check the [ARCHITECTURE.md](ARCHITECTURE.md) for module relationships
- Run with `--verbose` for diagnostic output
- Use `--explain` to see which repair strategies were applied
- Use `--diff --dry-run` to preview changes without writing
