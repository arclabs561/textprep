#![no_main]

use libfuzzer_sys::fuzz_target;

fn slice_by_char_range(text: &str, start: usize, end: usize) -> String {
    text.chars().skip(start).take(end.saturating_sub(start)).collect()
}

fuzz_target!(|data: &[u8]| {
    let s = String::from_utf8_lossy(data);
    let tokens = textprep::tokenize::tokenize_with_offsets(&s);
    let char_count = s.chars().count();

    for t in &tokens {
        debug_assert!(t.start <= t.end);
        debug_assert!(t.end <= char_count);
        debug_assert!(!t.text.is_empty());
        debug_assert!(t.text.chars().all(|c| !c.is_whitespace()));
        debug_assert_eq!(slice_by_char_range(&s, t.start, t.end), t.text);
    }

    for w in tokens.windows(2) {
        debug_assert!(w[0].end <= w[1].start);
    }
});

