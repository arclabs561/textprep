//! String similarity primitives.
//!
//! Scope: small, dependency-light building blocks that are reused across crates.
//! Keep policy (thresholds, weights, feature selection) in higher-level crates.
//!
//! Notes:
//! - Similarities are computed on **Unicode scalar values** (`char`), not bytes.
//! - Tokenization for `word_jaccard` is whitespace-based for predictability.

use std::collections::HashSet;

/// Jaccard similarity for whitespace-delimited tokens.
///
/// Returns a value in \([0, 1]\), where \(1\) means identical token sets.
///
/// Case-insensitive: lowercases both inputs first.
pub fn word_jaccard(a: &str, b: &str) -> f64 {
    let a_lower = a.to_lowercase();
    let b_lower = b.to_lowercase();

    let words_a: HashSet<&str> = a_lower.split_whitespace().collect();
    let words_b: HashSet<&str> = b_lower.split_whitespace().collect();

    if words_a.is_empty() && words_b.is_empty() {
        return 1.0;
    }
    if words_a.is_empty() || words_b.is_empty() {
        return 0.0;
    }

    let intersection = words_a.intersection(&words_b).count();
    let union = words_a.union(&words_b).count();

    if union == 0 {
        0.0
    } else {
        intersection as f64 / union as f64
    }
}

/// Jaccard similarity for character \(n\)-grams.
///
/// Case-insensitive: lowercases both inputs first.
///
/// Behavior for short strings:
/// - If the lowercased strings are identical, returns 1.0 (even if \(< n\)).
/// - Otherwise, if either side yields no n-grams, returns 0.0.
pub fn char_ngram_jaccard(a: &str, b: &str, n: usize) -> f64 {
    if n == 0 {
        return 0.0;
    }

    let a_lower = a.to_lowercase();
    let b_lower = b.to_lowercase();

    if a_lower == b_lower {
        return 1.0;
    }

    let ngrams_a: HashSet<String> = crate::ngram::char_ngrams(&a_lower, n).into_iter().collect();
    let ngrams_b: HashSet<String> = crate::ngram::char_ngrams(&b_lower, n).into_iter().collect();

    if ngrams_a.is_empty() || ngrams_b.is_empty() {
        return 0.0;
    }

    let intersection = ngrams_a.intersection(&ngrams_b).count();
    let union = ngrams_a.union(&ngrams_b).count();

    if union == 0 {
        0.0
    } else {
        intersection as f64 / union as f64
    }
}

/// Convenience: trigram (\(n=3\)) Jaccard similarity on characters.
pub fn trigram_jaccard(a: &str, b: &str) -> f64 {
    char_ngram_jaccard(a, b, 3)
}

/// Weighted blend of word-level Jaccard and character n-gram Jaccard.
///
/// \[
/// s = w_{\text{word}}\cdot J_{\text{word}}(a,b) + w_{\text{char}}\cdot J_{\text{char-}n}(a,b)
/// \]
///
/// No automatic normalization of weights is performed.
pub fn weighted_word_char_ngram_jaccard(
    a: &str,
    b: &str,
    n: usize,
    word_weight: f64,
    char_weight: f64,
) -> f64 {
    let w = word_jaccard(a, b);
    let c = char_ngram_jaccard(a, b, n);
    word_weight * w + char_weight * c
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn word_jaccard_basic() {
        assert!((word_jaccard("hello world", "world hello") - 1.0).abs() < 1e-9);
        assert!(word_jaccard("hello", "world") <= 1.0);
        assert!(word_jaccard("hello", "world") >= 0.0);
        assert_eq!(word_jaccard("", ""), 1.0);
        assert_eq!(word_jaccard("", "x"), 0.0);
    }

    #[test]
    fn char_ngram_jaccard_bounds_and_identity() {
        let s = "François Müller";
        assert!((char_ngram_jaccard(s, s, 3) - 1.0).abs() < 1e-9);
        let v = char_ngram_jaccard("hello", "world", 3);
        assert!((0.0..=1.0).contains(&v));
    }

    #[test]
    fn trigram_jaccard_short_strings() {
        // Same (case-insensitive) short strings: similarity 1.0 even though no trigrams exist.
        assert_eq!(trigram_jaccard("a", "A"), 1.0);
        // Different short strings: 0.0
        assert_eq!(trigram_jaccard("a", "b"), 0.0);
    }
}
