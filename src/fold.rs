//! Case folding and diacritics stripping.

use unicode_normalization::UnicodeNormalization;

pub fn strip_diacritics(text: &str) -> String {
    text.nfd().filter(|c| !is_combining_mark(*c)).collect()
}

fn is_combining_mark(c: char) -> bool {
    matches!(c, '\u{0300}'..='\u{036F}' | '\u{1DC0}'..='\u{1DFF}' | '\u{20D0}'..='\u{20FF}' | '\u{FE20}'..='\u{FE2F}')
}

/// Lowercase using Rust's built-in Unicode-aware `to_lowercase`.
///
/// This is **not** full Unicode case folding. For search/index keys where
/// full case folding matters, prefer `fold_nfkc_casefold` (feature-gated).
pub fn fold(text: &str) -> String {
    text.to_lowercase()
}

/// Normalize to NFKC and then apply full Unicode case folding (NFKC_Casefold).
///
/// This is useful for building robust lookup keys for identifiers/names:
/// it removes compatibility distinctions (NFKC) and applies language-agnostic
/// case folding.
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
}
