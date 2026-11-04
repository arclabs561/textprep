//! Stopword lists.
//!
//! The lists are NLTK's stopword corpus (`nltk_data/corpora/stopwords`), as
//! packaged by the `stop-words` crate with only its `nltk` feature, for the
//! 33 languages in [`LANGUAGES`]. NLTK took most of the lists for languages
//! with a Snowball stemmer from Snowball's hand-annotated function-word
//! lists (via PostgreSQL); its English list adds contraction forms such as
//! "don't" and "should've". Other lists (Arabic, Chinese, Slovene, Hinglish
//! and more) have no stated provenance in NLTK's README.
//!
//! textprep does not ship stopwords-iso: it merges 26 lists into 1,298
//! English words, including content words such as "information", "system"
//! and "computer". If another crate in the build enables `stop-words`'s
//! `iso` feature, the NLTK list still wins for every code in [`LANGUAGES`].
//!
//! Tokenizers that follow UAX #29 keep `don’t` (with U+2019) as one token,
//! and it never matches the list's `don't`. [`fold_apostrophes`] fixes that;
//! [`crate::language::Language::is_stopword`] applies it.

use std::borrow::Cow;
use std::collections::HashSet;
use std::sync::{Arc, OnceLock};

/// The 127-word English list that textprep shipped before 0.3.
///
/// This is NLTK's English list before nltk_data issue 22 added contraction
/// forms. Its contents are unchanged, but textprep no longer uses it:
/// [`get`]`("en")`, [`is_english_stopword`] and
/// [`crate::language::Language::english`] use NLTK's current 198-word list,
/// which is this list plus 71 contraction forms and fragments ("ain",
/// "aren", "aren't", "couldn", ..., "ll", "m", "ma", "o", "re", "ve", "y",
/// "you've").
#[deprecated(
    since = "0.3.0",
    note = "textprep now uses NLTK's 198-word English list; use `stopwords::list(\"en\")` or `Language::english()`"
)]
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

/// The codes of the languages with a stopword list, sorted: ISO 639-1
/// codes plus NLTK's `"hinglish"`.
pub const LANGUAGES: &[&str] = &[
    "ar", "az", "be", "bn", "ca", "da", "de", "el", "en", "es", "eu", "fi", "fr", "he", "hinglish",
    "hu", "id", "it", "kk", "ne", "nl", "no", "pt", "ro", "ru", "sl", "sq", "sv", "ta", "tg", "tr",
    "uz", "zh",
];

/// ISO 639-3 code and English name of each code in [`LANGUAGES`], in the
/// same order. `"hinglish"` has no ISO code.
const ALIASES: [(&str, &str); 33] = [
    ("ara", "arabic"),
    ("aze", "azerbaijani"),
    ("bel", "belarusian"),
    ("ben", "bengali"),
    ("cat", "catalan"),
    ("dan", "danish"),
    ("deu", "german"),
    ("ell", "greek"),
    ("eng", "english"),
    ("spa", "spanish"),
    ("eus", "basque"),
    ("fin", "finnish"),
    ("fra", "french"),
    ("heb", "hebrew"),
    ("", "hinglish"),
    ("hun", "hungarian"),
    ("ind", "indonesian"),
    ("ita", "italian"),
    ("kaz", "kazakh"),
    ("nep", "nepali"),
    ("nld", "dutch"),
    ("nor", "norwegian"),
    ("por", "portuguese"),
    ("ron", "romanian"),
    ("rus", "russian"),
    ("slv", "slovene"),
    ("sqi", "albanian"),
    ("swe", "swedish"),
    ("tam", "tamil"),
    ("tgk", "tajik"),
    ("tur", "turkish"),
    ("uzb", "uzbek"),
    ("zho", "chinese"),
];

/// The code in [`LANGUAGES`] for a language name or code, if it has a list.
///
/// Accepts the ISO 639-1 code (`"de"`), the ISO 639-3 code (`"deu"`) or the
/// English name (`"german"`), in any case, plus `"nb"` and `"nob"`
/// (Bokmål) for Norwegian, `"cmn"` (Mandarin) for Chinese and
/// `"slovenian"` for Slovene. A region or script subtag after `-` or `_` is
/// ignored, so `"en-US"` and `"pt_BR"` resolve too.
///
/// ```
/// use textprep::stopwords::canonical_code;
/// assert_eq!(canonical_code("German"), Some("de"));
/// assert_eq!(canonical_code("en-US"), Some("en"));
/// assert_eq!(canonical_code("pl"), None);
/// ```
pub fn canonical_code(lang: &str) -> Option<&'static str> {
    let lang = lang.trim().to_lowercase();
    let primary = lang.split(['-', '_']).next().unwrap_or("");
    let primary = match primary {
        "nb" | "nob" => "no",
        "cmn" => "zh",
        "slovenian" => "sl",
        other => other,
    };
    LANGUAGES
        .iter()
        .zip(ALIASES)
        .find(|(code, (iso3, name))| {
            **code == primary || (!iso3.is_empty() && *iso3 == primary) || *name == primary
        })
        .map(|(code, _)| *code)
}

