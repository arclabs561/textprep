//! N-gram generation.

/// Generate character n-grams from text.
///
/// ```
/// use textprep::ngram::char_ngrams;
///
/// let trigrams = char_ngrams("hello", 3);
/// assert_eq!(trigrams, vec!["hel", "ell", "llo"]);
/// assert!(char_ngrams("hi", 3).is_empty()); // too short
/// ```
pub fn char_ngrams(text: &str, n: usize) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    if chars.len() < n {
        return Vec::new();
    }
    let mut result = Vec::with_capacity(chars.len() - n + 1);
    for window in chars.windows(n) {
        result.push(window.iter().collect());
    }
    result
}

/// Generate word n-grams by joining adjacent words with spaces.
///
/// ```
/// use textprep::ngram::word_ngrams;
///
/// let bigrams = word_ngrams(&["the", "quick", "fox"], 2);
/// assert_eq!(bigrams, vec!["the quick", "quick fox"]);
/// ```
pub fn word_ngrams(words: &[&str], n: usize) -> Vec<String> {
    if words.len() < n {
        return Vec::new();
    }
    let mut result = Vec::with_capacity(words.len() - n + 1);
    for window in words.windows(n) {
        result.push(window.join(" "));
    }
    result
}

/// Generate token n-grams as vectors of `&str`.
///
/// This is a convenience for callers that want to apply their own hashing/keying
/// scheme instead of joining tokens into a single string.
pub fn token_ngrams<'a>(words: &'a [&'a str], n: usize) -> Vec<Vec<&'a str>> {
    if n == 0 || words.len() < n {
        return Vec::new();
    }
    let mut result = Vec::with_capacity(words.len() - n + 1);
    for window in words.windows(n) {
        result.push(window.to_vec());
    }
    result
}
