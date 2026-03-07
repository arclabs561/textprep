# textprep

[![crates.io](https://img.shields.io/crates/v/textprep.svg)](https://crates.io/crates/textprep)
[![Documentation](https://docs.rs/textprep/badge.svg)](https://docs.rs/textprep)
[![CI](https://github.com/arclabs561/textprep/actions/workflows/ci.yml/badge.svg)](https://github.com/arclabs561/textprep/actions/workflows/ci.yml)

Text preprocessing primitives: normalization, tokenization, n-grams, string similarity, stopwords, and fast keyword matching.

```toml
[dependencies]
textprep = "0.1.1"
```

## Normalization

`scrub` normalizes text to a canonical form for indexing and comparison: NFC normalization, case folding, and diacritics stripping.

```rust
use textprep::scrub;

assert_eq!(scrub("Muller"), "muller");
assert_eq!(scrub("Cafe\u{0301}"), "cafe");  // combining accent
```

For search pipelines that need stricter normalization (NFKC, bidi control removal, zero-width stripping), use `ScrubConfig`:

```rust
use textprep::{scrub_with, ScrubConfig};

let cfg = ScrubConfig::search_key();
let key = scrub_with("  Hello\u{200B}World  ", &cfg);
// NFKC + lowercase + collapsed whitespace
```

## Tokenization

Split text into words or sentences, with character offsets:

```rust
use textprep::tokenize::{words, sentences, tokenize_with_offsets};

let w = words("Hello, world!");
assert_eq!(w, vec!["Hello", "world"]);

let s = sentences("First sentence. Second one!");
assert_eq!(s.len(), 2);

// With character offsets (not byte offsets)
let tokens = tokenize_with_offsets("Hello world");
assert_eq!(tokens[0].text, "Hello");
assert_eq!(tokens[0].start, 0);
assert_eq!(tokens[0].end, 5);
```

## Fast keyword matching

`FlashText` provides linear-time multi-pattern keyword search (Aho-Corasick based):

```rust
use textprep::FlashText;

let mut ft = FlashText::new();
ft.add_keyword("Big Apple", "New York");
ft.add_keyword("NYC", "New York");

let matches = ft.find("I live in the Big Apple, also known as NYC.");
assert_eq!(matches[0].value, "New York");
// matches[0].start/end are character offsets
```

## N-grams

Character-level and word-level n-gram generation:

```rust
use textprep::ngram::{char_ngrams, word_ngrams};

let cg = char_ngrams("hello", 3);
// ["hel", "ell", "llo"]

let words = vec!["the", "quick", "brown", "fox"];
let wg = word_ngrams(&words, 2);
// ["the quick", "quick brown", "brown fox"]
```

## String similarity

Jaccard similarity at word and character-ngram levels:

```rust
use textprep::similarity::{word_jaccard, trigram_jaccard};

let sim = word_jaccard("hello world", "world hello");
assert!((sim - 1.0).abs() < f64::EPSILON); // same words

let sim = trigram_jaccard("kitten", "sitting");
assert!(sim > 0.0 && sim < 1.0);
```

## Stopwords

Built-in English stopword list, plus loadable lists for other languages:

```rust
use textprep::stopwords::is_english_stopword;

assert!(is_english_stopword("the"));
assert!(!is_english_stopword("quantum"));
```

## Unicode utilities

Direct access to normalization forms and text cleaning:

```rust
use textprep::unicode::{nfc, nfkc};
use textprep::fold::{fold, strip_diacritics};
use textprep::html::decode_entities;

let normalized = nfkc("ﬁ");       // "fi" (compatibility decomposition)
let lowered = fold("Straße");      // "straße"
let plain = strip_diacritics("cafe\u{0301}"); // "cafe"
let decoded = decode_entities("&amp; &lt;"); // "& <"
```

## Feature flags

| Feature | What it adds |
|---------|-------------|
| `casefold` | Full Unicode NFKC_Casefold (e.g. sharp-s to "ss") |
| `serde` | Serialize/deserialize for `Token`, `KeywordMatch`, `ScrubConfig` |

## License

MIT OR Apache-2.0