/// The stopword list for a language, as packaged: NLTK's order, which can
/// repeat a word. `lang` is resolved with [`canonical_code`]; `None` when
/// there is no list for it.
///
/// ```
/// let de = textprep::stopwords::list("de").unwrap();
/// assert!(de.contains(&"und"));
/// assert!(textprep::stopwords::list("xx").is_none());
/// ```
pub fn list(lang: &str) -> Option<&'static [&'static str]> {
    canonical_code(lang).and_then(stop_words::lookup)
}

/// The stopword set for a code in [`LANGUAGES`], built once and shared.
pub(crate) fn shared(code: &'static str) -> Arc<HashSet<String>> {
    static SETS: [OnceLock<Arc<HashSet<String>>>; 33] = [const { OnceLock::new() }; 33];
    let i = LANGUAGES
        .iter()
        .position(|c| *c == code)
        .expect("code is in LANGUAGES");
    SETS[i]
        .get_or_init(|| {
            let words = stop_words::lookup(code).unwrap_or_default();
            Arc::new(words.iter().map(|w| w.to_string()).collect())
        })
        .clone()
}

/// Return the stopword set for a language.
///
/// Every language in [`LANGUAGES`] has a set. `lang` is resolved with
/// [`canonical_code`], so `"en"`, `"English"` and `"eng"` give the same set.
/// Each set is built once and cloned on every call. Unknown languages
/// return an empty set.
///
/// Before 0.3 only English had a set, the 127-word list kept as the
/// deprecated `ENGLISH`; English now has NLTK's 198 words.
pub fn get(lang: &str) -> HashSet<String> {
    match canonical_code(lang) {
        Some(code) => shared(code).as_ref().clone(),
        None => HashSet::new(),
    }
}

/// Replace U+2018 and U+2019 (curly single quotes) with `'`.
///
/// The lists spell contractions with `'`, while text often has U+2019.
/// Borrows when there is nothing to replace.
///
/// ```
/// use textprep::stopwords::fold_apostrophes;
/// assert_eq!(fold_apostrophes("don\u{2019}t"), "don't");
/// ```
pub fn fold_apostrophes(token: &str) -> Cow<'_, str> {
    const CURLY: [char; 2] = ['\u{2018}', '\u{2019}'];
    if token.contains(CURLY) {
        Cow::Owned(token.replace(CURLY, "'"))
    } else {
        Cow::Borrowed(token)
    }
}

/// Membership check for NLTK's English stopwords (198 words).
///
/// Matching is case-sensitive, as before, but U+2018 and U+2019 count as
/// `'`, so `don’t` is a stopword. Before 0.3 this checked the 127-word list
/// kept as the deprecated `ENGLISH`.
pub fn is_english_stopword(token: &str) -> bool {
    shared("en").contains(fold_apostrophes(token).as_ref())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(deprecated)]
    fn old_english_list_is_sorted_and_inside_nltk_english() {
        assert_eq!(ENGLISH.len(), 127);
        assert!(ENGLISH.windows(2).all(|w| w[0] < w[1]));
        let en = get("en");
        assert!(ENGLISH.iter().all(|w| en.contains(*w)));
        assert_eq!(en.len() - ENGLISH.len(), 71);
    }

    #[test]
    fn is_english_stopword_smoke() {
        assert!(is_english_stopword("the"));
        assert!(is_english_stopword("don't"));
        assert!(is_english_stopword("don\u{2019}t"));
        assert!(is_english_stopword("should\u{2018}ve"));
        assert!(!is_english_stopword("The"));
        assert!(!is_english_stopword("tokio"));
    }

    #[test]
    fn codes_resolve_and_aliases_agree() {
        assert!(LANGUAGES.windows(2).all(|w| w[0] < w[1]));
        for (code, (iso3, name)) in LANGUAGES.iter().zip(ALIASES) {
            assert_eq!(canonical_code(code), Some(*code));
            assert_eq!(canonical_code(name), Some(*code), "{name}");
            if !iso3.is_empty() {
                assert_eq!(canonical_code(iso3), Some(*code), "{iso3}");
            }
        }
        assert_eq!(canonical_code(" PT_br "), Some("pt"));
        assert_eq!(canonical_code("nb-NO"), Some("no"));
        assert_eq!(canonical_code(""), None);
        assert_eq!(canonical_code("xx"), None);
        assert!(get("xx").is_empty());
        assert_eq!(get("English"), get("eng"));
    }

    #[test]
    fn fold_apostrophes_borrows_when_unchanged() {
        assert!(matches!(fold_apostrophes("plain"), Cow::Borrowed(_)));
        assert_eq!(fold_apostrophes("\u{2018}tis o\u{2019}"), "'tis o'");
    }
}
