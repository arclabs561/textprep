//! Text tokenization utilities.

use unicode_segmentation::UnicodeSegmentation;

pub fn words(text: &str) -> Vec<&str> {
    text.unicode_words().collect()
}

pub fn sentences(text: &str) -> Vec<&str> {
    text.unicode_sentences().collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TokenRef<'a> {
    pub text: &'a str,
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Token {
    pub text: String,
    pub start: usize,
    pub end: usize,
}

pub fn tokenize_refs_with_offsets(text: &str) -> Vec<TokenRef<'_>> {
    let mut tokens = Vec::new();
    // `unicode_word_indices()` yields (byte_offset, &str). Converting byte→char offsets
    // via `text[..byte_idx].chars().count()` is correct but can be \(O(n^2)\) overall.
    // Instead, advance incrementally from the last word boundary.
    let mut last_byte = 0usize;
    let mut last_char = 0usize;

    for (byte_idx, word) in text.unicode_word_indices() {
        // Advance char counter from last_byte → byte_idx.
        if byte_idx >= last_byte {
            last_char += text[last_byte..byte_idx].chars().count();
        } else {
            // Defensive: should not happen (monotonic iterator), but keep correctness.
            last_char = text[..byte_idx].chars().count();
        }

        let start = last_char;
        let len = word.chars().count();
        // Defensive: `unicode_word_indices()` should not return whitespace-containing tokens,
        // but some inputs (control chars / edge cases) can produce surprising results.
        // We only keep tokens that contain at least one non-whitespace char and no whitespace.
        if !word.is_empty() && word.chars().all(|c| !c.is_whitespace()) {
            tokens.push(TokenRef {
                text: word,
                start,
                end: start + len,
            });
        }

        last_byte = byte_idx + word.len();
        last_char = start + len;
    }
    tokens
}

pub fn tokenize_with_offsets(text: &str) -> Vec<Token> {
    tokenize_refs_with_offsets(text)
        .into_iter()
        .map(|t| Token {
            text: t.text.to_string(),
            start: t.start,
            end: t.end,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenize_refs_with_offsets_matches_owned_tokenizer() {
        let s = "a 東京 b Müller c";
        let refs = tokenize_refs_with_offsets(s);
        let owned = tokenize_with_offsets(s);
        assert_eq!(refs.len(), owned.len());
        for (r, o) in refs.iter().zip(owned.iter()) {
            assert_eq!(r.text, o.text);
            assert_eq!(r.start, o.start);
            assert_eq!(r.end, o.end);
        }
    }
}
