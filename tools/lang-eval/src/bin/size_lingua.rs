//! Binary size with lingua, limited to its 29 languages with NLTK lists.
use lingua::Language::*;
fn main() {
    let text = std::env::args().nth(1).unwrap_or_default();
    let detector = lingua::LanguageDetectorBuilder::from_languages(&[
        Albanian,
        Arabic,
        Azerbaijani,
        Basque,
        Belarusian,
        Bengali,
        Bokmal,
        Catalan,
        Chinese,
        Danish,
        Dutch,
        English,
        Finnish,
        French,
        German,
        Greek,
        Hebrew,
        Hungarian,
        Indonesian,
        Italian,
        Kazakh,
        Portuguese,
        Romanian,
        Russian,
        Slovene,
        Spanish,
        Swedish,
        Tamil,
        Turkish,
    ])
    .build();
    println!("{:?}", detector.detect_language_of(text));
}
