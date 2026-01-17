//! # textprep
//!
//! Text preprocessing primitives for the representational stack.
//!
//! Provides Unicode normalization, case folding, diacritics stripping,
//! tokenization, and fast keyword matching.

pub mod flash;
pub mod fold;
pub mod ngram;
pub mod stopwords;
pub mod tokenize;
pub mod unicode;

pub use flash::{FlashText, KeywordMatch};
pub use fold::{fold, strip_diacritics};
pub use tokenize::Token;
pub use unicode::{nfc, nfkc};

/// A convenience function to perform a default "scrub" of text.
///
/// Normalizes to NFC, folds case, and strips diacritics.
pub fn scrub(text: &str) -> String {
    let normalized = unicode::nfc(text);
    let folded = fold::fold(&normalized);
    fold::strip_diacritics(&folded)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scrub() {
        let text = "Müller";
        assert_eq!(scrub(text), "muller");
    }
}
