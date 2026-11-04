//! Per-sentence language, stopwords and stems for a two-language text.
//!
//! cargo run --release --example sentence_languages --features detect,stem

use textprep::language::{sentence_languages, Language, LanguagePolicy};
use textprep::segment::word_byte_ranges;

fn main() {
    let text = "Die Abgeordneten diskutierten mehrere Stunden über die Kosten. \
                The committee didn\u{2019}t publish its final report. Ja.";
    let policy = LanguagePolicy::DetectPerSentence {
        fallback: Language::english(),
    };
    for s in sentence_languages(text, &policy) {
        let sentence = &text[s.range.clone()];
        let stems: Vec<String> = word_byte_ranges(sentence)
            .into_iter()
            .map(|r| &sentence[r])
            .filter(|w| w.chars().any(char::is_alphabetic) && !s.language.is_stopword(w))
            .map(|w| s.language.stem(w))
            .collect();
        println!(
            "{} ({:?}): {sentence}\n    {}",
            s.language.code(),
            s.source,
            stems.join(" ")
        );
    }
}
