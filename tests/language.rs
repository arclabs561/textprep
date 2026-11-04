//! Known-answer tests for stopword lists, stemmers and language policies.

use textprep::language::{
    sentence_languages_with, Language, LanguagePolicy, LanguageSource, MIN_DETECT_CHARS,
};
use textprep::stopwords;

/// (code, words in NLTK's file, distinct words) for `stop-words` 0.10.1
/// with its `nltk` feature. A changed count means the packaged list changed.
const LIST_SIZES: [(&str, usize, usize); 33] = [
    ("ar", 754, 701),
    ("az", 165, 165),
    ("be", 224, 224),
    ("bn", 398, 398),
    ("ca", 278, 278),
    ("da", 94, 94),
    ("de", 232, 232),
    ("el", 265, 265),
    ("en", 198, 198),
    ("es", 313, 313),
    ("eu", 326, 258),
    ("fi", 235, 229),
    ("fr", 157, 157),
    ("he", 221, 193),
    ("hinglish", 1036, 1036),
    ("hu", 199, 198),
    ("id", 758, 757),
    ("it", 279, 279),
    ("kk", 324, 276),
    ("ne", 255, 254),
    ("nl", 101, 101),
    ("no", 176, 172),
    ("pt", 207, 207),
    ("ro", 356, 356),
    ("ru", 151, 151),
    ("sl", 1784, 1784),
    ("sq", 237, 237),
    ("sv", 114, 114),
    ("ta", 125, 125),
    ("tg", 163, 163),
    ("tr", 53, 53),
    ("uz", 288, 287),
    ("zh", 841, 841),
];

#[test]
fn every_nltk_language_has_its_list() {
    assert_eq!(stopwords::LANGUAGES.len(), 33);
    for (code, (expected, len, distinct)) in stopwords::LANGUAGES.iter().zip(LIST_SIZES) {
        assert_eq!(*code, expected);
        assert_eq!(stopwords::list(code).map(<[_]>::len), Some(len), "{code}");
        assert_eq!(stopwords::get(code).len(), distinct, "{code}");
        let language = Language::from_code(code).unwrap();
        assert_eq!(language.code(), *code);
        // Greek lists "δ'" and "δ’", "ἀλλ'" and "ἀλλ’"; folding merges them.
        let folded = if *code == "el" {
            distinct - 2
        } else {
            distinct
        };
        assert_eq!(language.stopwords().len(), folded, "{code}");
    }
    assert_eq!(Language::english(), Language::from_code("English").unwrap());
}

#[test]
fn curly_apostrophes_match_straight_ones() {
    let en = Language::english();
    for token in ["don't", "don\u{2019}t", "Don\u{2018}t", "SHOULD\u{2019}VE"] {
        assert!(en.is_stopword(token), "{token}");
    }
    // Greek's list spells this elision both ways.
    let el = Language::from_code("el").unwrap();
    assert!(el.is_stopword("\u{3b4}'"));
    assert!(el.is_stopword("\u{3b4}\u{2019}"));
    assert!(stopwords::get("el").contains("\u{3b4}\u{2019}"));
}

#[test]
fn domain_words_extend_one_language() {
    let de = Language::from_code("de")
        .unwrap()
        .with_extra_stopwords(["Abb", "vgl"]);
    assert!(de.is_stopword("abb") && de.is_stopword("Vgl") && de.is_stopword("und"));
    assert!(!Language::from_code("de").unwrap().is_stopword("abb"));
}

#[test]
fn fixed_policy_assigns_every_sentence() {
    let text = "Der Hund schläft. The dog sleeps.";
    let fr = Language::from_code("fr").unwrap();
    let none = |_: &str| -> Option<Language> { None };
    let got = sentence_languages_with(text, &LanguagePolicy::Fixed(fr.clone()), &none);
    assert_eq!(got.len(), 2);
    assert!(got
        .iter()
        .all(|s| s.language == fr && s.source == LanguageSource::Fixed));
    assert_eq!(
        LanguagePolicy::default(),
        LanguagePolicy::Fixed(Language::english())
    );
}

