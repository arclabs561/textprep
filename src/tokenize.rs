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
    for (byte_idx, word) in text.unicode_word_indices() {
        let start = text[..byte_idx].chars().count();
        let len = word.chars().count();
        tokens.push(Token {
            text: word.to_string(),
            start,
            end: start + len,
        });
    }
    tokens
}
