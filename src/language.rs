//! A language as a bundle of text resources, and a policy for choosing it.
//!
//! [`Language`] holds a language code, its stopword set (NLTK's list from
//! [`crate::stopwords`], optionally replaced or extended) and, with the
//! `stem` feature, its Snowball stemmer. [`LanguagePolicy`] says which
//! language applies to which part of a text: one fixed language, one
//! detected per document, or one detected per sentence.
//! [`sentence_languages_with`] applies a policy with any [`Detector`];
//! with the `detect` feature, `sentence_languages` uses the built-in
//! whatlang detector.
//!
//! ```
//! use textprep::language::Language;
//!
//! let en = Language::english().with_extra_stopwords(["et", "al"]);
//! assert!(en.is_stopword("The"));
//! assert!(en.is_stopword("don\u{2019}t"));
//! assert!(en.is_stopword("al"));
//! assert!(!en.is_stopword("model"));
//! ```
//!
//! # Detection
//!
//! A [`Detector`] maps text to a [`Language`]. The `detect` feature adds
//! `WhatlangDetector` (whatlang 0.18, MIT, limited to the 28 languages it
//! shares with the stopword lists). Any other detector plugs in by
//! implementing [`Detector`], or as a closure `Fn(&str) -> Option<Language>`.
//! Policies never ask a detector about text with fewer than
//! [`MIN_DETECT_CHARS`] (10) letters.
//!
//! # Measurement
//!
//! `tools/lang-eval` in the repository detects 18,953 Tatoeba sentences
//! in 17 languages, alone and inside documents that interleave sentences
//! from two languages. Top-1 accuracy on single sentences, by length in
//! words (detectors limited to the languages with a stopword list):
//!
//! | detector | 1-3 | 4-5 | 6-8 | 9-12 | 13-20 | 21+ | sentences/s |
//! |---|---|---|---|---|---|---|---|
//! | whatlang 0.18 | 0.686 | 0.814 | 0.895 | 0.935 | 0.978 | 0.994 | 11,800 |
//! | lingua 1.8 | 0.936 | 0.972 | 0.987 | 0.996 | 0.999 | 1.000 | 760 |
//! | sentences | 2,781 | 6,180 | 6,371 | 2,681 | 780 | 160 | |
//!
//! lingua is the measured better choice for short sentences. textprep
//! does not depend on it: its models add about 143 MB to a release binary
//! (whatlang: 0.25 MB), it is 15 times slower, and lingua 1.8 needs Rust
//! 1.88. Use it through the [`Detector`] hook:
//!
//! ```ignore
//! // With lingua 1.8 as your own dependency.
//! let lingua = lingua::LanguageDetectorBuilder::from_languages(&[
//!     lingua::Language::English,
//!     lingua::Language::German,
//! ])
//! .build();
//! let detector = |text: &str| {
//!     let lang = lingua.detect_language_of(text)?;
//!     Language::from_code(&lang.iso_code_639_1().to_string())
//! };
//! let sentences = sentence_languages_with(text, &policy, &detector);
//! ```
//!
//! whatlang's own reliability flag is not used: it accepted only 36% of
//! single sentences, and gating on it lowered per-sentence accuracy in
//! documents mixing 6 and 3 sentences of two languages from 0.847 to 0.695.
//!
//! The 10-letter cutoff: below 10 letters whatlang is right on 51% of
//! sentences and lingua on 76%. Using the document's language there
//! instead changed per-sentence accuracy from 0.847 to 0.847 (whatlang)
//! and 0.977 to 0.974 (lingua) in the 6 + 3 documents, and from 0.846 to
//! 0.852 and 0.976 to 0.979 in documents mixing 8 and 1 sentences. Higher
//! cutoffs help only when other-language sentences are rare: at 20
//! letters, whatlang reaches 0.891 on 8 + 1 documents but drops to 0.818 on
//! 6 + 3.

