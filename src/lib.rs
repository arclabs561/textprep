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
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ScrubNormalization {
    None,
    Nfc,
    Nfkc,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ScrubCase {
    None,
    Lower,
    /// Full Unicode case folding (NFKC_Casefold). Requires `casefold` feature.
    #[cfg(feature = "casefold")]
    NfkcCasefold,
}

impl ScrubConfig {
    /// Policy for building a *search key* from user text.
    ///
    /// This is intentionally lossy and should generally be used for *matching/indexing*,
    /// not for permanent storage.
    ///
    /// Defaults:
    /// - normalize newlines
    /// - collapse whitespace
    /// - remove bidi controls (Trojan Source-style)
    /// - NFKC normalization (compatibility folding)
    /// - case folding (full NFKC_Casefold when available)
    /// - strip diacritics
    #[cfg(feature = "casefold")]
    pub fn search_key() -> Self {
        Self {
            normalize_newlines: true,
            remove_zero_width: false, // policy-sensitive (ZWJ/ZWNJ can be meaningful)
            remove_bidi_controls: true,
            collapse_whitespace: true,
            normalization: ScrubNormalization::Nfkc,
            case: ScrubCase::NfkcCasefold,
            strip_diacritics: true,
        }
    }

    /// See `search_key()` (fallback when `casefold` is disabled).
    #[cfg(not(feature = "casefold"))]
    pub fn search_key() -> Self {
        Self {
            normalize_newlines: true,
            remove_zero_width: false, // policy-sensitive (ZWJ/ZWNJ can be meaningful)
            remove_bidi_controls: true,
            collapse_whitespace: true,
            normalization: ScrubNormalization::Nfkc,
            case: ScrubCase::Lower,
            strip_diacritics: true,
        }
    }

    /// Like `search_key()`, but also removes common zero-width characters (ZWSP/ZWNJ/ZWJ/WJ/BOM).
    ///
    /// This is a deliberate trade-off:
    /// - **Pro**: avoids "ghost mismatches" in mostly-Latin corpora where ZWJ/ZWNJ are usually
    ///   artifacts (copy/paste, rich text) rather than orthographic intent.
    /// - **Con**: ZWJ/ZWNJ are semantically meaningful in multiple scripts (and for emoji ZWJ
    ///   sequences). Stripping can create false positives/negatives depending on the task.
    pub fn search_key_strict_invisibles() -> Self {
        let mut cfg = Self::search_key();
        cfg.remove_zero_width = true;
        cfg
    }
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
    // `scrub_with` is a small pipeline. We try to be explicit about what steps are
    // allocation-amortized (via `*_into`) versus which steps necessarily allocate
    // with our current APIs (normalization, fold/case, diacritics stripping).
    let mut s = text.to_string();
    let mut buf = String::new();

    if cfg.normalize_newlines {
        unicode::normalize_newlines_into(&s, &mut buf);
        std::mem::swap(&mut s, &mut buf);
    }
    if cfg.remove_zero_width {
        unicode::remove_zero_width_into(&s, &mut buf);
        std::mem::swap(&mut s, &mut buf);
    }
    if cfg.remove_bidi_controls {
        unicode::remove_bidi_controls_into(&s, &mut buf);
        std::mem::swap(&mut s, &mut buf);
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

    // Important ordering: compatibility normalization + diacritics stripping can introduce
    // ASCII spaces (e.g. U+00A8 DIAERESIS → " \u{0308}" under NFKD-like decomposition,
    // then stripping removes the combining mark). If we collapse whitespace *before* those
    // steps, we can end up returning keys with leading/trailing spaces.
    if cfg.collapse_whitespace {
        unicode::collapse_whitespace_into(&s, &mut buf);
        std::mem::swap(&mut s, &mut buf);
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

    #[test]
    #[cfg(feature = "casefold")]
    fn test_search_key_nfkc_casefold_expands_sharp_s() {
        let cfg = ScrubConfig::search_key();
        // ß case-folds to "ss" under Unicode CaseFolding.
        assert_eq!(scrub_with("Straße", &cfg), scrub_with("strasse", &cfg));
    }

    #[test]
    fn test_search_key_strict_invisibles_removes_zwj_family() {
        let cfg = ScrubConfig::search_key_strict_invisibles();
        let text = "a\u{200B}b\u{200C}c\u{200D}d\u{2060}e\u{FEFF}f";
        assert_eq!(scrub_with(text, &cfg), "abcdef");
    }

    #[test]
    fn test_search_key_preserves_zwj_by_default() {
        let cfg = ScrubConfig::search_key();
        let text = "x\u{200D}y"; // ZWJ
        let out = scrub_with(text, &cfg);
        assert_eq!(out, text);
    }

    #[test]
    #[cfg(feature = "serde")]
    fn test_serde_roundtrip_scrub_config() {
        let cfg = ScrubConfig {
            normalize_newlines: true,
            remove_zero_width: false,
            remove_bidi_controls: true,
            collapse_whitespace: true,
            normalization: ScrubNormalization::Nfkc,
            case: ScrubCase::Lower,
            strip_diacritics: true,
        };
        let s = serde_json::to_string(&cfg).expect("serialize");
        let de: ScrubConfig = serde_json::from_str(&s).expect("deserialize");
        assert_eq!(cfg.normalize_newlines, de.normalize_newlines);
        assert_eq!(cfg.remove_zero_width, de.remove_zero_width);
        assert_eq!(cfg.remove_bidi_controls, de.remove_bidi_controls);
        assert_eq!(cfg.collapse_whitespace, de.collapse_whitespace);
        assert_eq!(cfg.normalization, de.normalization);
        assert_eq!(cfg.case, de.case);
        assert_eq!(cfg.strip_diacritics, de.strip_diacritics);
    }

    #[test]
    #[cfg(feature = "serde")]
    fn test_serde_roundtrip_token() {
        let t = Token {
            text: "東京".to_string(),
            start: 1,
            end: 3,
        };
        let s = serde_json::to_string(&t).expect("serialize");
        let de: Token = serde_json::from_str(&s).expect("deserialize");
        assert_eq!(t, de);
    }

    #[test]
    #[cfg(feature = "serde")]
    fn test_serde_roundtrip_keyword_match() {
        let m = KeywordMatch {
            keyword: "Müller".to_string(),
            value: "muller".to_string(),
            start: 0,
            end: 6,
        };
        let s = serde_json::to_string(&m).expect("serialize");
        let de: KeywordMatch = serde_json::from_str(&s).expect("deserialize");
        assert_eq!(m, de);
    }
}
