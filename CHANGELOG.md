# Changelog

All notable changes to this crate are documented here.
Format: [Keep a Changelog](https://keepachangelog.com).

## [Unreleased]

## [0.3.1] - 2026-11-04

### Added

- `stopwords::smart_english`: the SMART English stopword list (570 words,
  from the RCV1 paper's appendix 11), for splitting untagged text into
  candidate phrases. NLTK's lists stay the default everywhere else.

## [0.3.0] - 2026-11-04

### Changed

- Declare `rust-version = "1.85"`. The declared 1.75 did not hold: a fresh
  resolve picks `unicode-segmentation` 1.13, which requires Rust 1.85.
- Behaviour change: `stopwords::get` returns NLTK's list for each of 33
  languages (from the `stop-words` crate, `nltk` feature only), where it
  returned a set only for English. It also accepts ISO 639-3 codes and
  English names (`"deu"`, `"german"`) and ignores a region subtag
  (`"en-US"`).
- Behaviour change: English stopwords are NLTK's current 198-word list.
  `get("en")` and `is_english_stopword` used the 127-word list; the 71
  added words are contraction forms and fragments ("ain", "aren",
  "aren't", "couldn", "couldn't", "d", "didn", "didn't", "doesn",
  "doesn't", "don't", "hadn", "hadn't", "hasn", "hasn't", "haven",
  "haven't", "he'd", "he'll", "he's", "i'd", "i'll", "i'm", "i've", "isn",
  "isn't", "it'd", "it'll", "it's", "ll", "m", "ma", "mightn", "mightn't",
  "mustn", "mustn't", "needn", "needn't", "o", "re", "shan", "shan't",
  "she'd", "she'll", "she's", "should've", "shouldn", "shouldn't",
  "that'll", "they'd", "they'll", "they're", "they've", "ve", "wasn",
  "wasn't", "we'd", "we'll", "we're", "we've", "weren", "weren't", "won",
  "won't", "wouldn", "wouldn't", "y", "you'd", "you'll", "you're",
  "you've"). No word was removed.
- Behaviour change: `is_english_stopword` treats U+2018 and U+2019 as `'`,
  so `don’t` is a stopword.

### Deprecated

- `stopwords::ENGLISH`, the 127-word list. Its contents are unchanged, but
  textprep no longer uses it; use `stopwords::list("en")` or
  `Language::english()`.

### Added

- `language::Language`: a language code with its stopword set and, with
  the new `stem` feature, its Snowball stemmer (rust-stemmers, 18
  languages): `from_code`, `english`, `with_stopwords`,
  `with_extra_stopwords`, `is_stopword` (lowercases and treats U+2018 and
  U+2019 as `'`), `stem` and `has_stemmer`.
- `language::LanguagePolicy` (`Fixed`, `Detect`, `DetectPerSentence {
  fallback }`), the `language::Detector` trait (also implemented for
  closures) and `language::sentence_languages_with`, which assigns a
  language to each sentence from `segment::sentence_byte_ranges`. Text
  with fewer than 10 letters (`MIN_DETECT_CHARS`) is never sent to the
  detector.
- `detect` feature: `WhatlangDetector` (whatlang 0.18), `language::detect`
  and `language::sentence_languages`. On Tatoeba sentences in 17
  languages lingua 1.8 was more accurate on short sentences (94% against
  69% correct at 1 to 3 words), but it is not a dependency: it adds about
  143 MB to a release binary and needs Rust 1.88. Plug it in through
  `Detector`. `tools/lang-eval` (not packaged) holds the measurement.
- `stopwords::LANGUAGES`, `stopwords::canonical_code`, `stopwords::list`
  and `stopwords::fold_apostrophes`.
- Example `sentence_languages`.
- `segment::sentence_byte_ranges` and `segment::word_byte_ranges`: UAX #29
  boundaries adapted to prose, as byte ranges into the input. A single line
  break does not end a sentence, a period after a known English
  abbreviation does not either unless a sentence opener follows, and
  hyphenated compounds stay one word. The abbreviation and opener lists
  are vendored from sentencex 0.1.33 (MIT).

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
