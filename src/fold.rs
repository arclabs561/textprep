//! Case folding and diacritics stripping.

use unicode_normalization::UnicodeNormalization;

pub fn strip_diacritics(text: &str) -> String {
    text.nfd()
        .filter(|c| !is_combining_mark(*c))
        .collect()
}

fn is_combining_mark(c: char) -> bool {
    matches!(c, '\u{0300}'..='\u{036F}' | '\u{1DC0}'..='\u{1DFF}' | '\u{20D0}'..='\u{20FF}' | '\u{FE20}'..='\u{FE2F}')
}

pub fn fold(text: &str) -> String {
    text.to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_diacritics() {
        assert_eq!(strip_diacritics("Müller"), "Muller");
    }
}
