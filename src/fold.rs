//! Case folding and diacritics stripping.

use unicode_normalization::UnicodeNormalization;

/// Strip combining marks (diacritics) from text.
///
/// ```
/// use textprep::strip_diacritics;
///
/// assert_eq!(strip_diacritics("Müller"), "Muller");
/// assert_eq!(strip_diacritics("naïve"), "naive");
/// assert_eq!(strip_diacritics("ASCII"), "ASCII"); // no-op
/// ```
///
/// The result is recomposed to NFC, so base characters that carry no
/// stripped mark stay in their composed form (e.g. Hangul syllables).
pub fn strip_diacritics(text: &str) -> String {
    text.nfd()
        .filter(|c| !is_combining_mark(*c))
        .nfc()
        .collect()
}

fn is_combining_mark(c: char) -> bool {
    matches!(c, '\u{0300}'..='\u{036F}' | '\u{1DC0}'..='\u{1DFF}' | '\u{20D0}'..='\u{20FF}' | '\u{FE20}'..='\u{FE2F}')
}

/// Lowercase using Rust's built-in Unicode-aware `to_lowercase`.
///
/// This is **not** full Unicode case folding. For search/index keys where
/// full case folding matters, prefer `fold_nfkc_casefold` (feature-gated).
///
/// ```
/// use textprep::fold::fold;
///
/// assert_eq!(fold("Hello WORLD"), "hello world");
/// assert_eq!(fold("Ω"), "ω");
/// ```
pub fn fold(text: &str) -> String {
    text.to_lowercase()
}

/// Normalize to NFKC and then apply full Unicode case folding.
///
/// This is useful for building robust lookup keys for identifiers/names:
/// it removes compatibility distinctions (NFKC) and applies language-agnostic
/// case folding.
///
/// This is not exactly the Unicode `NFKC_Casefold` mapping (UAX #44): that
/// also applies NFKC again after folding and removes Default_Ignorable code
/// points (soft hyphen, ZWJ/ZWNJ, variation selectors). Here those code
/// points are kept, so [`crate::ScrubConfig::search_key`] can leave ZWJ/ZWNJ
/// in place by policy.
///
/// This is feature-gated to avoid pulling extra dependencies into minimal builds.
#[cfg(feature = "casefold")]
pub fn fold_nfkc_casefold(text: &str) -> String {
    use unicode_casefold::UnicodeCaseFold;
    text.nfkc().case_fold().collect()
}

/// Like `fold_nfkc_casefold`, but writes into an existing `String`.
#[cfg(feature = "casefold")]
pub fn fold_nfkc_casefold_into(text: &str, out: &mut String) {
    use unicode_casefold::UnicodeCaseFold;
    out.clear();
    out.reserve(text.len());
    out.extend(text.nfkc().case_fold());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_diacritics() {
        assert_eq!(strip_diacritics("Müller"), "Muller");
    }

    #[test]
    fn strip_diacritics_output_is_nfc() {
        // Hangul syllables decompose to jamo under NFD; none are marks, so
        // they must come back composed.
        let out = strip_diacritics("한글 café");
        assert_eq!(out, "한글 cafe");
        assert!(unicode_normalization::is_nfc(&out));
    }
}
