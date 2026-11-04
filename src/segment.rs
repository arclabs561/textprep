//! Sentence and word boundaries for running text, as byte ranges.
//!
//! [`crate::tokenize::sentences`] and [`crate::tokenize::words`] return the
//! plain UAX #29 segments. The functions here start from the same UAX #29
//! boundaries (from `unicode-segmentation`) and adapt them to prose:
//! hard-wrapped lines, abbreviations and hyphenated compounds. They return
//! byte ranges into the input, so `&text[range]` recovers each segment.

use std::collections::HashSet;
use std::ops::Range;

use unicode_segmentation::UnicodeSegmentation;

/// Byte ranges of the sentences of `text`, without surrounding whitespace.
///
/// A sentence ends at a UAX #29 sentence boundary: after `.`, `!`, `?` and
/// the other sentence terminals, including any closing quotes or brackets
/// that follow them (`2014." Later` splits after the quote), but not before
/// a lowercase word (`see fig. two`). Two changes adapt UAX #29 to running
/// text:
///
/// - A single line break does not end a sentence, since text is often hard
///   wrapped. A break that UAX #29 places only because of a line break is
///   kept when the line break is part of a blank line. Otherwise it is kept
///   only where UAX #29 would also break if the line break were a space.
/// - A period after a known English abbreviation (`Mr.`, `e.g.`, `U.S.`,
///   single-letter initials) does not end a sentence, unless the next word
///   is a common sentence opener such as "The", "We" or "However" (`the
///   U.S. The` splits, `U.S. Senator` does not). Both word lists and the
///   rule come from the sentencex segmenter (Wikimedia; MIT License);
///   abbreviations are matched lowercased, openers case-sensitively.
///
/// ```
/// let text = "Mr. Smith left in 2014.\" Later he returned.";
/// let got: Vec<&str> = textprep::segment::sentence_byte_ranges(text)
///     .into_iter()
///     .map(|r| &text[r])
///     .collect();
/// assert_eq!(got, ["Mr. Smith left in 2014.\"", "Later he returned."]);
/// ```
pub fn sentence_byte_ranges(text: &str) -> Vec<Range<usize>> {
    let mut out = Vec::new();
    let mut start = 0;
    for (i, chunk) in text.split_sentence_bound_indices() {
        let end = i + chunk.len();
        if end < text.len() && !keep_break(&text[start..end], &text[end..]) {
            continue;
        }
        let body = text[start..end].trim();
        if !body.is_empty() {
            let lead = text[start..end].len() - text[start..end].trim_start().len();
            out.push(start + lead..start + lead + body.len());
        }
        start = end;
    }
    out
}

/// Whether a UAX #29 sentence break between `before` and `after` is kept.
fn keep_break(before: &str, after: &str) -> bool {
    let body = before.trim_end();
    let gap = &before[body.len()..];
    let line_breaks = gap
        .replace("\r\n", "\n")
        .chars()
        .filter(|&c| is_line_break(c))
        .count();
    if line_breaks >= 2 {
        return true;
    }
    if line_breaks > 0 {
        // Would UAX #29 break here if every line break were a space?
        let flow = |c: char| if is_line_break(c) { ' ' } else { c };
        let mut tail: Vec<char> = body.chars().rev().take(32).map(flow).collect();
        tail.reverse();
        let tail: String = tail.into_iter().collect();
        let next: String = after.trim_start().chars().take(32).map(flow).collect();
        let flowed = format!("{tail} {next}");
        let cut = tail.len() + 1;
        if !flowed.split_sentence_bound_indices().any(|(i, _)| i == cut) {
            return false;
        }
    }
    !ends_with_abbreviation(body) || starts_sentence(after)
}

