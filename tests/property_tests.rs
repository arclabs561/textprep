use proptest::prelude::*;

const ZWSP: char = '\u{200B}'; // ZERO WIDTH SPACE
const ZWNJ: char = '\u{200C}'; // ZERO WIDTH NON-JOINER
const ZWJ: char = '\u{200D}'; // ZERO WIDTH JOINER
const WJ: char = '\u{2060}'; // WORD JOINER
const BOM: char = '\u{FEFF}'; // ZERO WIDTH NO-BREAK SPACE (BOM)

const LRE: char = '\u{202A}'; // LEFT-TO-RIGHT EMBEDDING
const RLE: char = '\u{202B}'; // RIGHT-TO-LEFT EMBEDDING
const PDF: char = '\u{202C}'; // POP DIRECTIONAL FORMATTING
const LRO: char = '\u{202D}'; // LEFT-TO-RIGHT OVERRIDE
const RLO: char = '\u{202E}'; // RIGHT-TO-LEFT OVERRIDE
const LRI: char = '\u{2066}'; // LEFT-TO-RIGHT ISOLATE
const RLI: char = '\u{2067}'; // RIGHT-TO-LEFT ISOLATE
const FSI: char = '\u{2068}'; // FIRST-STRONG ISOLATE
const PDI: char = '\u{2069}'; // POP DIRECTIONAL ISOLATE
const LRM: char = '\u{200E}'; // LEFT-TO-RIGHT MARK
const RLM: char = '\u{200F}'; // RIGHT-TO-LEFT MARK
const ALM: char = '\u{061C}'; // ARABIC LETTER MARK

fn any_reasonable_string() -> impl Strategy<Value = String> {
    // Keep it bounded to avoid slow quadratic behavior in tests.
    // Includes full Unicode scalar range (including control chars).
    proptest::collection::vec(any::<char>(), 0..200).prop_map(|cs| cs.into_iter().collect())
}

fn slice_by_char_range(text: &str, start: usize, end: usize) -> String {
    text.chars()
        .skip(start)
        .take(end.saturating_sub(start))
        .collect()
}

fn assert_flash_matches_sane(
    text: &str,
    matches: &[textprep::KeywordMatch],
) -> Result<(), TestCaseError> {
    let char_count = text.chars().count();

    let mut last_end = 0usize;
    for m in matches {
        prop_assert!(m.start <= m.end);
        prop_assert!(m.end <= char_count);
        prop_assert!(last_end <= m.start);

        let extracted = slice_by_char_range(text, m.start, m.end);
        // `FlashText` uses ASCII case-insensitive matching (configurable internally),
        // so the matched substring may differ in ASCII casing from the pattern.
        prop_assert_eq!(
            extracted.to_ascii_lowercase(),
            m.keyword.to_ascii_lowercase()
        );

        last_end = m.end;
    }

    Ok(())
}

