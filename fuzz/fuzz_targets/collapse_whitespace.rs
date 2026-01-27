#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let s = String::from_utf8_lossy(data);
    let out = textprep::unicode::collapse_whitespace(&s);

    // Basic invariants (should always hold, even for weird inputs).
    if !out.is_empty() {
        debug_assert!(!out.starts_with(' '));
        debug_assert!(!out.ends_with(' '));
        debug_assert!(!out.contains("  "));
    }
    debug_assert!(out.chars().all(|c| !c.is_whitespace() || c == ' '));

    // Equivalence to *_into variant.
    let mut out_into = String::new();
    textprep::unicode::collapse_whitespace_into(&s, &mut out_into);
    debug_assert_eq!(out_into, out);

    // Idempotence.
    debug_assert_eq!(textprep::unicode::collapse_whitespace(&out), out);

    // Newline normalization should remove CR, and the *_into variant should match.
    let nl = textprep::unicode::normalize_newlines(&s);
    debug_assert!(!nl.contains('\r'));
    let mut nl_into = String::new();
    textprep::unicode::normalize_newlines_into(&s, &mut nl_into);
    debug_assert_eq!(nl_into, nl);
});

