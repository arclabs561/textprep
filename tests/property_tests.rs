use proptest::prelude::*;

const ZWSP: char = '\u{200B}'; // ZERO WIDTH SPACE
const ZWNJ: char = '\u{200C}'; // ZERO WIDTH NON-JOINER
const ZWJ: char = '\u{200D}'; // ZERO WIDTH JOINER
const WJ: char = '\u{2060}'; // WORD JOINER
const BOM: char = '\u{FEFF}'; // ZERO WIDTH NO-BREAK SPACE (BOM)

fn any_reasonable_string() -> impl Strategy<Value = String> {
    // Keep it bounded to avoid slow quadratic behavior in tests.
    // Includes full Unicode scalar range (including control chars).
    proptest::collection::vec(any::<char>(), 0..200).prop_map(|cs| cs.into_iter().collect())
}

fn slice_by_char_range(text: &str, start: usize, end: usize) -> String {
    text.chars().skip(start).take(end.saturating_sub(start)).collect()
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
        prop_assert!(w1 >= 0.0 && w1 <= 1.0);
        prop_assert!((w1 - w2).abs() < 1e-12);

        let c1 = textprep::similarity::char_ngram_jaccard(&a, &b, n);
        let c2 = textprep::similarity::char_ngram_jaccard(&b, &a, n);
        prop_assert!(c1 >= 0.0 && c1 <= 1.0);
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
        prop_assert!(s >= 0.0 && s <= 1.0);
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
}

