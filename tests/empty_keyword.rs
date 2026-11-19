use textprep::FlashText;

#[test]
fn empty_keywords_match_only_character_boundaries() {
    let mut matcher = FlashText::new();
    matcher.add_keyword("", "empty");

    for text in ["", "abc", "é東京🚀"] {
        let matches = matcher.find(text);
        assert_eq!(
            matches.iter().map(|m| (m.start, m.end)).collect::<Vec<_>>(),
            (0..=text.chars().count())
                .map(|i| (i, i))
                .collect::<Vec<_>>()
        );
        assert!(matches
            .iter()
            .all(|m| m.keyword.is_empty() && m.value == "empty"));
    }
}

#[test]
fn empty_keyword_does_not_disrupt_real_matches_or_reused_output() {
    let mut matcher = FlashText::new();
    matcher.add_keyword("cat", "feline");
    let text = "é cat 東京";
    let expected = matcher.find(text);
    assert_eq!(expected.len(), 1);
    assert_eq!((expected[0].start, expected[0].end), (2, 5));

    matcher.add_keyword("", "empty");
    let mut matches = expected.clone();
    matcher.find_into(text, &mut matches);
    assert_eq!(
        matches
            .into_iter()
            .filter(|m| !m.keyword.is_empty())
            .collect::<Vec<_>>(),
        expected
    );
}

#[test]
fn incomplete_keyword_keeps_empty_match_at_its_start() {
    let mut matcher = FlashText::new();
    matcher.add_keyword("", "empty");
    matcher.add_keyword("界🦀", "pair");

    let matches = matcher.find("界!界🦀");
    let spans: Vec<_> = matches
        .iter()
        .map(|m| (m.value.as_str(), m.start, m.end))
        .collect();
    assert_eq!(spans, [("empty", 0, 0), ("empty", 1, 1), ("pair", 2, 4)]);
}
