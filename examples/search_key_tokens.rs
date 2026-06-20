//! Build normalized token keys while preserving offsets into the original text.
//!
//! This is the safer shape for entity lookup: normalize each token for
//! matching, but report spans against the original text.
//!
//! Run:
//! `cargo run --release --example search_key_tokens`

use std::collections::BTreeMap;
use textprep::{scrub_with, tokenize_with_offsets, ScrubConfig};

fn main() {
    let text = "Café\u{200b} Müller filed from M\u{202e}unich. The CAFE team tagged Mueller too.";

    let aliases = BTreeMap::from([
        ("cafe".to_string(), "cafe"),
        ("muller".to_string(), "muller"),
        ("mueller".to_string(), "muller"),
        ("munich".to_string(), "munich"),
    ]);

    let cfg = ScrubConfig::search_key_strict_invisibles();
    let tokens = tokenize_with_offsets(text);

    println!("source: {text:?}\n");
    println!("matches:");

    let mut found = Vec::new();
    for token in &tokens {
        let key = scrub_with(&token.text, &cfg);
        if let Some(canonical) = aliases.get(&key) {
            println!(
                "  {:<7} key={:<7} span={:>2}..{:<2} original={:?}",
                canonical, key, token.start, token.end, token.text
            );
            found.push((*canonical, token.start, token.end));
        }
    }

    assert_eq!(found.len(), 5);
    assert!(found.iter().any(|(name, _, _)| *name == "munich"));
    assert!(found.iter().any(|(_, start, end)| text
        .chars()
        .skip(*start)
        .take(end - start)
        .collect::<String>()
        .contains('\u{202e}')));
}