use std::collections::HashSet;
use std::fmt;
use std::ops::Range;
use std::sync::Arc;

use crate::stopwords;

/// A language: its code, stopword set and (with the `stem` feature)
/// Snowball stemmer.
///
/// Cloning is cheap: the stopword set is shared until it is changed.
#[derive(Clone)]
pub struct Language {
    code: &'static str,
    stopwords: Arc<HashSet<String>>,
    #[cfg(feature = "stem")]
    stemmer: Option<rust_stemmers::Algorithm>,
}

impl Language {
    /// The language for a code or name, with NLTK's stopword list and, with
    /// the `stem` feature, its Snowball stemmer when Snowball has one.
    ///
    /// Accepts what [`stopwords::canonical_code`] accepts: ISO 639-1
    /// (`"de"`), ISO 639-3 (`"deu"`), the English name (`"german"`), with
    /// an optional region (`"de-AT"`). `None` for a language without a
    /// stopword list ([`stopwords::LANGUAGES`] lists the 33 that have one).
    ///
    /// ```
    /// use textprep::language::Language;
    /// let de = Language::from_code("de-AT").unwrap();
    /// assert_eq!(de.code(), "de");
    /// assert!(de.is_stopword("und"));
    /// assert!(Language::from_code("pl").is_none());
    /// ```
    pub fn from_code(code: &str) -> Option<Self> {
        let code = stopwords::canonical_code(code)?;
        let mut words = stopwords::shared(code);
        // Greek's list spells two elisions with U+2019 ("δ’", "ἀλλ’").
        if words.iter().any(|w| key(w) != *w) {
            words = Arc::new(words.iter().map(|w| key(w)).collect());
        }
        Some(Self {
            code,
            stopwords: words,
            #[cfg(feature = "stem")]
            stemmer: stemmer_for(code),
        })
    }

    /// English: NLTK's 198-word list and (with `stem`) the Porter2 stemmer.
    pub fn english() -> Self {
        Self::from_code("en").expect("English has a stopword list")
    }

    /// The language code, from [`stopwords::LANGUAGES`].
    pub fn code(&self) -> &'static str {
        self.code
    }

    /// The stopword set, lowercase and with `'` for U+2018 and U+2019.
    pub fn stopwords(&self) -> &HashSet<String> {
        &self.stopwords
    }

    /// Replace the stopword set, for example with a reference tool's own
    /// list. Each word is lowercased and has U+2018 and U+2019 replaced by
    /// `'`, as [`Language::is_stopword`] does with its argument.
    pub fn with_stopwords<I, S>(mut self, words: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        self.stopwords = Arc::new(words.into_iter().map(|w| key(w.as_ref())).collect());
        self
    }

    /// Add words to the stopword set, such as a domain list ("et", "al",
    /// "fig"). Words are normalized as in [`Language::with_stopwords`].
    pub fn with_extra_stopwords<I, S>(mut self, words: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        Arc::make_mut(&mut self.stopwords).extend(words.into_iter().map(|w| key(w.as_ref())));
        self
    }

    /// Whether `token` is a stopword. The token is lowercased and U+2018
    /// and U+2019 count as `'`, so `Don’t` matches `don't`.
    pub fn is_stopword(&self, token: &str) -> bool {
        self.stopwords.contains(&key(token))
    }

    /// Whether this language has a Snowball stemmer. Of the 33 languages,
    /// 18 do: ar, da, de, el, en, es, fi, fr, hu, it, nl, no, pt, ro, ru,
    /// sv, ta, tr.
    #[cfg(feature = "stem")]
    pub fn has_stemmer(&self) -> bool {
        self.stemmer.is_some()
    }

    /// The Snowball stem of `word`, lowercased first (Snowball expects
    /// lowercase) and with U+2018 and U+2019 replaced by `'`. A language
    /// without a stemmer returns the normalized word unchanged.
    ///
    /// ```
    /// use textprep::language::Language;
    /// assert_eq!(Language::english().stem("Generalizations"), "general");
    /// assert_eq!(Language::from_code("ca").unwrap().stem("Cases"), "cases");
    /// ```
    #[cfg(feature = "stem")]
    pub fn stem(&self, word: &str) -> String {
        let word = key(word);
        match self.stemmer {
            Some(algorithm) => rust_stemmers::Stemmer::create(algorithm)
                .stem(&word)
                .into_owned(),
            None => word,
        }
    }
}

