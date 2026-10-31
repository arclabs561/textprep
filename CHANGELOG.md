# Changelog

All notable changes to this crate are documented here.
Format: [Keep a Changelog](https://keepachangelog.com).

## Source history (unversioned)

These source changes are listed without assigning them to a published release.

### Fixed

- Repeated registration of an exact keyword updates its replacement without accumulating duplicate matcher patterns. Case-distinct registrations, first-registration order and match offsets are preserved.
  ([source change](https://github.com/arclabs561/textprep/commit/d0f449f2a31b83b0981f963955dc9eb4342045ed))

## [Unreleased]

### Changed

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

### Added

- Optional `graphemes` API for typed conversion between UTF-8 byte, Unicode
  scalar, and extended grapheme cluster boundaries.
- `search_key_tokens` example showing normalized token keys with original character offsets.

## [0.1.6] - 2026-04-28

### Deprecated

- `trigram_jaccard`: the canonical implementation now lives in the `gramdex`
  crate. The function remains for backward compatibility but new code should
  prefer the gramdex version.

## [0.1.5] - 2026-04-16

Earlier releases predate this changelog.
