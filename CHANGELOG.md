# Changelog

All notable changes to this crate are documented here.
Format: [Keep a Changelog](https://keepachangelog.com).

## [Unreleased]

## [0.2.1] - 2026-11-03

### Fixed

- `decode_entities` follows the HTML5 rules for numeric character
  references: `&#128;`..`&#159;` map through the windows-1252 table
  (`&#146;` is `’`), `&#0;`, surrogates and values above U+10FFFF decode
  to U+FFFD (previously out-of-range references such as `&#xFFFFFF;` were
  left as text), and references with a sign (`&#+65;`) stay literal.
- `char_ngrams` and `word_ngrams` return an empty `Vec` for `n == 0`
  instead of panicking, as `token_ngrams` already did.
- `strip_diacritics` also strips Hebrew niqqud and cantillation, Arabic
  harakat and Quranic marks, and the Combining Diacritical Marks Extended
  block (U+1AB0..U+1AFF). Combining marks of other scripts, such as kana
  voicing marks and Indic vowel signs, are still kept.
- Case-insensitive `FlashText` matching now lowercases non-ASCII
  characters too, so `Müller` matches `MÜLLER`. Offsets still refer to the
  original text.

## [0.2.0] - 2026-11-03

### Changed

- Behaviour change: `FlashText` keywords now match only at word boundaries
  (a word character is alphanumeric or `_`), so `cat` no longer matches inside
  `concatenate`, `cats` or `under_cat`.
- Behaviour change: `strip_diacritics` now returns NFC text instead of NFD, so
  characters without a stripped mark (for example Hangul syllables) stay
  composed.
- Behaviour change: the `casefold` feature now folds with `caseless` 0.2.2
  (Unicode 16.0 `CaseFolding.txt`) instead of `unicode-casefold` 0.2.0
  (Unicode 9.0). `fold_nfkc_casefold`, `fold_nfkc_casefold_into` and
  `ScrubConfig::search_key` now fold capitals added after Unicode 9, such as
  Georgian Mtavruli (U+1C90 folds to U+10D0), which were previously left
  unchanged. Search keys for such text differ from earlier releases, so
  indexes built with the old keys should be rebuilt. The feature name is
  unchanged.

### Fixed

- `FlashText::find` no longer lets a match rejected at a word boundary consume
  its span, so a whole-word keyword that starts inside it is still found:
  with keywords `food` and `cat food`, `"bobcat food"` now yields `food`.
- Repeated registration of an exact keyword updates its replacement without
  accumulating duplicate matcher patterns. Case-distinct registrations,
  first-registration order and match offsets are preserved.
- An empty keyword matches once at each character boundary and no longer
  disrupts other matches on multi-byte text.
- `decode_entities` no longer panics when the 14-byte entity search window
  ends inside a multi-byte character (for example `"&ééééééé;"`).

### Added

- Optional `graphemes` API for typed conversion between UTF-8 byte, Unicode
  scalar, and extended grapheme cluster boundaries.
- `words`, `sentences`, `tokenize_with_offsets` and
  `tokenize_refs_with_offsets` are re-exported at the crate root.
- `search_key_tokens` example showing normalized token keys with original character offsets.

## [0.1.6] - 2026-04-28

### Deprecated

- `trigram_jaccard`: the canonical implementation now lives in the `gramdex`
  crate. The function remains for backward compatibility but new code should
  prefer the gramdex version.

## [0.1.5] - 2026-04-16

Earlier releases predate this changelog.
