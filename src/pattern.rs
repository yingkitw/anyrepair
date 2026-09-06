//! Minimal hand-written pattern helpers used by the format repairers.
//!
//! These replace the small subset of `regex` crate operations that anyrepair
//! previously relied on. Each helper is purpose-built for a specific
//! transformation rather than implementing a general regex engine.

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

/// Unicode word char semantics on bytes: if the char starting at byte `i` is
/// alphanumeric or `_` (identical to `char::is_alphanumeric() || c == '_'`),
/// return its byte length; otherwise `None`. Callers advance by the returned
/// length, which keeps the scan on char boundaries.
#[inline]
pub(crate) fn word_char_len_at(content: &str, bytes: &[u8], i: usize) -> Option<usize> {
    let b = bytes[i];
    if b.is_ascii_alphanumeric() || b == b'_' {
        Some(1)
    } else if b >= 0x80 {
        let c = content[i..].chars().next().unwrap();
        if c.is_alphanumeric() {
            Some(c.len_utf8())
        } else {
            None
        }
    } else {
        None
    }
}

/// True if the char immediately before byte `i` (a char boundary) is a word
/// char. Walks back over continuation bytes to find the previous char start.
#[inline]
pub(crate) fn prev_char_is_word(content: &str, bytes: &[u8], i: usize) -> bool {
    let mut j = i - 1;
    while j > 0 && bytes[j] & 0xC0 == 0x80 {
        j -= 1;
    }
    word_char_len_at(content, bytes, j).is_some()
}

/// Skip ASCII and Unicode whitespace starting at byte `i`, returning the next
/// byte index. Equivalent to the `is_whitespace()` char loop.
#[inline]
pub(crate) fn skip_whitespace_bytes(content: &str, bytes: &[u8], mut i: usize) -> usize {
    let n = bytes.len();
    while i < n {
        let b = bytes[i];
        if b == b' ' || b == b'\n' || b == b'\t' || b == b'\r' {
            i += 1;
        } else if b >= 0x80 {
            let c = content[i..].chars().next().unwrap();
            if c.is_whitespace() {
                i += c.len_utf8();
            } else {
                break;
            }
        } else {
            break;
        }
    }
    i
}

/// Append the char starting at byte `i` (full UTF-8 sequence if non-ASCII) to
/// `out`, returning the next byte index. Safe on char boundaries only.
#[inline]
pub(crate) fn push_char(content: &str, bytes: &[u8], out: &mut String, i: usize) -> usize {
    let b = bytes[i];
    if b >= 0x80 {
        let cl = utf8_char_len(b);
        let end = (i + cl).min(bytes.len());
        out.push_str(&content[i..end]);
        end
    } else {
        out.push(b as char);
        i + 1
    }
}

/// UTF-8 sequence length from the leading byte.
///
/// A leading continuation byte (0x80–0xBF) is invalid UTF-8 but is treated as
/// a single byte so the scanner can resync without over-reading.
#[inline]
fn utf8_char_len(b: u8) -> usize {
    if b < 0xC0 { 1 } // ASCII (0x00–0x7F) or invalid leading continuation byte (0x80–0xBF)
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

#[cfg(test)]
mod tests {
    use super::*;

    // --- Tests for replace_word_any with non-ASCII content ---
    // These kill mutants in utf8_char_len and the non-ASCII branch of
    // replace_word_any that were missed by cargo-mutants.

    #[test]
    fn test_replace_word_any_non_ascii_before_target() {
        // "café" ends with é (2-byte UTF-8). "true" follows a space.
        // If utf8_char_len returns 1 for é, the scanner splits the
        // multi-byte sequence and may miss the word boundary.
        let input = "café true";
        let result = replace_word_any(input, &["true"], |_| "false".to_string());
        assert_eq!(result, "café false");
    }

    #[test]
    fn test_replace_word_any_non_ascii_after_target() {
        let input = "true café";
        let result = replace_word_any(input, &["true"], |_| "false".to_string());
        assert_eq!(result, "false café");
    }

    #[test]
    fn test_replace_word_any_non_ascii_at_start() {
        // 3-byte UTF-8 sequence (日本) at position 0, then target word.
        // Tests the i=0 + non-ASCII path in replace_word_any.
        let input = "日本 true";
        let result = replace_word_any(input, &["true"], |_| "false".to_string());
        assert_eq!(result, "日本 false");
    }

    #[test]
    fn test_replace_word_any_4byte_utf8_at_start() {
        // 😀 is a 4-byte UTF-8 sequence. If utf8_char_len returns 1 or 2,
        // the scanner would split the emoji and corrupt the output.
        let input = "😀 null";
        let result = replace_word_any(input, &["null"], |_| "None".to_string());
        assert_eq!(result, "😀 None");
    }

    #[test]
    fn test_replace_word_any_non_ascii_adjacent_no_space() {
        // Target word immediately after a multi-byte char with no space.
        // The word boundary check uses is_word_char_byte on the byte
        // before the match. A non-ASCII byte (>= 0x80) is not a word char,
        // so the boundary is valid.
        let input = "cafétrue";
        let result = replace_word_any(input, &["true"], |_| "false".to_string());
        assert_eq!(result, "caféfalse");
    }

    #[test]
    fn test_replace_word_any_preserves_non_ascii_in_output() {
        // Ensure non-ASCII content is copied verbatim through the output.
        let input = "naïve true résumé";
        let result = replace_word_any(input, &["true"], |_| "false".to_string());
        assert_eq!(result, "naïve false résumé");
    }

    #[test]
    fn test_replace_word_any_multiple_words_with_non_ascii() {
        let input = "café true null 日本 false";
        let result = replace_word_any(input, &["true", "null", "false"], |w| {
            format!("<{}>", w)
        });
        assert_eq!(result, "café <true> <null> 日本 <false>");
    }

    // --- Tests for replace_smart_quotes and replace_single_quotes ---

    #[test]
    fn test_replace_smart_quotes_basic() {
        let input = "\u{201C}hello\u{201D}";
        let result = replace_smart_quotes(input);
        assert_eq!(result, "\"hello\"");
    }

    #[test]
    fn test_replace_single_quotes_basic() {
        let input = "'hello'";
        let result = replace_single_quotes(input);
        assert_eq!(result, "\"hello\"");
    }

    #[test]
    fn test_replace_single_quotes_unclosed() {
        let input = "'hello";
        let result = replace_single_quotes(input);
        assert_eq!(result, "'hello");
    }
}
