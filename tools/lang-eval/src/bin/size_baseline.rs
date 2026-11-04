//! Binary-size baseline: textprep stopwords only, no detector.
fn main() {
    let text = std::env::args().nth(1).unwrap_or_default();
    let en = textprep::language::Language::english();
    println!(
        "{}",
        text.split_whitespace()
            .filter(|w| en.is_stopword(w))
            .count()
    );
}
