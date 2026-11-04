//! Measures whatlang and lingua as sentence language detectors.
//!
//! Reads the Tatoeba samples that `fetch.sh` writes to `data/`, then:
//!
//! 1. Detects each sentence on its own and reports accuracy by sentence
//!    length (words).
//! 2. Builds mixed-language documents by interleaving sentences from two
//!    languages (6 from one, 3 from another), segments them with
//!    `textprep::segment::sentence_byte_ranges`, and scores the
//!    `DetectPerSentence` rule (detect a sentence when it has at least
//!    `min` letters, else use the document's language) for several `min`.
//!
//! Run: `cargo run --release` from this directory.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::time::Instant;

use lingua::{LanguageDetector, LanguageDetectorBuilder};
use textprep::language::{sentence_languages, sentence_languages_with, Language, LanguagePolicy};

/// Tatoeba code, ISO 639-1 code.
const LANGS: [(&str, &str); 17] = [
    ("eng", "en"),
    ("deu", "de"),
    ("fra", "fr"),
    ("spa", "es"),
    ("ita", "it"),
    ("por", "pt"),
    ("nld", "nl"),
    ("swe", "sv"),
    ("dan", "da"),
    ("nob", "no"),
    ("fin", "fi"),
    ("hun", "hu"),
    ("rus", "ru"),
    ("tur", "tr"),
    ("ell", "el"),
    ("ron", "ro"),
    ("cat", "ca"),
];

const BUCKETS: [(usize, usize, &str); 6] = [
    (1, 3, "1-3"),
    (4, 5, "4-5"),
    (6, 8, "6-8"),
    (9, 12, "9-12"),
    (13, 20, "13-20"),
    (21, usize::MAX, "21+"),
];

fn bucket(words: usize) -> usize {
    BUCKETS
        .iter()
        .position(|(lo, hi, _)| (*lo..=*hi).contains(&words))
        .unwrap_or(0)
}

/// A detector's answer: code (canonical textprep code), and whether the
/// detector itself marks it reliable.
type Answer = (Option<&'static str>, bool);

trait Detect {
    fn name(&self) -> &'static str;
    fn detect(&self, text: &str) -> Answer;
}

struct Whatlang {
    name: &'static str,
    detector: whatlang::Detector,
}

impl Detect for Whatlang {
    fn name(&self) -> &'static str {
        self.name
    }
    fn detect(&self, text: &str) -> Answer {
        match self.detector.detect(text) {
            Some(info) => (whatlang_code(info.lang()), info.is_reliable()),
            None => (None, false),
        }
    }
}

fn whatlang_code(lang: whatlang::Lang) -> Option<&'static str> {
    textprep::stopwords::canonical_code(lang.code())
}

struct Lingua {
    name: &'static str,
    detector: LanguageDetector,
}

impl Detect for Lingua {
    fn name(&self) -> &'static str {
        self.name
    }
    fn detect(&self, text: &str) -> Answer {
        let values = self.detector.compute_language_confidence_values(text);
        match values.first() {
            Some((lang, confidence)) => {
                let code = lang.iso_code_639_1().to_string().to_lowercase();
                (
                    textprep::stopwords::canonical_code(&code),
                    *confidence >= 0.5,
                )
            }
            None => (None, false),
        }
    }
}

fn lingua_languages() -> Vec<lingua::Language> {
    use lingua::Language::*;
    vec![
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
    ]
}

/// Small deterministic generator (xorshift64*).
struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
    fn shuffle<T>(&mut self, v: &mut [T]) {
        for i in (1..v.len()).rev() {
            let j = (self.next() % (i as u64 + 1)) as usize;
            v.swap(i, j);
        }
    }
}

fn letters(s: &str) -> usize {
    s.chars().filter(|c| c.is_alphabetic()).count()
}

