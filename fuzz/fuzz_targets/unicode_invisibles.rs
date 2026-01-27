#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let s = String::from_utf8_lossy(data);

    // Zero-width detection ↔ stripping consistency.
    let zw_hits = textprep::unicode::zero_width_with_offsets(&s);
    for (i, c) in &zw_hits {
        debug_assert_eq!(s.chars().nth(*i), Some(*c));
    }
    let no_zw = textprep::unicode::remove_zero_width(&s);
    debug_assert!(!textprep::unicode::contains_zero_width(&no_zw));
    debug_assert!(textprep::unicode::zero_width_with_offsets(&no_zw).is_empty());
    debug_assert_eq!(no_zw.chars().count() + zw_hits.len(), s.chars().count());

    let mut no_zw_into = String::new();
    textprep::unicode::remove_zero_width_into(&s, &mut no_zw_into);
    debug_assert_eq!(no_zw_into, no_zw);

    // Bidi controls detection ↔ stripping consistency.
    let bidi_hits = textprep::unicode::bidi_controls_with_offsets(&s);
    for (i, c) in &bidi_hits {
        debug_assert_eq!(s.chars().nth(*i), Some(*c));
    }
    let no_bidi = textprep::unicode::remove_bidi_controls(&s);
    debug_assert!(!textprep::unicode::contains_bidi_controls(&no_bidi));
    debug_assert!(textprep::unicode::bidi_controls_with_offsets(&no_bidi).is_empty());
    debug_assert_eq!(no_bidi.chars().count() + bidi_hits.len(), s.chars().count());

    let mut no_bidi_into = String::new();
    textprep::unicode::remove_bidi_controls_into(&s, &mut no_bidi_into);
    debug_assert_eq!(no_bidi_into, no_bidi);
});

