# textprep

Text preprocessing primitives: normalization, tokenization, and fast keyword matching.

## Contract

- **Invariants (must never change)**:
  - **Normalization**: `scrub` defaults to **NFC** normalization + **lower case**.
  - **Offsets**: `Token` and `KeywordMatch` always return **byte offsets** (usize) into the *original* input string (not the normalized string), enabling zero-copy slicing of the source.
  - **No panic on Unicode**: All functions must handle invalid UTF-8 gracefully (usually by `String` type constraints or `char::REPLACEMENT_CHARACTER`) without panicking.

- **Support / Dependencies**:
  - **Unicode**: Relies on `unicode-normalization` and `unicode-segmentation`.
  - **Keyword Matching**: Uses Aho-Corasick (`FlashText` equivalent) for linear-time multi-pattern search.

- **Exports**:
  - `scrub(text)`: standard "search key" normalization.
  - `FlashText`: Aho-Corasick wrapper for keyword replacement/extraction.
  - `SubwordTokenizer`: BPE-like splitting.

## Usage

```rust
use textprep::{scrub, FlashText};

// Normalization
let raw = "Héllö World!";
let key = scrub(raw); // "hello world!"

// Fast Keyword Matching
let mut ft = FlashText::new();
ft.add_keyword("Big Apple", "New York");
ft.add_keyword("SF", "San Francisco");

let text = "I live in the Big Apple.";
let found = ft.extract_keywords(text);
assert_eq!(found[0].clean_name, "New York");
```
