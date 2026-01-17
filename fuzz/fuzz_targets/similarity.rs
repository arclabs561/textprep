#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Split fuzz input into two strings.
    let mid = data.len() / 2;
    let a = String::from_utf8_lossy(&data[..mid]);
    let b = String::from_utf8_lossy(&data[mid..]);

    let w = textprep::similarity::word_jaccard(&a, &b);
    debug_assert!(w >= 0.0 && w <= 1.0);
    debug_assert!((w - textprep::similarity::word_jaccard(&b, &a)).abs() < 1e-12);

    for n in 1..=6 {
        let c = textprep::similarity::char_ngram_jaccard(&a, &b, n);
        debug_assert!(c >= 0.0 && c <= 1.0);
        debug_assert!(
            (c - textprep::similarity::char_ngram_jaccard(&b, &a, n)).abs() < 1e-12
        );
    }
});

