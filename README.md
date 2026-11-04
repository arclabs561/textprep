# textprep

Text normalization, tokenization, and keyword matching primitives

## Install

```toml
[dependencies]
textprep = "0.2.1"
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

## Languages

`Language` bundles a language's stopwords (NLTK's lists, 33 languages) and,
with the `stem` feature, its Snowball stemmer (18 languages). Stopword
lookup lowercases the token and treats `’` as `'`:

```rust
use textprep::language::Language;

let de = Language::from_code("de").unwrap().with_extra_stopwords(["vgl"]);
assert!(de.is_stopword("Und"));
assert!(de.is_stopword("vgl"));
assert!(Language::english().is_stopword("didn\u{2019}t"));
```

With the `detect` feature, `LanguagePolicy::DetectPerSentence` assigns each
sentence its own language. A sentence with fewer than 10 letters takes the
language of the whole text. `cargo run --release --example
sentence_languages --features detect,stem` prints each sentence's language,
where it came from, and the stems of its non-stopwords:

```text
de (Sentence): Die Abgeordneten diskutierten mehrere Stunden über die Kosten.
    abgeordnet diskutiert mehr stund kost
en (Sentence): The committee didn’t publish its final report.
    committe publish final report
de (Document): Ja.
    ja
```

## Choices and limits

- Scrubbing is lossy: it can change case, normalization form, whitespace, and
  diacritics. Keep the source text when you need display text or source spans.
- Token and keyword-match offsets count Unicode scalar values, not UTF-8 bytes
  or user-perceived grapheme clusters.
- `ScrubConfig::search_key()` preserves zero-width characters. Use
  `search_key_strict_invisibles()` only when removing joiners and other common
  invisible characters is appropriate for the input.
- Stopword lists are NLTK's function-word lists. Lists for larger
  vocabularies (stopwords-iso, SMART) are not included.
- Language detection is per text or per sentence, not per phrase. On short
  sentences the built-in whatlang detector (`detect`) is often wrong: 69%
  correct at 1 to 3 words and 81% at 4 to 5 in the `tools/lang-eval`
  measurement, against 94% and 97% for lingua. lingua is not a dependency
  (its models add about 143 MB); plug it or any other detector in through
  `language::Detector` and `sentence_languages_with`.

Optional features:

- `casefold`: full Unicode case folding for search keys.
- `graphemes`: typed conversion between byte,
  scalar, and grapheme boundaries.
- `serde`: serialization for tokens, keyword matches, and scrub configuration.
- `stem`: Snowball stemmers (rust-stemmers).
- `detect`: built-in language detection with whatlang (about 0.25 MB).

See [the examples](examples/README.md) for keyword matching, fuzzy matching,
normalized token keys, and source-offset handling.

## License

MIT OR Apache-2.0
