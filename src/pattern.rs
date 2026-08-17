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
pub fn replace_word_any<F: Fn(&str) -> String>(
    content: &str,
    words: &[&str],
    map: F,
) -> String {
    let chars: Vec<char> = content.chars().collect();
    let n = chars.len();
    let mut out = String::with_capacity(content.len());
    let mut i = 0;
    while i < n {
        let boundary_before = i == 0 || !is_word_char(chars[i - 1]);
        if boundary_before
            && let Some((w, len)) = match_word(&chars, i, words) {
                out.push_str(&map(w));
                i += len;
                continue;
            }
        out.push(chars[i]);
        i += 1;
    }
    out
}

/// If a word from `words` starts at `i` and is followed by a word boundary,
/// return `(word, char_count)`.
fn match_word<'a>(chars: &[char], i: usize, words: &[&'a str]) -> Option<(&'a str, usize)> {
    let n = chars.len();
    for w in words {
        let wc: Vec<char> = w.chars().collect();
        let len = wc.len();
        if i + len <= n && chars[i..i + len] == wc[..] {
            let after = i + len;
            let boundary_after = after == n || !is_word_char(chars[after]);
            if boundary_after {
                return Some((w, len));
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