#[test]
fn caller_detector_is_used_and_never_sees_short_text() {
    use std::cell::RefCell;
    let seen = RefCell::new(Vec::new());
    let detector = |text: &str| {
        seen.borrow_mut().push(text.to_string());
        // A stand-in: Swedish for any text containing "och".
        text.contains(" och ")
            .then(|| Language::from_code("sv").unwrap())
    };
    let text = "Hon gick hem och åt middag. Nej. Something else entirely.";
    let policy = LanguagePolicy::DetectPerSentence {
        fallback: Language::english(),
    };
    let got: Vec<(&str, LanguageSource)> = sentence_languages_with(text, &policy, &detector)
        .into_iter()
        .map(|s| (s.language.code(), s.source))
        .collect();
    assert_eq!(
        got,
        [
            ("sv", LanguageSource::Sentence),
            ("sv", LanguageSource::Document),
            ("sv", LanguageSource::Document),
        ]
    );
    let seen = seen.into_inner();
    assert_eq!(seen.len(), 3, "whole text and two long sentences: {seen:?}");
    assert!(seen
        .iter()
        .all(|t| t.chars().filter(|c| c.is_alphabetic()).count() >= MIN_DETECT_CHARS));
}

#[cfg(feature = "stem")]
mod stem {
    use super::*;

    /// Word>stem pairs from the sample vocabularies published with each
    /// Snowball algorithm (snowballstem.org/algorithms/<language>/stemmer.html):
    /// all 80 English pairs, 24 of the others.
    const SNOWBALL_SAMPLES: [(&str, &str); 5] = [
        (
            "en",
            "consign>consign consigned>consign consigning>consign consignment>consign \
     consist>consist consisted>consist consistency>consist consistent>consist \
     consistently>consist consisteth>consisteth consisting>consist \
     consistory>consistori consists>consist consolate>consol consolation>consol \
     consolations>consol consolatory>consolatori console>consol consoled>consol \
     consoler>consol consoles>consol consolidate>consolid consolidated>consolid \
     consolidating>consolid consolidation>consolid consoling>consol \
     consolingly>consol consols>consol consonancy>conson consonant>conson \
     consort>consort consorted>consort consortest>consortest consorting>consort \
     conspectuities>conspectu conspicuous>conspicu conspicuously>conspicu \
     conspir>conspir conspiracy>conspiraci conspirant>conspir knack>knack \
     knackeries>knackeri knacks>knack knag>knag knapp>knapp knapsack>knapsack \
     knav>knav knave>knave knaveries>knaveri knavery>knaveri knaves>knave \
     knavish>knavish knead>knead kneaded>knead kneading>knead knee>knee kneel>kneel \
     kneeled>kneel kneeling>kneel kneels>kneel knees>knee knell>knell knelled>knell \
     knelt>knelt knew>knew knewest>knewest knick>knick knicknacks>knicknack knif>knif \
     knife>knife knight>knight knighted>knight knighthood>knighthood \
     knighthoods>knighthood knightly>knight knights>knight knightsbridge>knightsbridg \
     knit>knit knits>knit knitted>knit",
        ),
        (
            "de",
            "aufeinander>aufeinand aufeinanderbiss>aufeinanderbiss \
     aufeinanderfolge>aufeinanderfolg aufeinanderfolgen>aufeinanderfolg \
     aufeinanderfolgend>aufeinanderfolg aufeinanderfolgende>aufeinanderfolg \
     aufeinanderfolgenden>aufeinanderfolg aufeinanderfolgender>aufeinanderfolg \
     aufeinanderfolgt>aufeinanderfolgt aufeinanderfolgten>aufeinanderfolgt \
     aufeinanderschlügen>aufeinanderschlug aufenthalt>aufenthalt kategorie>kategori \
     kategorien>kategori kategorisch>kategor kategorische>kategor \
     kategorischen>kategor kategorischer>kategor kater>kat katerliede>katerlied \
     katern>kat katers>kat kathedrale>kathedral kathinka>kathinka",
        ),
        (
            "fr",
            "continu>continu continua>continu continuait>continu continuant>continu \
     continuation>continu continue>continu continuel>continuel continuelle>continuel \
     continuellement>continuel continuelles>continuel continuels>continuel \
     continuer>continu main>main mains>main maintenaient>mainten maintenait>mainten \
     maintenant>mainten maintenir>mainten maintenue>maintenu maintien>maintien \
     maintint>maintint maire>mair maires>mair mairie>mair",
        ),
        (
            "es",
            "che>che checa>chec checar>chec checo>chec checoslovaquia>checoslovaqui \
     chedraoui>chedraoui chefs>chefs cheliabinsk>cheliabinsk chelo>chel \
     chemical>chemical chemicalweek>chemicalweek chemise>chemis torá>tor tos>tos \
     toscano>toscan tosferina>tosferin tostado>tost tota>tot total>total \
     totales>total totalidad>total totalizó>totaliz totalmente>total totopos>totop",
        ),
        (
            "ru",
            "в>в вавиловка>вавиловк вагнера>вагнер вагон>вагон вагона>вагон вагоне>вагон \
     вагонов>вагон вагоном>вагон вагоны>вагон важная>важн важнее>важн важнейшие>важн \
     п>п па>па пава>пав павел>павел павильон>павильон павильонам>павильон павла>павл \
     павлиний>павлин павлиньи>павлин павлиньим>павлин павлович>павлович \
     павловна>павловн",
        ),
    ];

