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
use textprep::tokenize::tokenize_with_offsets;
use textprep::{scrub_with, ScrubConfig};

let text = "Cafés in Zürich";
let config = ScrubConfig::search_key();
let tokens = tokenize_with_offsets(text);

assert_eq!(scrub_with(&tokens[0].text, &config), "cafes");
assert_eq!((tokens[0].start, tokens[0].end), (0, 5));
assert_eq!(scrub_with(&tokens[2].text, &config), "zurich");
assert_eq!((tokens[2].start, tokens[2].end), (9, 15));
```

`cargo run --release --example search_key_tokens` applies the same idea to
noisier text, with `ScrubConfig::search_key_strict_invisibles()`:

```text
source: "Café\u{200b} Müller filed from M\u{202e}unich. The CAFE team tagged Mueller too."

matches:
  cafe    key=cafe    span= 0..4  original="Café"
  muller  key=muller  span= 6..12 original="Müller"
  munich  key=munich  span=24..31 original="M\u{202e}unich"
  cafe    key=cafe    span=37..41 original="CAFE"
  muller  key=mueller span=54..61 original="Mueller"
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
- `graphemes` (unreleased, not in 0.1.6): typed conversion between byte,
  scalar, and grapheme boundaries.
- `serde`: serialization for tokens, keyword matches, and scrub configuration.

See [the examples](examples/README.md) for keyword matching, fuzzy matching,
normalized token keys, and source-offset handling.

## License

MIT OR Apache-2.0
