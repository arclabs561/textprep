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
});