impl Default for Language {
    fn default() -> Self {
        Self::english()
    }
}

impl PartialEq for Language {
    fn eq(&self, other: &Self) -> bool {
        #[cfg(feature = "stem")]
        if self.stemmer != other.stemmer {
            return false;
        }
        self.code == other.code
            && (Arc::ptr_eq(&self.stopwords, &other.stopwords) || self.stopwords == other.stopwords)
    }
}

impl fmt::Debug for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut d = f.debug_struct("Language");
        d.field("code", &self.code);
        d.field("stopwords", &self.stopwords.len());
        #[cfg(feature = "stem")]
        d.field("stemmer", &self.stemmer);
        d.finish()
    }
}

/// Lowercase and fold curly apostrophes: the form stopword sets store.
fn key(token: &str) -> String {
    stopwords::fold_apostrophes(token).to_lowercase()
}

/// The Snowball algorithm for a code in [`stopwords::LANGUAGES`], if any.
#[cfg(feature = "stem")]
fn stemmer_for(code: &str) -> Option<rust_stemmers::Algorithm> {
    use rust_stemmers::Algorithm::*;
    Some(match code {
        "ar" => Arabic,
        "da" => Danish,
        "de" => German,
        "el" => Greek,
        "en" => English,
        "es" => Spanish,
        "fi" => Finnish,
        "fr" => French,
        "hu" => Hungarian,
        "it" => Italian,
        "nl" => Dutch,
        "no" => Norwegian,
        "pt" => Portuguese,
        "ro" => Romanian,
        "ru" => Russian,
        "sv" => Swedish,
        "ta" => Tamil,
        "tr" => Turkish,
        _ => return None,
    })
}

/// Which language applies to which part of a text.
///
/// Detection policies need a [`Detector`]; see [`sentence_languages_with`].
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum LanguagePolicy {
    /// One language for every sentence. The default, with English.
    Fixed(Language),
    /// One detected language for the whole text; English when there is
    /// no detection.
    Detect,
    /// A detected language per sentence. A sentence too short to call
    /// (fewer than [`MIN_DETECT_CHARS`] letters) or without a detection
    /// gets the language detected for the whole text, and `fallback` when
    /// the whole text has no detection either.
    DetectPerSentence {
        /// Used when neither the sentence nor the whole text has a
        /// detection.
        fallback: Language,
    },
}

impl Default for LanguagePolicy {
    fn default() -> Self {
        Self::Fixed(Language::english())
    }
}

/// Where a sentence's language came from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum LanguageSource {
    /// The policy's fixed language.
    Fixed,
    /// Detected on the sentence itself.
    Sentence,
    /// Detected on the whole text.
    Document,
    /// The policy's fallback (English for `Detect`).
    Fallback,
}

/// The language assigned to one sentence.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct SentenceLanguage {
    /// Byte range of the sentence in the text, from
    /// [`crate::segment::sentence_byte_ranges`].
    pub range: Range<usize>,
    /// The language for this sentence.
    pub language: Language,
    /// How the language was chosen.
    pub source: LanguageSource,
}

/// Detects the language of a text.
///
/// Implemented for closures, so `&|text: &str| ...` works as a detector.
/// Return `None` when there is no answer, or when the answer has no
/// [`Language`] (use [`Language::from_code`] to map a detector's code).
pub trait Detector {
    /// The language of `text`, if the detector has one.
    fn detect(&self, text: &str) -> Option<Language>;
}

