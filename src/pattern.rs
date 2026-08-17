//! Minimal hand-written pattern helpers used by the format repairers.
//!
//! These replace the small subset of `regex` crate operations that anyrepair
//! previously relied on. Each helper is purpose-built for a specific
//! transformation rather than implementing a general regex engine.

/// Word character: matches regex `\w` (alphanumeric or `_`).
#[inline]
pub(crate) fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// Replace every whole-word occurrence of any string in `words` using `map`.
///
/// Word boundaries follow `\b` semantics: a transition between a word char
/// and any other char (or the start/end of the string). At each scan
/// position, `words` are tried in order and the first match wins, mirroring
/// regex alternation (leftmost-first).
///
/// All `words` must be ASCII; the content may contain UTF-8. Non-ASCII
/// content is passed through verbatim.
pub fn replace_word_any<F: Fn(&str) -> String>(
    content: &str,
    words: &[&str],
    map: F,
) -> String {
    // Fast path: if no word appears as a substring, no whole-word match is possible.
    // `str::contains` is backed by memchr and much faster than the full scan.
    if !words.iter().any(|w| content.contains(w)) {
        return content.to_string();
    }

    let bytes = content.as_bytes();
    let mut out = String::with_capacity(content.len());
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        // Non-ASCII byte: copy the full UTF-8 sequence verbatim.
        if b >= 0x80 {
            let cl = utf8_char_len(b);
            let end = (i + cl).min(bytes.len());
            out.push_str(&content[i..end]);
            i = end;
            continue;
        }
        let boundary_before = i == 0 || !is_word_char_byte(bytes[i - 1]);
        if boundary_before
            && let Some((w_idx, end)) = match_word_bytes(bytes, i, words) {
                out.push_str(&map(words[w_idx]));
                i = end;
                continue;
            }
        out.push(b as char);
        i += 1;
    }
    out
}

/// Byte-level word character check (ASCII only).
#[inline]
fn is_word_char_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

/// UTF-8 sequence length from the leading byte.
#[inline]
fn utf8_char_len(b: u8) -> usize {
    if b < 0x80 { 1 }
    else if b < 0xC0 { 1 } // continuation byte (shouldn't be first)
    else if b < 0xE0 { 2 }
    else if b < 0xF0 { 3 }
    else { 4 }
}

/// If an ASCII word from `words` starts at byte `i` and is followed by a word
/// boundary, return `(word_index, end_byte)`.
fn match_word_bytes(bytes: &[u8], i: usize, words: &[&str]) -> Option<(usize, usize)> {
    let n = bytes.len();
    for (idx, w) in words.iter().enumerate() {
        let wb = w.as_bytes();
        let len = wb.len();
        if i + len <= n && &bytes[i..i + len] == wb {
            let after = i + len;
            let boundary_after = after == n || !is_word_char_byte(bytes[after]);
            if boundary_after {
                return Some((idx, after));
            }
        }
    }
    None
}

/// Replace curly/smart quote characters with straight ASCII equivalents.
pub fn replace_smart_quotes(content: &str) -> String {
    content
        .chars()
        .map(|c| match c {
            '\u{201c}' | '\u{201d}' => '"',
            '\u{2018}' | '\u{2019}' => '\'',
            other => other,
        })
        .collect()
}

/// Replace single-quoted strings `'...'` with double-quoted strings `"..."`.
///
/// Mirrors regex `'([^']*)'`: a single quote, zero or more non-quote chars,
/// then a closing single quote. The inner content is preserved verbatim.
/// Unmatched opening quotes are left intact.
pub fn replace_single_quotes(content: &str) -> String {
    let mut out = String::with_capacity(content.len());
    let mut chars = content.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\'' {
            let mut inner = String::new();
            let mut closed = false;
            for nc in chars.by_ref() {
                if nc == '\'' {
                    closed = true;
                    break;
                }
                inner.push(nc);
            }
            if closed {
                out.push('"');
                out.push_str(&inner);
                out.push('"');
            } else {
                out.push('\'');
                out.push_str(&inner);
            }
        } else {
            out.push(c);
        }
    }
    out
}
