//! Dictionary (gazetteer) entity tagging over noisy web text.
//!
//! Demonstrates `textprep`'s Aho-Corasick keyword matcher (`FlashText`), the
//! headline "fast keyword matching" feature, on text that first needs HTML
//! entity cleanup. Unlike the tokenize-then-normalize examples, this matches a
//! whole gazetteer against raw text in a single pass and reports character
//! offsets, with longest-match resolution for overlapping patterns.
//!
//! Distinctive ops shown here (not in the other examples):
//! - `html::decode_entities`: strip `&amp;` / `&#x2019;` web artifacts.
//! - `FlashText`: case-insensitive multi-pattern matching with char offsets and
//!   canonical values, including leftmost-longest overlap resolution.
//! - `FlashText::find_into`: allocation-reusing variant for hot loops.
//!
//! Run:
//! `cargo run --example gazetteer_tagging`

use textprep::{decode_entities, FlashText, KeywordMatch};

/// Recover the matched substring from the original (decoded) text by char offset.
///
/// `KeywordMatch` offsets are Unicode scalar (char) offsets, not byte offsets.
fn slice_chars(text: &str, start: usize, end: usize) -> String {
    text.chars().skip(start).take(end - start).collect()
}

fn main() {
    // 1. Build a gazetteer: surface form -> canonical value. Aho-Corasick is
    //    built lazily on first `find`, so adds are cheap.
    let mut gz = FlashText::new();
    gz.add_keyword("New York", "NYC");
    gz.add_keyword("York", "YORK"); // overlaps "New York"; longest wins
    gz.add_keyword("San Francisco", "SF");
    gz.add_keyword("AT&T", "ATT");
    gz.add_keyword("café", "CAFE"); // multibyte: offsets stay char-correct

    // 2. Raw text as it might arrive from a scraper: HTML entities + casing noise.
    let raw = "We opened a caf&#xe9; in new york, near AT&amp;T, \
               then flew to SAN FRANCISCO for York Fashion Week.";

    // Decode HTML entities BEFORE matching so "AT&amp;T" can match "AT&T" and
    // "caf&#xe9;" becomes "café".
    let decoded = decode_entities(raw);

    println!("raw:     {raw:?}");
    println!("decoded: {decoded:?}\n");

    let matches = gz.find(&decoded);

    println!("matches ({}):", matches.len());
    for KeywordMatch {
        value, start, end, ..
    } in &matches
    {
        let surface = slice_chars(&decoded, *start, *end);
        println!("  {value:<5} chars {start:>2}..{end:<2}  surface={surface:?}");
    }
    println!();

    // 3. Tag a second document reusing the same matcher and a shared output
    //    buffer via `find_into` (no per-call Vec allocation).
    let docs = [
        "Direct from new york to san francisco.",
        "AT&amp;T sponsored the york stage.",
    ];
    let mut buf = Vec::new();
    let mut total = 0usize;
    for doc in &docs {
        let clean = decode_entities(doc);
        gz.find_into(&clean, &mut buf);
        total += buf.len();
        let tags: Vec<&str> = buf.iter().map(|m| m.value.as_str()).collect();
        println!("doc {clean:?}\n    -> {tags:?}");
    }

    // Self-checks: leftmost-longest picks "New York" (NYC) over "York";
    // the multibyte "café" and the decoded "AT&T" both match with correct spans.
    let values: Vec<&str> = matches.iter().map(|m| m.value.as_str()).collect();
    assert_eq!(values, ["CAFE", "NYC", "ATT", "SF", "YORK"]);
    // The standalone "York Fashion Week" mention tags as YORK, not NYC.
    assert!(matches.iter().any(|m| m.value == "YORK"));
    // "café" round-trips through HTML decode + char-offset slicing.
    let cafe = matches.iter().find(|m| m.value == "CAFE").unwrap();
    assert_eq!(slice_chars(&decoded, cafe.start, cafe.end), "café");
    // Second pass found NYC+SF in doc 0 and ATT+YORK in doc 1.
    assert_eq!(total, 4);

    println!("\nok");
}
