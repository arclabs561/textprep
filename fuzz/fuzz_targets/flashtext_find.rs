#![no_main]

use libfuzzer_sys::fuzz_target;

fn slice_by_char_range(text: &str, start: usize, end: usize) -> String {
    text.chars().skip(start).take(end.saturating_sub(start)).collect()
}

fuzz_target!(|data: &[u8]| {
    let s = String::from_utf8_lossy(data);

    let mut ft = textprep::FlashText::new();
    ft.add_keyword("François", "francois");
    ft.add_keyword("Müller", "muller");
    ft.add_keyword("北京", "beijing");
    ft.add_keyword("hello", "hello");
    ft.add_keyword("🎉", "party");

    let matches = ft.find(&s);
    let mut out = Vec::new();
    ft.find_into(&s, &mut out);
    debug_assert_eq!(out, matches);

    let char_count = s.chars().count();
    let mut last_end = 0usize;
    for m in &matches {
        debug_assert!(m.start <= m.end);
        debug_assert!(m.end <= char_count);
        debug_assert!(last_end <= m.start);

        let extracted = slice_by_char_range(&s, m.start, m.end);
        debug_assert_eq!(extracted.to_ascii_lowercase(), m.keyword.to_ascii_lowercase());

        last_end = m.end;
    }
});