/// Reads one of the vendored word lists: a word per line, `#` comments.
fn word_list(list: &'static str) -> impl Iterator<Item = &'static str> {
    list.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
}

/// True when the first word of `after` is a sentence starter such as "The",
/// "We" or "However" (case-sensitive).
fn starts_sentence(after: &str) -> bool {
    thread_local! {
        static STARTERS: HashSet<&'static str> =
            word_list(include_str!("data/sentence_starters_en.txt")).collect();
    }
    let after = after.trim_start();
    let end = after
        .find(|c: char| c.is_whitespace() || matches!(c, ',' | '.' | '!' | '?'))
        .unwrap_or(after.len());
    end > 0 && STARTERS.with(|s| s.contains(&after[..end]))
}

/// UAX #29's paragraph separators (classes CR, LF and Sep).
fn is_line_break(c: char) -> bool {
    matches!(c, '\n' | '\r' | '\u{85}' | '\u{2028}' | '\u{2029}')
}

/// True when `body` ends with a period directly after a known abbreviation.
fn ends_with_abbreviation(body: &str) -> bool {
    thread_local! {
        static ABBREVIATIONS: HashSet<String> =
            word_list(include_str!("data/abbreviations_en.txt"))
                .map(str::to_lowercase)
                .collect();
    }
    let Some(head) = body.strip_suffix('.') else {
        return false;
    };
    // The last whitespace-separated word, without opening quotes or brackets.
    let word = head
        .rsplit(char::is_whitespace)
        .next()
        .unwrap_or("")
        .trim_start_matches(|c: char| !c.is_alphanumeric());
    !word.is_empty()
        && word.chars().all(|c| c.is_alphanumeric() || c == '.')
        && ABBREVIATIONS.with(|a| a.contains(&word.to_lowercase()))
}

/// Byte ranges of the tokens of `text`, skipping whitespace.
///
/// Tokens are the UAX #29 word boundaries: "don't", "3.5", "v3.5", "e.g"
/// and "U.S" are single words, and every punctuation character is its own
/// token. One change: words joined by a single `-` with no space around it
/// stay one token ("state-of-the-art"), where UAX #29 splits at each hyphen.
///
/// ```
/// let text = "A state-of-the-art model, v3.5.";
/// let got: Vec<&str> = textprep::segment::word_byte_ranges(text)
///     .into_iter()
///     .map(|r| &text[r])
///     .collect();
/// assert_eq!(got, ["A", "state-of-the-art", "model", ",", "v3.5", "."]);
/// ```
pub fn word_byte_ranges(text: &str) -> Vec<Range<usize>> {
    let is_word = |r: &Range<usize>| text[r.clone()].chars().any(char::is_alphanumeric);
    let mut out: Vec<Range<usize>> = Vec::new();
    for (i, segment) in text.split_word_bound_indices() {
        if segment.chars().all(char::is_whitespace) {
            continue;
        }
        let range = i..i + segment.len();
        if let [.., left, hyphen] = out.as_slice() {
            let joined = is_word(&range)
                && &text[hyphen.clone()] == "-"
                && hyphen.end == range.start
                && left.end == hyphen.start
                && is_word(left);
            if joined {
                let start = left.start;
                out.truncate(out.len() - 2);
                out.push(start..range.end);
                continue;
            }
        }
        out.push(range);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sentences(text: &str) -> Vec<&str> {
        sentence_byte_ranges(text)
            .into_iter()
            .map(|r| &text[r])
            .collect()
    }

    #[test]
    fn words_follow_uax29_but_keep_hyphenated_compounds() {
        let text = "U.S. e.g. don\u{2019}t x86_64 non-linear -- a - b C++";
        let got: Vec<&str> = word_byte_ranges(text)
            .into_iter()
            .map(|r| &text[r])
            .collect();
        assert_eq!(
            got,
            vec![
                "U.S",
                ".",
                "e.g",
                ".",
                "don\u{2019}t",
                "x86_64",
                "non-linear",
                "-",
                "-",
                "a",
                "-",
                "b",
                "C",
                "+",
                "+"
            ]
        );
    }

    #[test]
    fn sentence_needs_uppercase_after_period() {
        assert_eq!(
            sentences("see fig. two. Then more."),
            vec!["see fig. two.", "Then more."]
        );
    }

    #[test]
    fn sentence_ends_after_closing_quote() {
        assert_eq!(
            sentences("He left in 2014.\" Later he (aged 40.) returned! Why? Yes."),
            vec![
                "He left in 2014.\"",
                "Later he (aged 40.) returned!",
                "Why?",
                "Yes."
            ]
        );
    }

    #[test]
    fn abbreviations_do_not_end_sentences() {
        let text = "Mr. Smith met Dr. J. Doe in the U.S. on Jan. 5. He hasn't. U.S. Senator \
                    Lee told me. We left the U.S. The end.";
        assert_eq!(
            sentences(text),
            vec![
                "Mr. Smith met Dr. J. Doe in the U.S. on Jan. 5.",
                "He hasn't.",
                "U.S. Senator Lee told me.",
                "We left the U.S.",
                "The end."
            ]
        );
    }

    #[test]
    fn single_line_breaks_join_and_blank_lines_split() {
        let text = "A hard\nwrapped sentence.\nNext one\r\n\r\nHeading\n\nBody text.";
        assert_eq!(
            sentences(text),
            vec![
                "A hard\nwrapped sentence.",
                "Next one",
                "Heading",
                "Body text."
            ]
        );
    }

    #[test]
    fn ranges_index_the_source_text() {
        let text = "  \u{201C}Caf\u{e9} au lait.\u{201D} Next.  ";
        assert_eq!(
            sentences(text),
            vec!["\u{201C}Caf\u{e9} au lait.\u{201D}", "Next."]
        );
        assert!(sentence_byte_ranges("").is_empty());
        assert!(sentence_byte_ranges(" \n\n ").is_empty());
    }
}