impl<F: Fn(&str) -> Option<Language>> Detector for F {
    fn detect(&self, text: &str) -> Option<Language> {
        self(text)
    }
}

/// Texts with fewer letters than this are too short to call: policies do
/// not pass them to the detector, and [`Detector`] implementations in
/// textprep return `None` for them.
pub const MIN_DETECT_CHARS: usize = 10;

/// Number of alphabetic characters in `s`.
fn letters(s: &str) -> usize {
    s.chars().filter(|c| c.is_alphabetic()).count()
}

/// The sentences of `text` (from [`crate::segment::sentence_byte_ranges`]),
/// each with the language `policy` assigns to it, using `detector` for the
/// detection policies.
///
/// ```
/// use textprep::language::{sentence_languages_with, Language, LanguagePolicy, LanguageSource};
///
/// // A stand-in detector: German if the text has "der", else none.
/// let detector = |text: &str| {
///     text.split_whitespace()
///         .any(|w| w.eq_ignore_ascii_case("der"))
///         .then(|| Language::from_code("de").unwrap())
/// };
/// let text = "Der Hund schläft im Garten. The dog sleeps. Ja.";
/// let policy = LanguagePolicy::DetectPerSentence { fallback: Language::english() };
/// let got: Vec<_> = sentence_languages_with(text, &policy, &detector)
///     .into_iter()
///     .map(|s| (s.language.code(), s.source))
///     .collect();
/// assert_eq!(
///     got,
///     [
///         ("de", LanguageSource::Sentence),
///         ("de", LanguageSource::Document),
///         ("de", LanguageSource::Document),
///     ]
/// );
/// ```
pub fn sentence_languages_with<D: Detector + ?Sized>(
    text: &str,
    policy: &LanguagePolicy,
    detector: &D,
) -> Vec<SentenceLanguage> {
    let call = |t: &str| {
        if letters(t) < MIN_DETECT_CHARS {
            None
        } else {
            detector.detect(t)
        }
    };
    let ranges = crate::segment::sentence_byte_ranges(text);
    let all = |language: &Language, source| {
        ranges
            .iter()
            .map(|range| SentenceLanguage {
                range: range.clone(),
                language: language.clone(),
                source,
            })
            .collect()
    };
    match policy {
        LanguagePolicy::Fixed(language) => all(language, LanguageSource::Fixed),
        LanguagePolicy::Detect => match call(text) {
            Some(language) => all(&language, LanguageSource::Document),
            None => all(&Language::english(), LanguageSource::Fallback),
        },
        LanguagePolicy::DetectPerSentence { fallback } => {
            let (document, document_source) = match call(text) {
                Some(language) => (language, LanguageSource::Document),
                None => (fallback.clone(), LanguageSource::Fallback),
            };
            ranges
                .into_iter()
                .map(|range| {
                    let (language, source) = match call(&text[range.clone()]) {
                        Some(language) => (language, LanguageSource::Sentence),
                        None => (document.clone(), document_source),
                    };
                    SentenceLanguage {
                        range,
                        language,
                        source,
                    }
                })
                .collect()
        }
    }
}

/// [`sentence_languages_with`] using [`WhatlangDetector`].
///
/// ```
/// use textprep::language::{sentence_languages, Language, LanguagePolicy, LanguageSource};
///
/// let text = "Der Hund schläft. Le chat dort.";
/// let policy = LanguagePolicy::Fixed(Language::english());
/// let got = sentence_languages(text, &policy);
/// assert_eq!(got.len(), 2);
/// assert_eq!(&text[got[1].range.clone()], "Le chat dort.");
/// assert!(got.iter().all(|s| s.language.code() == "en" && s.source == LanguageSource::Fixed));
/// ```
#[cfg(feature = "detect")]
pub fn sentence_languages(text: &str, policy: &LanguagePolicy) -> Vec<SentenceLanguage> {
    sentence_languages_with(text, policy, &WhatlangDetector::new())
}

