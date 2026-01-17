//! # textprep
//!
//! Text preprocessing primitives for the representational stack.
//!
//! Provides Unicode normalization, case folding, diacritics stripping,
//! tokenization, and fast keyword matching.

pub mod flash;
pub mod fold;
pub mod ngram;
pub mod similarity;
pub mod stopwords;
pub mod subword;
pub mod tokenize;
pub mod unicode;

pub use flash::{FlashText, KeywordMatch};
pub use fold::{fold, strip_diacritics};
pub use subword::{BpeTokenizer, SubwordTokenizer};
pub use tokenize::Token;
pub use unicode::{nfc, nfkc};

/// Policy/config for constructing normalized keys / comparison forms.
///
/// The intent is to make the pipeline explicit: most real bugs here are from
/// *implicitly* normalizing and accidentally destroying semantics (ZWJ/ZWNJ, bidi marks,
/// punctuation, newlines).
#[derive(Debug, Clone)]
pub struct ScrubConfig {
    /// Normalize newlines (`\r\n`/`\r` → `\n`) before any other whitespace policy.
    pub normalize_newlines: bool,
    /// Remove common zero-width characters (ZWSP/ZWNJ/ZWJ/WORD JOINER/BOM).
    pub remove_zero_width: bool,
    /// Remove Unicode bidirectional control characters (Trojan Source-style).
    pub remove_bidi_controls: bool,
    /// Collapse all Unicode whitespace to single ASCII spaces (and trim).
    pub collapse_whitespace: bool,
    /// Which normalization form to apply before case/diacritics.
    pub normalization: ScrubNormalization,
    /// Case handling strategy.
    pub case: ScrubCase,
    /// Strip combining marks (diacritics) after normalization + case mapping.
    pub strip_diacritics: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrubNormalization {
    None,
    Nfc,
    Nfkc,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrubCase {
    None,
    Lower,
    /// Full Unicode case folding (NFKC_Casefold). Requires `casefold` feature.
    #[cfg(feature = "casefold")]
    NfkcCasefold,
}

impl Default for ScrubConfig {
    fn default() -> Self {
        Self {
            normalize_newlines: false,
            remove_zero_width: false,
            remove_bidi_controls: false,
            collapse_whitespace: false,
            normalization: ScrubNormalization::Nfc,
            case: ScrubCase::Lower,
            strip_diacritics: true,
        }
    }
}

/// A convenience function to perform a default "scrub" of text.
///
/// Normalizes to NFC, folds case, and strips diacritics.
pub fn scrub(text: &str) -> String {
    scrub_with(text, &ScrubConfig::default())
}

/// Scrub text using an explicit policy.
pub fn scrub_with(text: &str, cfg: &ScrubConfig) -> String {
    let mut s = text.to_string();

    if cfg.normalize_newlines {
        s = unicode::normalize_newlines(&s);
    }
    if cfg.remove_zero_width {
        s = unicode::remove_zero_width(&s);
    }
    if cfg.remove_bidi_controls {
        s = unicode::remove_bidi_controls(&s);
    }
    if cfg.collapse_whitespace {
        s = unicode::collapse_whitespace(&s);
    }

    s = match cfg.normalization {
        ScrubNormalization::None => s,
        ScrubNormalization::Nfc => unicode::nfc(&s),
        ScrubNormalization::Nfkc => unicode::nfkc(&s),
    };

    s = match cfg.case {
        ScrubCase::None => s,
        ScrubCase::Lower => fold::fold(&s),
        #[cfg(feature = "casefold")]
        ScrubCase::NfkcCasefold => fold::fold_nfkc_casefold(&s),
    };

    if cfg.strip_diacritics {
        s = fold::strip_diacritics(&s);
    }

    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scrub() {
        let text = "Müller";
        assert_eq!(scrub(text), "muller");
    }

    #[test]
    fn test_scrub_with_preserves_when_disabled() {
        let cfg = ScrubConfig {
            normalization: ScrubNormalization::None,
            case: ScrubCase::None,
            strip_diacritics: false,
            ..ScrubConfig::default()
        };
        assert_eq!(scrub_with("Müller", &cfg), "Müller");
    }
}
