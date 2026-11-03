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
/// Stripped after NFD: the Combining Diacritical Marks blocks (U+0300,
/// U+1AB0, U+1DC0, U+20D0, U+FE20) and the combining marks of the Hebrew
/// (niqqud, cantillation) and Arabic (harakat, Quranic annotation) blocks.
/// Combining marks of other scripts are kept, because there they often form
/// distinct letters or vowels: kana voicing marks (`が` stays `が`),
/// Indic viramas and vowel signs, Thai vowels.
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
    match c {
        '\u{0300}'..='\u{036F}'
        | '\u{1AB0}'..='\u{1AFF}'
        | '\u{1DC0}'..='\u{1DFF}'
        | '\u{20D0}'..='\u{20FF}'
        | '\u{FE20}'..='\u{FE2F}' => true,
        // Hebrew; Arabic; Arabic Extended-B and Extended-A. These blocks
        // also hold letters and format characters, so only marks match.
        '\u{0590}'..='\u{05FF}' | '\u{0600}'..='\u{06FF}' | '\u{0870}'..='\u{08FF}' => {
            unicode_normalization::char::is_combining_mark(c)
        }
        _ => false,
    }
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
    use caseless::Caseless;
    text.nfkc().default_case_fold().collect()
}

/// Like `fold_nfkc_casefold`, but writes into an existing `String`.
#[cfg(feature = "casefold")]
pub fn fold_nfkc_casefold_into(text: &str, out: &mut String) {
    use caseless::Caseless;
    out.clear();
    out.reserve(text.len());
    out.extend(text.nfkc().default_case_fold());
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

    #[test]
    fn strip_diacritics_removes_hebrew_and_arabic_points() {
        // shalom with niqqud -> consonants
        assert_eq!(
            strip_diacritics("\u{05E9}\u{05C1}\u{05B8}\u{05DC}\u{05D5}\u{05B9}\u{05DD}"),
            "\u{05E9}\u{05DC}\u{05D5}\u{05DD}"
        );
        // Precomposed U+FB2A (shin with shin dot) decomposes, then loses the dot.
        assert_eq!(strip_diacritics("\u{FB2A}"), "\u{05E9}");
        // muhammad with harakat -> rasm
        assert_eq!(
            strip_diacritics("\u{0645}\u{064F}\u{062D}\u{064E}\u{0645}\u{0651}\u{064E}\u{062F}"),
            "\u{0645}\u{062D}\u{0645}\u{062F}"
        );
        // a + U+1AB0 COMBINING DOUBLED CIRCUMFLEX ACCENT
        assert_eq!(strip_diacritics("a\u{1AB0}"), "a");
    }

    #[test]
    fn strip_diacritics_keeps_marks_that_change_letters_in_other_scripts() {
        // Kana voicing marks: NFD splits が into か + U+3099; it must recompose.
        assert_eq!(strip_diacritics("がぱ"), "がぱ");
        // Devanagari virama and vowel signs, Thai vowels.
        assert_eq!(strip_diacritics("क्षमा"), "क्षमा");
        assert_eq!(strip_diacritics("กุ"), "กุ");
        // Arabic letters and the end-of-ayah sign (a format character) stay.
        assert_eq!(strip_diacritics("\u{0628}\u{06DD}"), "\u{0628}\u{06DD}");
    }

    #[cfg(feature = "casefold")]
    #[test]
    fn casefold_covers_georgian_mtavruli() {
        // U+1C90 GEORGIAN MTAVRULI CAPITAL LETTER AN (Unicode 11) folds to
        // U+10D0 GEORGIAN LETTER AN.
        assert_eq!(fold_nfkc_casefold("\u{1C90}"), "\u{10D0}");
        let mut out = String::new();
        fold_nfkc_casefold_into("\u{1C90}\u{1C91}", &mut out);
        assert_eq!(out, "\u{10D0}\u{10D1}");
    }
}
