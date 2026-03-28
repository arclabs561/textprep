//! Stopword lists.

use std::collections::HashSet;

/// Sorted list of common English stopwords.
pub const ENGLISH: &[&str] = &[
    "a",
    "about",
    "above",
    "after",
    "again",
    "against",
    "all",
    "am",
    "an",
    "and",
    "any",
    "are",
    "as",
    "at",
    "be",
    "because",
    "been",
    "before",
    "being",
    "below",
    "between",
    "both",
    "but",
    "by",
    "can",
    "did",
    "do",
    "does",
    "doing",
    "don",
    "down",
    "during",
    "each",
    "few",
    "for",
    "from",
    "further",
    "had",
    "has",
    "have",
    "having",
    "he",
    "her",
    "here",
    "hers",
    "herself",
    "him",
    "himself",
    "his",
    "how",
    "i",
    "if",
    "in",
    "into",
    "is",
    "it",
    "its",
    "itself",
    "just",
    "me",
    "more",
    "most",
    "my",
    "myself",
    "no",
    "nor",
    "not",
    "now",
    "of",
    "off",
    "on",
    "once",
    "only",
    "or",
    "other",
    "our",
    "ours",
    "ourselves",
    "out",
    "over",
    "own",
    "s",
    "same",
    "she",
    "should",
    "so",
    "some",
    "such",
    "t",
    "than",
    "that",
    "the",
    "their",
    "theirs",
    "them",
    "themselves",
    "then",
    "there",
    "these",
    "they",
    "this",
    "those",
    "through",
    "to",
    "too",
    "under",
    "until",
    "up",
    "very",
    "was",
    "we",
    "were",
    "what",
    "when",
    "where",
    "which",
    "while",
    "who",
    "whom",
    "why",
    "will",
    "with",
    "you",
    "your",
    "yours",
    "yourself",
    "yourselves",
];

/// Fast membership check for English stopwords.
///
/// The `ENGLISH` list is kept sorted so we can use `binary_search` without
/// allocating a `HashSet`.
pub fn is_english_stopword(token: &str) -> bool {
    ENGLISH.binary_search(&token).is_ok()
}

/// Return the stopword set for a language.
///
/// The English set is cached after the first call to avoid repeated allocation.
/// Unknown languages return an empty set (always a new allocation).
pub fn get(lang: &str) -> HashSet<String> {
    static ENGLISH_SET: std::sync::OnceLock<HashSet<String>> = std::sync::OnceLock::new();

    match lang.to_lowercase().as_str() {
        "en" | "english" => ENGLISH_SET
            .get_or_init(|| ENGLISH.iter().map(|s| s.to_string()).collect())
            .clone(),
        _ => HashSet::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn english_stopwords_are_sorted_for_binary_search() {
        assert!(
            ENGLISH.windows(2).all(|w| w[0] <= w[1]),
            "ENGLISH stopwords must remain sorted for binary_search"
        );
    }

    #[test]
    fn is_english_stopword_smoke() {
        assert!(is_english_stopword("the"));
        assert!(!is_english_stopword("tokio"));
    }
}
