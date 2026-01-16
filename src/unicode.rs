//! Unicode normalization utilities.

use unicode_normalization::UnicodeNormalization;

pub fn nfc(text: &str) -> String {
    text.nfc().collect()
}

pub fn nfkc(text: &str) -> String {
    text.nfkc().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nfc() {
        let decomposed = "a\u{0308}";
        let normalized = nfc(decomposed);
        assert_eq!(normalized, "ä");
    }
}