/// The built-in detector: whatlang, limited to the 28 languages it shares
/// with [`stopwords::LANGUAGES`] (it lacks Albanian, Basque, Kazakh,
/// Tajik and Hinglish). The answer is whatlang's best guess; its
/// reliability flag is not used (see the module docs).
#[cfg(feature = "detect")]
#[derive(Clone, Copy, Debug, Default)]
#[non_exhaustive]
pub struct WhatlangDetector {}

#[cfg(feature = "detect")]
impl WhatlangDetector {
    /// The whatlang detector. Its language tables are built once per
    /// process and shared.
    pub fn new() -> Self {
        Self {}
    }
}

#[cfg(feature = "detect")]
impl Detector for WhatlangDetector {
    fn detect(&self, text: &str) -> Option<Language> {
        detect(text)
    }
}

/// The language of `text` from [`WhatlangDetector`]; `None` when `text`
/// has fewer than [`MIN_DETECT_CHARS`] letters or whatlang has no answer.
///
/// ```
/// use textprep::language::detect;
/// let lang = detect("Der schnelle braune Fuchs springt über den faulen Hund.");
/// assert_eq!(lang.map(|l| l.code()), Some("de"));
/// assert!(detect("Ja.").is_none());
/// ```
#[cfg(feature = "detect")]
pub fn detect(text: &str) -> Option<Language> {
    use std::sync::OnceLock;
    if letters(text) < MIN_DETECT_CHARS {
        return None;
    }
    static DETECTOR: OnceLock<whatlang::Detector> = OnceLock::new();
    let detector = DETECTOR.get_or_init(|| {
        whatlang::Detector::with_allowlist(WHATLANG.iter().map(|(lang, _)| *lang).collect())
    });
    let lang = detector.detect_lang(text)?;
    Language::from_code(WHATLANG.iter().find(|(l, _)| *l == lang)?.1)
}

/// whatlang's languages that have a stopword list, with their codes.
#[cfg(feature = "detect")]
const WHATLANG: [(whatlang::Lang, &str); 28] = {
    use whatlang::Lang::*;
    [
        (Ara, "ar"),
        (Aze, "az"),
        (Bel, "be"),
        (Ben, "bn"),
        (Cat, "ca"),
        (Cmn, "zh"),
        (Dan, "da"),
        (Deu, "de"),
        (Ell, "el"),
        (Eng, "en"),
        (Fin, "fi"),
        (Fra, "fr"),
        (Heb, "he"),
        (Hun, "hu"),
        (Ind, "id"),
        (Ita, "it"),
        (Nep, "ne"),
        (Nld, "nl"),
        (Nob, "no"),
        (Por, "pt"),
        (Ron, "ro"),
        (Rus, "ru"),
        (Slv, "sl"),
        (Spa, "es"),
        (Swe, "sv"),
        (Tam, "ta"),
        (Tur, "tr"),
        (Uzb, "uz"),
    ]
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extra_stopwords_do_not_leak_into_other_values() {
        let base = Language::english();
        let extended = base.clone().with_extra_stopwords(["Fig"]);
        assert!(extended.is_stopword("fig"));
        assert!(!base.is_stopword("fig"));
        assert!(!Language::english().is_stopword("fig"));
        assert_ne!(base, extended);
        assert_eq!(base, Language::english());
    }

    #[test]
    fn with_stopwords_replaces_and_normalizes() {
        let en = Language::english().with_stopwords(["Won\u{2019}t"]);
        assert_eq!(en.stopwords().len(), 1);
        assert!(en.is_stopword("won't"));
        assert!(!en.is_stopword("the"));
    }

    #[cfg(feature = "detect")]
    #[test]
    fn whatlang_table_codes_have_lists() {
        for (_, code) in WHATLANG {
            assert_eq!(Language::from_code(code).map(|l| l.code()), Some(code));
        }
    }
}
