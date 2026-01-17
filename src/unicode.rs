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

/// Remove common zero-width characters that often cause "ghost mismatches".
///
/// This is intentionally conservative and targets the usual culprits:
/// - U+200B ZERO WIDTH SPACE
/// - U+200C ZERO WIDTH NON-JOINER
/// - U+200D ZERO WIDTH JOINER
/// - U+2060 WORD JOINER
/// - U+FEFF ZERO WIDTH NO-BREAK SPACE (BOM)
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
    fn test_remove_zero_width() {
        let text = "a\u{200b}b\u{200c}c\u{200d}d\u{2060}e\u{feff}f";
        assert_eq!(remove_zero_width(text), "abcdef");
    }

    #[test]
    fn test_collapse_whitespace() {
        let text = "  hello\tworld \n  東京  \r\n  Müller  ";
        let collapsed = collapse_whitespace(text);
        assert_eq!(collapsed, "hello world 東京 Müller");
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
    }
}
