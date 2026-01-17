//! Text tokenization utilities.

use unicode_segmentation::UnicodeSegmentation;

pub fn words(text: &str) -> Vec<&str> {
    text.unicode_words().collect()
}

pub fn sentences(text: &str) -> Vec<&str> {
    text.unicode_sentences().collect()
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Token {
    pub text: String,
    pub start: usize,
    pub end: usize,
}

pub fn tokenize_with_offsets(text: &str) -> Vec<Token> {
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
        tokens.push(Token {
            text: word.to_string(),
            start,
            end: start + len,
        });

        last_byte = byte_idx + word.len();
        last_char = start + len;
    }
    tokens
}
