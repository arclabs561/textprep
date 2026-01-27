//! Unicode normalization utilities.

use unicode_normalization::UnicodeNormalization;

pub fn nfc(text: &str) -> String {
    text.nfc().collect()
}

pub fn nfd(text: &str) -> String {
    text.nfd().collect()
}

pub fn nfkc(text: &str) -> String {
    text.nfkc().collect()
}

pub fn nfkd(text: &str) -> String {
    text.nfkd().collect()
}

/// Normalize newlines to LF (`\n`).
///
/// Converts:
/// - Windows CRLF (`\r\n`) → `\n`
/// - Old Mac CR (`\r`) → `\n`
pub fn normalize_newlines(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\r' {
            if chars.peek() == Some(&'\n') {
                let _ = chars.next();
            }
            out.push('\n');
        } else {
            out.push(c);
        }
    }
    out
}

/// Like [`normalize_newlines`], but writes into an existing `String`.
pub fn normalize_newlines_into(text: &str, out: &mut String) {
    out.clear();
    out.reserve(text.len());
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\r' {
            if chars.peek() == Some(&'\n') {
                let _ = chars.next();
            }
            out.push('\n');
        } else {
            out.push(c);
        }
    }
}

/// Trim each line (preserving internal spacing) and drop blank lines.
///
/// This is a “document cleaning” primitive:
/// - preserves case
/// - preserves internal whitespace runs (e.g. `"Hello   World"`)
/// - normalizes newlines to `\n`
/// - trims leading/trailing whitespace per line
/// - removes empty lines
///
/// If you want a *search key* (casefold + diacritics stripping + whitespace collapse),
/// use `crate::scrub_with` and an explicit `ScrubConfig`.
pub fn trim_lines_preserve_spaces(text: &str) -> String {
    let normalized = normalize_newlines(text);
    normalized
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

/// Remove common zero-width characters that often cause "ghost mismatches".
///
/// This is intentionally conservative and targets the usual culprits:
/// - U+200B ZERO WIDTH SPACE
/// - U+200C ZERO WIDTH NON-JOINER
/// - U+200D ZERO WIDTH JOINER
/// - U+2060 WORD JOINER
/// - U+FEFF ZERO WIDTH NO-BREAK SPACE (BOM)
///
/// Warning: some of these characters are semantically meaningful in certain scripts
/// (e.g. ZWNJ/ZWJ) or sequences (emoji ZWJ). Treat this as a normalization step for
/// matching/search, not as a general-purpose text rewriting.
pub fn remove_zero_width(text: &str) -> String {
    text.chars()
        .filter(|&c| {
            !matches!(
                c,
                '\u{200B}' | '\u{200C}' | '\u{200D}' | '\u{2060}' | '\u{FEFF}'
            )
        })
        .collect()
}

/// Like [`remove_zero_width`], but writes into an existing `String`.
pub fn remove_zero_width_into(text: &str, out: &mut String) {
    out.clear();
    out.reserve(text.len());
    out.extend(text.chars().filter(|&c| {
        !matches!(
            c,
            '\u{200B}' | '\u{200C}' | '\u{200D}' | '\u{2060}' | '\u{FEFF}'
        )
    }));
}

/// Check whether text contains any of the "common zero-width" characters targeted by
/// [`remove_zero_width`].
#[must_use]
pub fn contains_zero_width(text: &str) -> bool {
    text.chars().any(|c| {
        matches!(
            c,
            '\u{200B}' | '\u{200C}' | '\u{200D}' | '\u{2060}' | '\u{FEFF}'
        )
    })
}

/// Return all "common zero-width" characters found, with **character offsets**.
///
/// This is the detection/reporting counterpart to [`remove_zero_width`].
#[must_use]
pub fn zero_width_with_offsets(text: &str) -> Vec<(usize, char)> {
    text.chars()
        .enumerate()
        .filter_map(|(i, c)| {
            if matches!(
                c,
                '\u{200B}' | '\u{200C}' | '\u{200D}' | '\u{2060}' | '\u{FEFF}'
            ) {
                Some((i, c))
            } else {
                None
            }
        })
        .collect()
}

/// Remove Unicode bidirectional control characters.
///
/// This targets the classes of control characters used in "Trojan Source"-style
/// display obfuscation attacks (plus the common LRM/RLM marks):
/// - U+202A..U+202E (embeddings + overrides)
/// - U+2066..U+2069 (isolates)
/// - U+200E, U+200F (LRM/RLM)
/// - U+061C (ARABIC LETTER MARK, ALM)
///
/// This is a *policy* tool: for some natural-language text you may want to keep these.
pub fn remove_bidi_controls(text: &str) -> String {
    text.chars()
        .filter(|&c| {
            !matches!(
                c,
                '\u{202A}'
                    | '\u{202B}'
                    | '\u{202C}'
                    | '\u{202D}'
                    | '\u{202E}'
                    | '\u{2066}'
                    | '\u{2067}'
                    | '\u{2068}'
                    | '\u{2069}'
                    | '\u{200E}'
                    | '\u{200F}'
                    | '\u{061C}'
            )
        })
        .collect()
}

/// Like [`remove_bidi_controls`], but writes into an existing `String`.
pub fn remove_bidi_controls_into(text: &str, out: &mut String) {
    out.clear();
    out.reserve(text.len());
    out.extend(text.chars().filter(|&c| {
        !matches!(
            c,
            '\u{202A}'
                | '\u{202B}'
                | '\u{202C}'
                | '\u{202D}'
                | '\u{202E}'
                | '\u{2066}'
                | '\u{2067}'
                | '\u{2068}'
                | '\u{2069}'
                | '\u{200E}'
                | '\u{200F}'
                | '\u{061C}'
        )
    }));
}

/// Check whether text contains bidi control characters.
#[must_use]
pub fn contains_bidi_controls(text: &str) -> bool {
    text.chars().any(|c| {
        matches!(
            c,
            '\u{202A}'
                | '\u{202B}'
                | '\u{202C}'
                | '\u{202D}'
                | '\u{202E}'
                | '\u{2066}'
                | '\u{2067}'
                | '\u{2068}'
                | '\u{2069}'
                | '\u{200E}'
                | '\u{200F}'
                | '\u{061C}'
        )
    })
}

/// Return all bidi control characters found, with **character offsets**.
///
/// This is useful when you want to *detect and report* (like `rustc`'s
/// `text_direction_codepoint_in_comment` / `text_direction_codepoint_in_literal` lints)
/// instead of silently stripping.
///
/// Offsets are in **characters**, not bytes.
#[must_use]
pub fn bidi_controls_with_offsets(text: &str) -> Vec<(usize, char)> {
    text.chars()
        .enumerate()
        .filter_map(|(i, c)| {
            if matches!(
                c,
                '\u{202A}'
                    | '\u{202B}'
                    | '\u{202C}'
                    | '\u{202D}'
                    | '\u{202E}'
                    | '\u{2066}'
                    | '\u{2067}'
                    | '\u{2068}'
                    | '\u{2069}'
                    | '\u{200E}'
                    | '\u{200F}'
                    | '\u{061C}'
            ) {
                Some((i, c))
            } else {
                None
            }
        })
        .collect()
}

/// Collapse all Unicode whitespace into single ASCII spaces.
///
/// - Converts any `char::is_whitespace()` run into a single `' '`.
/// - Trims leading/trailing whitespace (by construction).
///
/// This intentionally loses newlines. If you want to preserve newlines,
/// normalize newlines first and apply a line-wise collapse yourself.
pub fn collapse_whitespace(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut in_ws = true; // treat start as whitespace to avoid leading space
    for c in text.chars() {
        if c.is_whitespace() {
            in_ws = true;
            continue;
        }
        if in_ws && !out.is_empty() {
            out.push(' ');
        }
        in_ws = false;
        out.push(c);
    }
    out
}

/// Like [`collapse_whitespace`], but writes into an existing `String`.
pub fn collapse_whitespace_into(text: &str, out: &mut String) {
    out.clear();
    out.reserve(text.len());

    let mut in_ws = true; // treat start as whitespace to avoid leading space
    for c in text.chars() {
        if c.is_whitespace() {
            in_ws = true;
            continue;
        }
        if in_ws && !out.is_empty() {
            out.push(' ');
        }
        in_ws = false;
        out.push(c);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn test_nfc() {
        let decomposed = "a\u{0308}";
        let normalized = nfc(decomposed);
        assert_eq!(normalized, "ä");
    }

    #[test]
    fn test_normalize_newlines() {
        let text = "Line 1\r\nLine 2\rLine 3\nLine 4";
        let normalized = normalize_newlines(text);
        assert!(!normalized.contains("\r\n"));
        assert!(!normalized.contains('\r'));
        assert!(normalized.contains('\n'));
    }

    #[test]
    fn test_normalize_newlines_into_matches() {
        let text = "Line 1\r\nLine 2\rLine 3\nLine 4";
        let expected = normalize_newlines(text);
        let mut out = String::new();
        normalize_newlines_into(text, &mut out);
        assert_eq!(out, expected);
    }

    #[test]
    fn test_trim_lines_preserve_spaces() {
        // Mixed scripts + diacritics + extra whitespace.
        let text = "  Hello   World  \r\n\r\n  東京  \n\n  Müller  ";
        let out = trim_lines_preserve_spaces(text);
        assert_eq!(out, "Hello   World\n東京\nMüller");
    }

    #[test]
    fn test_remove_zero_width() {
        let text = "a\u{200b}b\u{200c}c\u{200d}d\u{2060}e\u{feff}f";
        assert!(contains_zero_width(text));
        assert_eq!(
            zero_width_with_offsets(text),
            vec![
                (1, '\u{200B}'),
                (3, '\u{200C}'),
                (5, '\u{200D}'),
                (7, '\u{2060}'),
                (9, '\u{FEFF}')
            ]
        );
        assert_eq!(remove_zero_width(text), "abcdef");
        assert!(!contains_zero_width(&remove_zero_width(text)));

        let mut out = String::new();
        remove_zero_width_into(text, &mut out);
        assert_eq!(out, "abcdef");
    }

    #[test]
    fn test_remove_bidi_controls() {
        // Mix embeddings/overrides + isolates + marks.
        let text = "a\u{202e}\u{2066}b\u{2069}\u{202c}\u{200f}c";
        assert!(contains_bidi_controls(text));
        assert_eq!(
            bidi_controls_with_offsets(text),
            vec![
                (1, '\u{202E}'),
                (2, '\u{2066}'),
                (4, '\u{2069}'),
                (5, '\u{202C}'),
                (6, '\u{200F}')
            ]
        );
        assert_eq!(remove_bidi_controls(text), "abc");
        assert!(!contains_bidi_controls(&remove_bidi_controls(text)));

        let mut out = String::new();
        remove_bidi_controls_into(text, &mut out);
        assert_eq!(out, "abc");
    }

    #[test]
    fn test_remove_bidi_controls_includes_alm() {
        let text = "a\u{061c}b";
        assert!(contains_bidi_controls(text));
        assert_eq!(bidi_controls_with_offsets(text), vec![(1, '\u{061C}')]);
        assert_eq!(remove_bidi_controls(text), "ab");
    }

    #[test]
    fn test_collapse_whitespace() {
        let text = "  hello\tworld \n  東京  \r\n  Müller  ";
        let collapsed = collapse_whitespace(text);
        assert_eq!(collapsed, "hello world 東京 Müller");
    }

    #[test]
    fn test_collapse_whitespace_into_matches() {
        let text = "  hello\tworld \n  東京  \r\n  Müller  ";
        let expected = collapse_whitespace(text);
        let mut out = String::new();
        collapse_whitespace_into(text, &mut out);
        assert_eq!(out, expected);
    }

    proptest! {
        #[test]
        fn prop_remove_zero_width_removes_all_targets(s in ".*") {
            let out = remove_zero_width(&s);
            // Provide explicit messages: proptest's default message uses `stringify!(...)`
            // which includes `\u{...}` and can be interpreted as formatting braces.
            prop_assert!(!out.contains('\u{200B}'), "ZWSP (U+200B) not removed");
            prop_assert!(!out.contains('\u{200C}'), "ZWNJ (U+200C) not removed");
            prop_assert!(!out.contains('\u{200D}'), "ZWJ (U+200D) not removed");
            prop_assert!(!out.contains('\u{2060}'), "WORD JOINER (U+2060) not removed");
            prop_assert!(!out.contains('\u{FEFF}'), "BOM (U+FEFF) not removed");
        }

        #[test]
        fn prop_collapse_whitespace_has_no_runs(s in ".*") {
            let out = collapse_whitespace(&s);
            // By construction, output has no leading/trailing whitespace and no internal whitespace runs.
            prop_assert!(!out.starts_with(char::is_whitespace), "leading whitespace present");
            prop_assert!(!out.ends_with(char::is_whitespace), "trailing whitespace present");
            prop_assert!(!out.contains("  "), "double-space present");
            prop_assert!(!out.contains('\n'), "newline present");
            prop_assert!(!out.contains('\t'), "tab present");
            prop_assert!(!out.contains('\r'), "CR present");
        }

        #[test]
        fn prop_normalize_newlines_into_equivalent(s in ".*") {
            let expected = normalize_newlines(&s);
            let mut out = String::new();
            normalize_newlines_into(&s, &mut out);
            prop_assert_eq!(out, expected);
        }

        #[test]
        fn prop_collapse_whitespace_into_equivalent(s in ".*") {
            let expected = collapse_whitespace(&s);
            let mut out = String::new();
            collapse_whitespace_into(&s, &mut out);
            prop_assert_eq!(out, expected);
        }
    }
}