fn main() {
    let data = Path::new(env!("CARGO_MANIFEST_DIR")).join("data");
    let mut pools: BTreeMap<&'static str, Vec<String>> = BTreeMap::new();
    for (tatoeba, code) in LANGS {
        let path = data.join(format!("{tatoeba}.tsv"));
        let text = fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("{}: {e}; run fetch.sh first", path.display()));
        let sentences: Vec<String> = text
            .lines()
            .filter_map(|l| l.split_once('\t').map(|(_, s)| s.trim().to_string()))
            .filter(|s| !s.is_empty())
            .collect();
        pools.insert(code, sentences);
    }
    let total: usize = pools.values().map(Vec::len).sum();
    println!("# Sentences: {total} in {} languages", pools.len());
    for (code, v) in &pools {
        print!("{code}={} ", v.len());
    }
    println!();

    let allow: Vec<whatlang::Lang> = whatlang::Lang::all()
        .iter()
        .copied()
        .filter(|l| whatlang_code(*l).is_some())
        .collect();
    let t = Instant::now();
    let lingua_high = LanguageDetectorBuilder::from_languages(&lingua_languages())
        .with_preloaded_language_models()
        .build();
    let lingua_high_load = t.elapsed();
    let t = Instant::now();
    let lingua_low = LanguageDetectorBuilder::from_languages(&lingua_languages())
        .with_low_accuracy_mode()
        .with_preloaded_language_models()
        .build();
    let lingua_low_load = t.elapsed();
    println!(
        "# lingua model load: high {:.2}s, low {:.2}s",
        lingua_high_load.as_secs_f64(),
        lingua_low_load.as_secs_f64()
    );
    let detectors: Vec<Box<dyn Detect>> = vec![
        Box::new(Whatlang {
            name: "whatlang-28",
            detector: whatlang::Detector::with_allowlist(allow.clone()),
        }),
        Box::new(Whatlang {
            name: "whatlang-all",
            detector: whatlang::Detector::new(),
        }),
        Box::new(Lingua {
            name: "lingua-29-high",
            detector: lingua_high,
        }),
        Box::new(Lingua {
            name: "lingua-29-low",
            detector: lingua_low,
        }),
    ];

    // 1. Isolated sentences.
    let mut items: Vec<(&'static str, &str)> = Vec::new();
    for (code, v) in &pools {
        for s in v {
            items.push((code, s.as_str()));
        }
    }
    let mut counts = [0usize; BUCKETS.len()];
    for (_, s) in &items {
        counts[bucket(s.split_whitespace().count())] += 1;
    }
    println!("\n## Isolated sentences: top-1 accuracy by length in words");
    println!("(rel = share the detector marks reliable; acc|rel = accuracy within those)\n");
    print!("| detector | sents/s |");
    for (i, (_, _, name)) in BUCKETS.iter().enumerate() {
        print!(" {name} (n={}) |", counts[i]);
    }
    println!(" all | rel | acc\\|rel |");
    print!("|---|---|");
    for _ in BUCKETS {
        print!("---|");
    }
    println!("---|---|---|");
    let mut answers: BTreeMap<&'static str, Vec<Answer>> = BTreeMap::new();
    for d in &detectors {
        let t = Instant::now();
        let got: Vec<Answer> = items.iter().map(|(_, s)| d.detect(s)).collect();
        let rate = items.len() as f64 / t.elapsed().as_secs_f64();
        let mut right = [0usize; BUCKETS.len()];
        let (mut all, mut rel, mut rel_right) = (0, 0, 0);
        for ((gold, s), (code, reliable)) in items.iter().zip(&got) {
            let ok = *code == Some(*gold);
            if ok {
                right[bucket(s.split_whitespace().count())] += 1;
                all += 1;
            }
            if *reliable {
                rel += 1;
                rel_right += ok as usize;
            }
        }
        print!("| {} | {:.0} |", d.name(), rate);
        for i in 0..BUCKETS.len() {
            print!(" {:.3} |", right[i] as f64 / counts[i].max(1) as f64);
        }
        println!(
            " {:.3} | {:.3} | {:.3} |",
            all as f64 / items.len() as f64,
            rel as f64 / items.len() as f64,
            rel_right as f64 / rel.max(1) as f64
        );
        answers.insert(d.name(), got);
    }

    // Accuracy by letter count, to place the "too short to call" cutoff.
    const LETTERS: [(usize, usize, &str); 6] = [
        (0, 4, "<5"),
        (5, 9, "5-9"),
        (10, 14, "10-14"),
        (15, 19, "15-19"),
        (20, 29, "20-29"),
        (30, usize::MAX, "30+"),
    ];
    let lbucket = |s: &str| {
        let n = letters(s);
        LETTERS
            .iter()
            .position(|(lo, hi, _)| (*lo..=*hi).contains(&n))
            .unwrap()
    };
    let mut lcounts = [0usize; LETTERS.len()];
    for (_, s) in &items {
        lcounts[lbucket(s)] += 1;
    }
    println!("\n## Isolated sentences: top-1 accuracy by letters\n");
    print!("| detector |");
    for (i, (_, _, name)) in LETTERS.iter().enumerate() {
        print!(" {name} (n={}) |", lcounts[i]);
    }
    println!();
    print!("|---|");
    for _ in LETTERS {
        print!("---|");
    }
    println!();
    for d in &detectors {
        let mut right = [0usize; LETTERS.len()];
        for ((gold, s), (code, _)) in items.iter().zip(&answers[d.name()]) {
            if *code == Some(*gold) {
                right[lbucket(s)] += 1;
            }
        }
        print!("| {} |", d.name());
        for i in 0..LETTERS.len() {
            print!(" {:.3} |", right[i] as f64 / lcounts[i].max(1) as f64);
        }
        println!();
    }

    // Cross-check: textprep::language::detect agrees with the matching
    // detector's top-1 answer on sentences with enough letters.
    let backend = "whatlang-28";
    let mut disagree = 0;
    for ((_, s), (code, _)) in items.iter().zip(&answers[backend]) {
        let ours = textprep::language::detect(s).map(|l| l.code());
        let theirs = if letters(s) >= textprep::language::MIN_DETECT_CHARS {
            *code
        } else {
            None
        };
        disagree += (ours != theirs) as usize;
    }
    println!(
        "\ntextprep::language::detect vs {backend} top-1 (with MIN_DETECT_CHARS = {}): {disagree} disagreements in {} sentences",
        textprep::language::MIN_DETECT_CHARS,
        items.len()
    );

    for (primary_n, secondary_n) in [(6, 3), (8, 1)] {
        mixed(&pools, &detectors, primary_n, secondary_n);
    }
}

