# Changelog

All notable changes to this crate are documented here.
Format: [Keep a Changelog](https://keepachangelog.com).

## Source history (unversioned)

These source changes are listed without assigning them to a published release.

### Fixed

- Repeated registration of an exact keyword updates its replacement without accumulating duplicate matcher patterns. Case-distinct registrations, first-registration order and match offsets are preserved.
  ([source change](https://github.com/arclabs561/textprep/commit/d0f449f2a31b83b0981f963955dc9eb4342045ed))

## [Unreleased]

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
