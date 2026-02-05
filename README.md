# textprep

Text preprocessing primitives: normalization, tokenization, and fast keyword matching.

## Contract

- **Invariants (must never change)**:
  - **Normalization**: `scrub` defaults to **NFC** normalization + **lower case**.
  - **Offsets**: `Token` and `KeywordMatch` return **character offsets** (usize) into the *original* input string.
    - **Note**: these are **not byte offsets**. For slicing, convert via `.chars()` (or build a byte-index map if you need fast repeated slicing).
  - **No panic on Unicode**: All functions must handle invalid UTF-8 gracefully.

- **Support / Dependencies**:
  - **Unicode**: Relies on `unicode-normalization` and `unicode-segmentation`.
  - **Keyword Matching**: Uses Aho-Corasick (`FlashText` equivalent) for linear-time multi-pattern search.

## Usage

```rust
use textprep::{scrub, FlashText};

// Normalization
let raw = "Héllö World!";
let key = scrub(raw); // "hello world!"

// Fast Keyword Matching
let mut ft = FlashText::new();
ft.add_keyword("Big Apple", "New York");

let text = "I live in the Big Apple.";
let found = ft.find(text);
// found[0].start/end are CHAR offsets (not byte offsets)
assert_eq!(found[0].value, "New York");
```