    #[test]
    fn snowball_published_samples() {
        for (code, pairs) in SNOWBALL_SAMPLES {
            let language = Language::from_code(code).unwrap();
            let mut n = 0;
            for pair in pairs.split_whitespace() {
                let (word, stem) = pair.split_once('>').unwrap();
                assert_eq!(language.stem(word), stem, "{code}: {word}");
                n += 1;
            }
            assert_eq!(n, if code == "en" { 80 } else { 24 }, "{code}");
        }
    }

    #[test]
    fn eighteen_languages_have_a_stemmer() {
        let with: Vec<&str> = stopwords::LANGUAGES
            .iter()
            .copied()
            .filter(|code| Language::from_code(code).unwrap().has_stemmer())
            .collect();
        assert_eq!(
            with,
            [
                "ar", "da", "de", "el", "en", "es", "fi", "fr", "hu", "it", "nl", "no", "pt", "ro",
                "ru", "sv", "ta", "tr"
            ]
        );
        // Without a stemmer, stem only normalizes.
        let ca = Language::from_code("ca").unwrap();
        assert_eq!(ca.stem("Cases\u{2019}"), "cases'");
        assert_eq!(Language::english().stem("Networks"), "network");
    }
}

#[cfg(feature = "detect")]
mod detect {
    use super::*;
    use textprep::language::{detect, sentence_languages};

    const PARAGRAPH: &str = "Der Bundestag hat heute ein neues Gesetz über erneuerbare \
        Energien beschlossen. Die Abgeordneten diskutierten mehrere Stunden lang über \
        die Kosten. The committee will publish its final report on renewable energy \
        next spring. Danach geht die Debatte im Bundesrat weiter. Ja.";

    fn codes(text: &str, policy: &LanguagePolicy) -> Vec<(String, &'static str, LanguageSource)> {
        sentence_languages(text, policy)
            .into_iter()
            .map(|s| (text[s.range].to_string(), s.language.code(), s.source))
            .collect()
    }

    #[test]
    fn per_sentence_policy_on_a_two_language_paragraph() {
        let policy = LanguagePolicy::DetectPerSentence {
            fallback: Language::english(),
        };
        let got: Vec<(&'static str, LanguageSource)> = codes(PARAGRAPH, &policy)
            .into_iter()
            .map(|(_, code, source)| (code, source))
            .collect();
        use LanguageSource::*;
        assert_eq!(
            got,
            [
                ("de", Sentence),
                ("de", Sentence),
                ("en", Sentence),
                ("de", Sentence),
                // "Ja." is too short to call: it takes the paragraph's language.
                ("de", Document),
            ]
        );
    }

    #[test]
    fn document_policy_uses_one_language() {
        let got = codes(PARAGRAPH, &LanguagePolicy::Detect);
        assert_eq!(got.len(), 5);
        assert!(got
            .iter()
            .all(|(_, code, source)| *code == "de" && *source == LanguageSource::Document));
    }

    #[test]
    fn short_text_falls_back() {
        let fr = Language::from_code("fr").unwrap();
        let policy = LanguagePolicy::DetectPerSentence {
            fallback: fr.clone(),
        };
        let got = sentence_languages("Nein. Ja.", &policy);
        assert_eq!(got.len(), 2);
        assert!(got
            .iter()
            .all(|s| s.language == fr && s.source == LanguageSource::Fallback));
        let got = sentence_languages("Ok.", &LanguagePolicy::Detect);
        assert_eq!(got[0].language, Language::english());
        assert_eq!(got[0].source, LanguageSource::Fallback);
        assert!(detect("Danke!").is_none());
    }
}
