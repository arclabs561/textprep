#![cfg(feature = "source-map")]
use std::ops::Range;
use textprep::clean::{CleanOptions, SourceKind};

fn assert_scalar_source(
    original: &str,
    analyzed: &str,
    out: Range<usize>,
    input: Range<usize>,
    kind: SourceKind,
) {
    let (doc, map) = CleanOptions::default().prepare_mapped(original).unwrap();
    assert_eq!(doc, analyzed);
    let runs = map.project_bytes(analyzed, out).unwrap();
    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0].source_bytes, Some(input));
    assert_eq!(runs[0].kind, kind);
}

#[test]
fn projects_unicode_composition_expansion_and_reordering() {
    use SourceKind::{Direct, Transformed};
    assert_scalar_source("cafE\u{301}", "cafÉ", 3..5, 3..6, Transformed);
    assert_scalar_source("\u{1100}\u{1161}\u{11a8}", "각", 0..3, 0..9, Transformed);
    // U+0344 expands into diaeresis and acute; both derive from the same scalar.
    assert_scalar_source("\u{0344}", "\u{308}\u{301}", 2..4, 0..2, Transformed);
    // These marks reorder, and each still names its own original bytes.
    assert_scalar_source("q\u{315}\u{300}", "q\u{300}\u{315}", 1..3, 3..5, Direct);
    assert_scalar_source("q\u{315}\u{300}", "q\u{300}\u{315}", 3..5, 1..3, Direct);
    // The grave composes across a mark; the intervening bytes are not contributors.
    for (raw, cleaned, ranges) in [
        ("a\u{315}\u{300}", "à\u{315}", vec![0..1, 3..5]),
        ("e\u{200b}\u{301}", "é", vec![0..1, 4..6]),
    ] {
        let (doc, map) = CleanOptions::default().prepare_mapped(raw).unwrap();
        assert_eq!(doc, cleaned);
        let runs = map.project_bytes(cleaned, 0..2).unwrap();
        assert_eq!(runs[0].kind, Transformed);
        assert_eq!(runs[0].source_bytes, None);
        assert_eq!(runs[0].source_ranges, ranges);
    }
}

#[test]
fn mapped_chunks_preserve_normalization_across_boundaries() {
    let prep = CleanOptions::default();
    for tail in [
        "e\u{200b}\u{301}",
        "\u{1100}\u{1161}\u{11a8}",
        "a\u{315}\u{300}",
        "x\u{0344}",
        "\u{09c7}\u{09be}", // Bengali vowel-sign composition (both CCC 0).
        "x\u{0f73}",        // Tibetan vowel decomposes to leading non-starters.
    ] {
        for position in 4090..4100 {
            let raw = format!("{}{}{}", "A".repeat(position), tail, "京".repeat(1500));
            let (doc, map) = prep.prepare_mapped(&raw).unwrap();
            assert_eq!(doc, prep.prepare(&raw));
            assert!(map.runs.len() <= 6, "unstable run count at {position}");
        }
    }
}

#[test]
fn whitespace_matches_legacy_newline_collapse() {
    let alphabet = [' ', '\t', '\r', '\n', '\u{a0}', 'A'];
    for mut n in 0..alphabet.len().pow(4) {
        let text: String = (0..4)
            .map(|_| {
                let ch = alphabet[n % alphabet.len()];
                n /= alphabet.len();
                ch
            })
            .collect();
        let mut expected = String::new();
        let mut was_space = false;
        for ch in textprep::unicode::normalize_newlines(&text).chars() {
            if !ch.is_whitespace() {
                expected.push(ch);
            } else if !was_space {
                expected.push(if ch == '\n' { '\n' } else { ' ' });
            } else if ch == '\n' && !expected.ends_with('\n') {
                expected.push('\n');
            }
            was_space = ch.is_whitespace();
        }
        assert_eq!(
            CleanOptions {
                normalize_unicode: false,
                clean_whitespace: true
            }
            .prepare(&text),
            expected.trim(),
            "{text:?}"
        );
    }
}

#[test]
fn whitespace_sources_include_contributing_runs_and_exclude_trimmed_edges() {
    let input = "\tA\u{a0}\t\r\n\r\n B  C\r ";
    let output = "A \nB C";
    assert_scalar_source(input, output, 0..1, 1..2, SourceKind::Direct);
    assert_scalar_source(input, output, 1..2, 2..5, SourceKind::Transformed);
    assert_scalar_source(input, output, 2..3, 5..10, SourceKind::Transformed);
    assert_scalar_source(input, output, 4..5, 11..13, SourceKind::Transformed);
}

#[test]
fn whitespace_collapse_excludes_removed_zero_width_bytes() {
    let (doc, map) = CleanOptions::default()
        .prepare_mapped("A \u{200b} B")
        .unwrap();
    assert_eq!(doc, "A B");
    let runs = map.project_bytes(&doc, 1..2).unwrap();
    assert_eq!(runs[0].kind, SourceKind::Transformed);
    assert_eq!(runs[0].source_ranges, vec![1..2, 5..6]);
}
