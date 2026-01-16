//! N-gram generation.

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