/// Mixed documents: `primary_n` sentences of one language and
/// `secondary_n` of another, in random order.
fn mixed(
    pools: &BTreeMap<&'static str, Vec<String>>,
    detectors: &[Box<dyn Detect>],
    primary_n: usize,
    secondary_n: usize,
) {
    let mut rng = Rng(0x9E37_79B9_7F4A_7C15);
    let mut shuffled = pools.clone();
    for v in shuffled.values_mut() {
        rng.shuffle(v);
    }
    let codes: Vec<&'static str> = shuffled.keys().copied().collect();
    struct Doc {
        text: String,
        spans: Vec<(std::ops::Range<usize>, &'static str)>,
        primary: &'static str,
    }
    let mut docs: Vec<Doc> = Vec::new();
    let mut failures = 0;
    let mut i = 0usize;
    while failures < codes.len() * 4 {
        let p = codes[i % codes.len()];
        let s = codes[(i * 7 + 3 + i / codes.len()) % codes.len()];
        i += 1;
        if p == s || shuffled[p].len() < primary_n || shuffled[s].len() < secondary_n {
            failures += 1;
            continue;
        }
        failures = 0;
        let mut parts: Vec<(&'static str, String)> = Vec::new();
        for _ in 0..primary_n {
            parts.push((p, shuffled.get_mut(p).unwrap().pop().unwrap()));
        }
        for _ in 0..secondary_n {
            parts.push((s, shuffled.get_mut(s).unwrap().pop().unwrap()));
        }
        rng.shuffle(&mut parts);
        let mut text = String::new();
        let mut spans = Vec::new();
        for (code, sentence) in parts {
            if !text.is_empty() {
                text.push(' ');
            }
            let start = text.len();
            text.push_str(&sentence);
            spans.push((start..text.len(), code));
        }
        docs.push(Doc {
            text,
            spans,
            primary: p,
        });
    }
    let mut segmented = 0;
    let mut mismatched = 0;
    // (gold, words, sentence text, doc index)
    let mut gold_sentences: Vec<(&'static str, usize, String, usize)> = Vec::new();
    for (di, doc) in docs.iter().enumerate() {
        for r in textprep::segment::sentence_byte_ranges(&doc.text) {
            segmented += 1;
            match doc.spans.iter().find(|(span, _)| *span == r) {
                Some((_, code)) => {
                    let s = doc.text[r].to_string();
                    gold_sentences.push((code, s.split_whitespace().count(), s, di));
                }
                None => mismatched += 1,
            }
        }
    }
    println!(
        "\n## Mixed documents, {primary_n} + {secondary_n} sentences from two languages: {} documents, {} segments, {} not matching one source sentence (excluded)\n",
        docs.len(),
        segmented,
        mismatched
    );
    let doc_answers: BTreeMap<&'static str, Vec<Answer>> = detectors
        .iter()
        .map(|d| {
            (
                d.name(),
                docs.iter().map(|doc| d.detect(&doc.text)).collect(),
            )
        })
        .collect();
    for d in detectors {
        let right = docs
            .iter()
            .zip(&doc_answers[d.name()])
            .filter(|(doc, (code, _))| *code == Some(doc.primary))
            .count();
        println!(
            "- document language = majority language, {}: {:.3}",
            d.name(),
            right as f64 / docs.len() as f64
        );
    }
    let sent_answers: BTreeMap<&'static str, Vec<Answer>> = detectors
        .iter()
        .map(|d| {
            (
                d.name(),
                gold_sentences
                    .iter()
                    .map(|(_, _, s, _)| d.detect(s))
                    .collect(),
            )
        })
        .collect();
    let mut counts = [0usize; BUCKETS.len()];
    for (_, w, _, _) in &gold_sentences {
        counts[bucket(*w)] += 1;
    }
    println!("\nDetectPerSentence accuracy by sentence length (words). A sentence with fewer than `min` letters takes the document's language. `rel`: whatlang's answer is used only when it marks it reliable.\n");
    print!("| detector | min letters |");
    for (i, (_, _, name)) in BUCKETS.iter().enumerate() {
        print!(" {name} (n={}) |", counts[i]);
    }
    println!(" all |");
    print!("|---|---|");
    for _ in BUCKETS {
        print!("---|");
    }
    println!("---|");
    let mut rows: Vec<(&str, bool, usize)> = vec![("whatlang-28", true, 0)];
    for d in detectors {
        for min in [0usize, 5, 10, 15, 20] {
            rows.push((d.name(), false, min));
        }
    }
    for (name, gated, min) in rows {
        let mut right = [0usize; BUCKETS.len()];
        let mut all = 0;
        for ((gold, w, s, di), (code, reliable)) in gold_sentences.iter().zip(&sent_answers[name]) {
            let (doc_code, doc_rel) = doc_answers[name][*di];
            let doc_lang = if doc_rel || !gated { doc_code } else { None }.unwrap_or("en");
            let use_sentence = letters(s) >= min && code.is_some() && (*reliable || !gated);
            let chosen = if use_sentence {
                code.unwrap()
            } else {
                doc_lang
            };
            if chosen == *gold {
                right[bucket(*w)] += 1;
                all += 1;
            }
        }
        print!("| {}{} | {} |", name, if gated { " rel" } else { "" }, min);
        for i in 0..BUCKETS.len() {
            print!(" {:.3} |", right[i] as f64 / counts[i].max(1) as f64);
        }
        println!(" {:.3} |", all as f64 / gold_sentences.len() as f64);
    }

    // textprep's DetectPerSentence on the same documents: the built-in
    // whatlang detector, and each harness detector through the Detector hook.
    let policy = LanguagePolicy::DetectPerSentence {
        fallback: Language::english(),
    };
    let score = |assign: &dyn Fn(&str) -> Vec<textprep::language::SentenceLanguage>| {
        let (mut right, mut n) = (0, 0);
        let mut sources: BTreeMap<String, usize> = BTreeMap::new();
        for doc in &docs {
            for a in assign(&doc.text) {
                *sources.entry(format!("{:?}", a.source)).or_default() += 1;
                if let Some((_, code)) = doc.spans.iter().find(|(span, _)| *span == a.range) {
                    n += 1;
                    right += (a.language.code() == *code) as usize;
                }
            }
        }
        (right as f64 / n.max(1) as f64, n, sources)
    };
    let (acc, n, sources) = score(&|t| sentence_languages(t, &policy));
    println!("\ntextprep DetectPerSentence, built-in whatlang: {acc:.3} over {n} sentences; sources {sources:?}");
    for d in detectors {
        let hook = |t: &str| d.detect(t).0.and_then(Language::from_code);
        let (acc, n, _) = score(&|t| sentence_languages_with(t, &policy, &hook));
        println!(
            "textprep DetectPerSentence via Detector hook, {}: {acc:.3} over {n} sentences",
            d.name()
        );
    }
}
