#![cfg(feature = "graphemes")]

use textprep::{ByteOffset, CharOffset, GraphemeMap, GraphemeOffset};

const CASES: &[&str] = &[
    "e\u{301}",                   // combining mark
    "\u{1f469}\u{200d}\u{1f4bb}", // ZWJ emoji
    "\u{1f1fa}\u{1f1f8}",         // regional-indicator flag
    "\u{915}\u{94d}\u{937}",      // Indic conjunct
    "\r\n",                       // Unicode grapheme boundary rule GB3
    "a e\u{301} \u{1f469}\u{200d}\u{1f4bb} \u{1f1fa}\u{1f1f8} \u{915}\u{94d}\u{937}\r\nz",
];

#[test]
fn boundary_coordinates_roundtrip_for_unicode_sequences() {
    for text in CASES {
        let map = GraphemeMap::new(text);

        for index in 0..=map.grapheme_count() {
            let grapheme = GraphemeOffset::new(index);
            let byte = map.grapheme_to_byte(grapheme).unwrap();
            let character = map.grapheme_to_char(grapheme).unwrap();

            assert_eq!(map.byte_to_grapheme(byte), Some(grapheme), "{text:?}");
            assert_eq!(map.char_to_grapheme(character), Some(grapheme), "{text:?}");
        }
    }
}

#[test]
fn adjacent_grapheme_slices_reconstruct_source_exactly() {
    for text in CASES {
        let map = GraphemeMap::new(text);
        let reconstructed: String = (0..map.grapheme_count())
            .map(|index| {
                map.slice(GraphemeOffset::new(index), GraphemeOffset::new(index + 1))
                    .unwrap()
            })
            .collect();

        assert_eq!(reconstructed, *text);
        assert_eq!(
            map.slice(
                GraphemeOffset::new(0),
                GraphemeOffset::new(map.grapheme_count())
            ),
            Some(*text)
        );
    }
}

#[test]
fn offsets_inside_graphemes_are_rejected() {
    let text = "e\u{301}\u{1f469}\u{200d}\u{1f4bb}\r\n";
    let map = GraphemeMap::new(text);

    assert_eq!(map.byte_to_grapheme(ByteOffset::new(1)), None);
    assert_eq!(map.char_to_grapheme(CharOffset::new(1)), None);
    assert_eq!(map.char_to_grapheme(CharOffset::new(3)), None);
    assert_eq!(map.char_to_grapheme(CharOffset::new(4)), None);
    assert_eq!(map.char_to_grapheme(CharOffset::new(6)), None);
}

#[test]
fn empty_text_has_one_terminal_boundary() {
    let map = GraphemeMap::new("");

    assert_eq!(map.grapheme_count(), 0);
    assert_eq!(
        map.grapheme_to_byte(GraphemeOffset::new(0)),
        Some(ByteOffset::new(0))
    );
    assert_eq!(
        map.grapheme_to_char(GraphemeOffset::new(0)),
        Some(CharOffset::new(0))
    );
    assert_eq!(
        map.slice(GraphemeOffset::new(0), GraphemeOffset::new(0)),
        Some("")
    );
}

#[test]
fn invalid_ranges_and_out_of_bounds_offsets_are_rejected() {
    let map = GraphemeMap::new("ab");

    assert_eq!(
        map.slice(GraphemeOffset::new(2), GraphemeOffset::new(1)),
        None
    );
    assert_eq!(map.grapheme_to_byte(GraphemeOffset::new(3)), None);
    assert_eq!(map.byte_to_grapheme(ByteOffset::new(3)), None);
    assert_eq!(map.char_to_grapheme(CharOffset::new(3)), None);
}
