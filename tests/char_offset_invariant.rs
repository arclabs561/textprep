//! The load-bearing contract textprep's consumers (anno, symproj, ...) rely on:
//! token/match `start`..`end` are CHARACTER offsets, so indexing the source's
//! `chars()` vector by `[start..end]` reproduces the surface text exactly. A
//! byte-vs-char offset regression would silently corrupt every downstream span
//! on multibyte input (and panic on byte-slicing). These guard that invariant
//! on emoji / accented / CJK text.

use textprep::{tokenize_with_offsets, FlashText};

const MULTIBYTE: &str = "a café ☕ and 日本語 then naïve coöperate end";

#[test]
fn tokenize_offsets_are_char_indices_on_multibyte() {
    let chars: Vec<char> = MULTIBYTE.chars().collect();
    let toks = tokenize_with_offsets(MULTIBYTE);
    assert!(!toks.is_empty());
    for t in &toks {
        assert!(
            t.start <= t.end && t.end <= chars.len(),
            "offsets out of range: {t:?}"
        );
        let slice: String = chars[t.start..t.end].iter().collect();
        assert_eq!(
            slice, t.text,
            "char-offset span [{}..{}] = {slice:?} but token.text = {:?}",
            t.start, t.end, t.text
        );
    }
}

#[test]
fn flash_match_offsets_are_char_indices_on_multibyte() {
    let mut ft = FlashText::new();
    ft.add_keyword("café", "ACCENT");
    ft.add_keyword("日本語", "CJK");
    ft.add_keyword("naïve", "DIAERESIS");

    let chars: Vec<char> = MULTIBYTE.chars().collect();
    let matches = ft.find(MULTIBYTE);
    assert!(
        !matches.is_empty(),
        "expected keyword matches in multibyte text"
    );
    for m in &matches {
        assert!(
            m.start <= m.end && m.end <= chars.len(),
            "match offsets out of range"
        );
        let slice: String = chars[m.start..m.end].iter().collect();
        assert_eq!(
            slice, m.keyword,
            "char-offset span [{}..{}] = {slice:?} but match.keyword = {:?}",
            m.start, m.end, m.keyword
        );
    }
}
