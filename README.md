# textprep

Text normalization, tokenization, and keyword matching primitives

## Install

```toml
[dependencies]
textprep = "0.1.6"
```

## Usage

Normalize tokens for matching while retaining offsets into the source text:

```rust
use textprep::{scrub_with, tokenize_with_offsets, ScrubConfig};

let text = "Cafés in Zürich";
let config = ScrubConfig::search_key();
let tokens = tokenize_with_offsets(text);

assert_eq!(scrub_with(&tokens[0].text, &config), "cafes");
assert_eq!((tokens[0].start, tokens[0].end), (0, 5));
assert_eq!(scrub_with(&tokens[2].text, &config), "zurich");
assert_eq!((tokens[2].start, tokens[2].end), (9, 15));
```

## Choices and limits

- Scrubbing is lossy: it can change case, normalization form, whitespace, and
  diacritics. Keep the source text when you need display text or source spans.
- Token and keyword-match offsets count Unicode scalar values, not UTF-8 bytes
  or user-perceived grapheme clusters.
- `ScrubConfig::search_key()` preserves zero-width characters. Use
  `search_key_strict_invisibles()` only when removing joiners and other common
  invisible characters is appropriate for the input.
- The built-in stopword list is English-only.

Optional features:

- `casefold`: full Unicode case folding for search keys.
- `graphemes`: typed conversion between byte, scalar, and grapheme boundaries.
- `serde`: serialization for tokens, keyword matches, and scrub configuration.

See [the examples](examples/README.md) for keyword matching, fuzzy matching,
normalized token keys, and source-offset handling.

## License

MIT OR Apache-2.0