proptest! {
    #[test]
    fn collapse_whitespace_is_trimmed_and_single_spaced(s in any_reasonable_string()) {
        let out = textprep::unicode::collapse_whitespace(&s);

        prop_assert!(!out.starts_with(' '));
        prop_assert!(!out.ends_with(' '));
        prop_assert!(!out.contains("  "));

        // Only ASCII spaces may remain as whitespace.
        prop_assert!(out.chars().all(|c| !c.is_whitespace() || c == ' '));
        prop_assert!(!out.contains('\t'));
        prop_assert!(!out.contains('\n'));
        prop_assert!(!out.contains('\r'));
    }

    #[test]
    fn remove_zero_width_is_idempotent_and_removes_targets(s in any_reasonable_string()) {
        let out1 = textprep::unicode::remove_zero_width(&s);
        let out2 = textprep::unicode::remove_zero_width(&out1);
        prop_assert_eq!(out1.as_str(), out2.as_str());

        // Avoid `\u{...}` escapes in `prop_assert!` (they contain `{}` and break formatting).
        prop_assert!(!out1.contains(ZWSP));
        prop_assert!(!out1.contains(ZWNJ));
        prop_assert!(!out1.contains(ZWJ));
        prop_assert!(!out1.contains(WJ));
        prop_assert!(!out1.contains(BOM));
    }

    #[test]
    fn zero_width_offsets_roundtrip(s in any_reasonable_string()) {
        let hits = textprep::unicode::zero_width_with_offsets(&s);
        prop_assert_eq!(textprep::unicode::contains_zero_width(&s), !hits.is_empty());
        for (i, c) in &hits {
            prop_assert_eq!(s.chars().nth(*i), Some(*c));
            prop_assert!(matches!(*c, ZWSP | ZWNJ | ZWJ | WJ | BOM));
        }

        let out = textprep::unicode::remove_zero_width(&s);
        prop_assert!(!textprep::unicode::contains_zero_width(&out));
        prop_assert!(textprep::unicode::zero_width_with_offsets(&out).is_empty());

        // remove_* is a pure deletion of these codepoints.
        prop_assert_eq!(out.chars().count() + hits.len(), s.chars().count());
    }

    #[test]
    fn bidi_offsets_roundtrip(s in any_reasonable_string()) {
        let hits = textprep::unicode::bidi_controls_with_offsets(&s);
        prop_assert_eq!(textprep::unicode::contains_bidi_controls(&s), !hits.is_empty());
        for (i, c) in &hits {
            prop_assert_eq!(s.chars().nth(*i), Some(*c));
            prop_assert!(matches!(
                *c,
                LRE | RLE | PDF | LRO | RLO | LRI | RLI | FSI | PDI | LRM | RLM | ALM
            ));
        }

        let out = textprep::unicode::remove_bidi_controls(&s);
        prop_assert!(!textprep::unicode::contains_bidi_controls(&out));
        prop_assert!(textprep::unicode::bidi_controls_with_offsets(&out).is_empty());

        prop_assert_eq!(out.chars().count() + hits.len(), s.chars().count());
    }

    #[test]
    fn normalize_newlines_removes_cr(s in any_reasonable_string()) {
        let out = textprep::unicode::normalize_newlines(&s);
        prop_assert!(!out.contains('\r'));
    }

    #[test]
    fn tokenize_with_offsets_produces_valid_spans(s in any_reasonable_string()) {
        let tokens = textprep::tokenize::tokenize_with_offsets(&s);
        let char_count = s.chars().count();

        for t in &tokens {
            prop_assert!(t.start <= t.end);
            prop_assert!(t.end <= char_count);
            prop_assert!(!t.text.is_empty());
            prop_assert!(t.text.chars().all(|c| !c.is_whitespace()));

            let extracted = slice_by_char_range(&s, t.start, t.end);
            prop_assert_eq!(extracted.as_str(), t.text.as_str());
        }

        for w in tokens.windows(2) {
            prop_assert!(w[0].start <= w[1].start);
            prop_assert!(w[0].end <= w[1].end);
            prop_assert!(w[0].end <= w[1].start);
        }
    }

    #[test]
    fn similarity_metrics_are_symmetric_and_bounded(
        a in any_reasonable_string(),
        b in any_reasonable_string(),
        n in 1usize..6usize,
    ) {
        let w1 = textprep::similarity::word_jaccard(&a, &b);
        let w2 = textprep::similarity::word_jaccard(&b, &a);
        prop_assert!((0.0..=1.0).contains(&w1));
        prop_assert!((w1 - w2).abs() < 1e-12);

        let c1 = textprep::similarity::char_ngram_jaccard(&a, &b, n);
        let c2 = textprep::similarity::char_ngram_jaccard(&b, &a, n);
        prop_assert!((0.0..=1.0).contains(&c1));
        prop_assert!((c1 - c2).abs() < 1e-12);
    }

    #[test]
    fn weighted_jaccard_is_bounded_when_weights_sum_to_one(
        a in any_reasonable_string(),
        b in any_reasonable_string(),
        n in 1usize..6usize,
        w in 0.0f64..=1.0f64,
    ) {
        let s = textprep::similarity::weighted_word_char_ngram_jaccard(&a, &b, n, w, 1.0 - w);
        prop_assert!((0.0..=1.0).contains(&s));
    }

    #[test]
    fn token_ngrams_match_windows(
        words in proptest::collection::vec(
            proptest::collection::vec(any::<char>(), 0..16).prop_map(|cs| cs.into_iter().collect::<String>()),
            0..20
        ),
        n in 1usize..6usize,
    ) {
        let refs: Vec<&str> = words.iter().map(|s| s.as_str()).collect();
        let out = textprep::ngram::token_ngrams(&refs, n);

        if refs.len() < n {
            prop_assert!(out.is_empty());
        } else {
            prop_assert_eq!(out.len(), refs.len() - n + 1);
            for (i, gram) in out.iter().enumerate() {
                prop_assert_eq!(gram.len(), n);
                for j in 0..n {
                    prop_assert_eq!(gram[j], refs[i + j]);
                }
            }
        }
    }

    #[test]
    fn flashtext_find_is_well_formed_and_matches_find_into(s in any_reasonable_string()) {
        let mut ft = textprep::FlashText::new();
        ft.add_keyword("François", "francois");
        ft.add_keyword("Müller", "muller");
        ft.add_keyword("北京", "beijing");
        ft.add_keyword("hello", "hello");

        let matches = ft.find(&s);
        assert_flash_matches_sane(&s, &matches)?;

        let mut out = Vec::new();
        ft.find_into(&s, &mut out);
        // Compare by reference so we can keep using `out` below.
        prop_assert_eq!(&out, &matches);
        assert_flash_matches_sane(&s, &out)?;
    }

    #[test]
    fn flashtext_finds_embedded_keyword(
        prefix in any_reasonable_string(),
        suffix in any_reasonable_string(),
    ) {
        // Keep the constructed string modest so tests stay fast.
        let prefix: String = prefix.chars().take(80).collect();
        let suffix: String = suffix.chars().take(80).collect();

        let text = format!("{prefix} HeLLo {suffix}");

        let mut ft = textprep::FlashText::new();
        ft.add_keyword("hello", "hello");

        let matches = ft.find(&text);
        assert_flash_matches_sane(&text, &matches)?;
        prop_assert!(matches.iter().any(|m| m.keyword == "hello"));
    }

    #[test]
    fn scrub_search_key_is_idempotent(s in any_reasonable_string()) {
        let cfg = textprep::ScrubConfig::search_key();
        let out1 = textprep::scrub_with(&s, &cfg);
        let out2 = textprep::scrub_with(&out1, &cfg);
        prop_assert_eq!(out1, out2);
    }

    #[test]
    fn scrub_search_key_is_trimmed_and_single_spaced(s in any_reasonable_string()) {
        let cfg = textprep::ScrubConfig::search_key();
        let out = textprep::scrub_with(&s, &cfg);
        prop_assert!(!out.starts_with(' '));
        prop_assert!(!out.ends_with(' '));
        prop_assert!(!out.contains("  "));
        prop_assert!(out.chars().all(|c| !c.is_whitespace() || c == ' '));
    }

    #[test]
    fn scrub_search_key_strict_is_idempotent(s in any_reasonable_string()) {
        let cfg = textprep::ScrubConfig::search_key_strict_invisibles();
        let out1 = textprep::scrub_with(&s, &cfg);
        let out2 = textprep::scrub_with(&out1, &cfg);
        prop_assert_eq!(out1, out2);
    }
}
