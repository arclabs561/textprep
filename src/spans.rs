//! Span boundary cleanup for NER post-processing.
//!
//! Entity spans extracted from text often include trailing punctuation or
//! possessive markers that leaked from word-boundary tokenization. This module
//! provides reusable primitives for trimming those artifacts while reporting
//! the number of characters removed (so callers can adjust offsets).

/// Punctuation characters commonly attached to NER spans by tokenizers.
const SPAN_PUNCT: &[char] = &['.', ',', ';', ':', '!', '?', ')', ']'];

/// Trim trailing punctuation and possessives from a span's text.
///
/// Returns `(cleaned_text, chars_trimmed_from_end)`.
///
/// The trimming order is:
/// 1. Trailing punctuation characters (`.`, `,`, `;`, `:`, `!`, `?`, `)`, `]`)
/// 2. Trailing possessives (`'s` and `\u{2019}s`)
///
/// # Examples
///
/// ```
/// use textprep::spans::clean_span_tail;
///
/// assert_eq!(clean_span_tail("Seattle."), ("Seattle", 1));
/// assert_eq!(clean_span_tail("Obama's"), ("Obama", 2));
/// assert_eq!(clean_span_tail("Obama\u{2019}s"), ("Obama", 2));
/// assert_eq!(clean_span_tail("hello"), ("hello", 0));
/// ```
pub fn clean_span_tail(text: &str) -> (&str, usize) {
    let original_chars = text.chars().count();
    let mut s = text;

    // Strip trailing punctuation.
    s = s.trim_end_matches(SPAN_PUNCT);

    // Strip possessives (ASCII straight quote and Unicode right single quote).
    s = s.trim_end_matches("'s");
    s = s.trim_end_matches("\u{2019}s");

    let cleaned_chars = s.chars().count();
    (s, original_chars - cleaned_chars)
}

/// Trim leading punctuation from a span's text.
///
/// Returns `(cleaned_text, chars_trimmed_from_start)`.
///
/// # Examples
///
/// ```
/// use textprep::spans::clean_span_head;
///
/// assert_eq!(clean_span_head(".Seattle"), ("Seattle", 1));
/// assert_eq!(clean_span_head("hello"), ("hello", 0));
/// ```
pub fn clean_span_head(text: &str) -> (&str, usize) {
    let original_chars = text.chars().count();
    let s = text.trim_start_matches(SPAN_PUNCT);
    let cleaned_chars = s.chars().count();
    (s, original_chars - cleaned_chars)
}

/// Clean both ends of a span. Returns `(cleaned_text, start_delta, end_delta)`.
///
/// `start_delta` is the number of characters trimmed from the start (add to start offset).
/// `end_delta` is the number of characters trimmed from the end (subtract from end offset).
///
/// # Examples
///
/// ```
/// use textprep::spans::clean_span_boundary;
///
/// let (text, head, tail) = clean_span_boundary(".,Seattle!?");
/// assert_eq!(text, "Seattle");
/// assert_eq!(head, 2);
/// assert_eq!(tail, 2);
/// ```
pub fn clean_span_boundary(text: &str) -> (&str, usize, usize) {
    let (s, head_trimmed) = clean_span_head(text);
    let (s, tail_trimmed) = clean_span_tail(s);
    (s, head_trimmed, tail_trimmed)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── clean_span_tail ──────────────────────────────────────────

    #[test]
    fn tail_trailing_period() {
        assert_eq!(clean_span_tail("Seattle."), ("Seattle", 1));
    }

    #[test]
    fn tail_multiple_punct() {
        assert_eq!(clean_span_tail("wow!?"), ("wow", 2));
    }

    #[test]
    fn tail_brackets() {
        // `)` and `.` are both in the punct set, so both are trimmed.
        assert_eq!(clean_span_tail("Inc.)"), ("Inc", 2));
        assert_eq!(clean_span_tail("array]"), ("array", 1));
    }

    #[test]
    fn tail_possessive_ascii() {
        assert_eq!(clean_span_tail("Obama's"), ("Obama", 2));
    }

    #[test]
    fn tail_possessive_curly() {
        assert_eq!(clean_span_tail("Obama\u{2019}s"), ("Obama", 2));
    }

    #[test]
    fn tail_possessive_plus_punct() {
        // Punctuation stripped first, then possessive.
        assert_eq!(clean_span_tail("Elon Musk's."), ("Elon Musk", 3));
    }

    #[test]
    fn tail_no_op() {
        assert_eq!(clean_span_tail("hello"), ("hello", 0));
    }

    #[test]
    fn tail_empty() {
        assert_eq!(clean_span_tail(""), ("", 0));
    }

    #[test]
    fn tail_all_punct() {
        assert_eq!(clean_span_tail("..."), ("", 3));
    }

    #[test]
    fn tail_unicode_text_with_punct() {
        assert_eq!(clean_span_tail("東京."), ("東京", 1));
    }

    // ── clean_span_head ──────────────────────────────────────────

    #[test]
    fn head_leading_period() {
        assert_eq!(clean_span_head(".Seattle"), ("Seattle", 1));
    }

    #[test]
    fn head_multiple() {
        assert_eq!(clean_span_head(".,;Seattle"), ("Seattle", 3));
    }

    #[test]
    fn head_no_op() {
        assert_eq!(clean_span_head("hello"), ("hello", 0));
    }

    #[test]
    fn head_empty() {
        assert_eq!(clean_span_head(""), ("", 0));
    }

    // ── clean_span_boundary ──────────────────────────────────────

    #[test]
    fn boundary_both_ends() {
        let (text, h, t) = clean_span_boundary(".,Seattle!?");
        assert_eq!(text, "Seattle");
        assert_eq!(h, 2);
        assert_eq!(t, 2);
    }

    #[test]
    fn boundary_only_trailing() {
        let (text, h, t) = clean_span_boundary("Seattle.");
        assert_eq!(text, "Seattle");
        assert_eq!(h, 0);
        assert_eq!(t, 1);
    }

    #[test]
    fn boundary_only_leading() {
        let (text, h, t) = clean_span_boundary(".Seattle");
        assert_eq!(text, "Seattle");
        assert_eq!(h, 1);
        assert_eq!(t, 0);
    }

    #[test]
    fn boundary_clean() {
        let (text, h, t) = clean_span_boundary("Seattle");
        assert_eq!(text, "Seattle");
        assert_eq!(h, 0);
        assert_eq!(t, 0);
    }

    #[test]
    fn boundary_empty() {
        let (text, h, t) = clean_span_boundary("");
        assert_eq!(text, "");
        assert_eq!(h, 0);
        assert_eq!(t, 0);
    }
}
