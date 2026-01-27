#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let s = String::from_utf8_lossy(data);

    // Search-key policy should always remove bidi controls (Trojan Source defense),
    // but should NOT remove ZWJ/ZWNJ by default.
    let key = textprep::ScrubConfig::search_key();
    let out_key = textprep::scrub_with(&s, &key);
    debug_assert!(!textprep::unicode::contains_bidi_controls(&out_key));

    // Strict policy also removes the "common zero-width" set.
    let strict = textprep::ScrubConfig::search_key_strict_invisibles();
    let out_strict = textprep::scrub_with(&s, &strict);
    debug_assert!(!textprep::unicode::contains_bidi_controls(&out_strict));
    debug_assert!(!textprep::unicode::contains_zero_width(&out_strict));

    // Both should be deterministic/idempotent.
    debug_assert_eq!(out_key, textprep::scrub_with(&out_key, &key));
    debug_assert_eq!(out_strict, textprep::scrub_with(&out_strict, &strict));
});

