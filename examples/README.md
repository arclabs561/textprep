# textprep examples

Each example is runnable from the repo root. Output excerpts below are real,
captured from release runs.

## Which example should I run?

| I want to... | Example |
|---|---|
| Normalize token keys while preserving source offsets | `search_key_tokens` |
| Fuzzy-match query strings against extracted tokens | `fuzzy_token_match` |
| Tag dictionary entities in noisy web text | `gazetteer_tagging` |

## Normalized Keys And Offsets

### `search_key_tokens`: can matching use normalized keys without losing spans?

Tokenizes noisy Unicode text, normalizes each token with
`ScrubConfig::search_key_strict_invisibles`, and reports spans against the
original source text.

```bash
cargo run --release --example search_key_tokens
```

```text
source: "Café\u{200b} Müller filed from M\u{202e}unich. The CAFE team tagged Mueller too."

matches:
  cafe    key=cafe    span= 0..4  original="Café"
  muller  key=muller  span= 6..12 original="Müller"
  munich  key=munich  span=24..31 original="M\u{202e}unich"
  cafe    key=cafe    span=37..41 original="CAFE"
  muller  key=mueller span=54..61 original="Mueller"
```

The match key is normalized, but the reported offsets still point into the
original text.

## Fuzzy Token Lookup

### `fuzzy_token_match`: which extracted tokens are close to a query?

Combines `textprep` tokenization and normalization with `gramdex` trigram
similarity. Candidate generation uses shared trigrams; verification ranks by
Jaccard score.

```bash
cargo run --release --example fuzzy_token_match
```

```text
Tokens (35):
  [ 23..30 ] ReactJS              -> reactjs
  [ 34..40 ] Svelte               -> svelte
  [ 53..57 ] Java                 -> java
  [ 58..64 ] Script               -> script
  [ 90..95 ] Biome                -> biome
  [ 97..107] TypeScript           -> typescript
  [131..141] PostgreSQL           -> postgresql
  [167..175] Postgres             -> postgres
  [228..238] Javascript           -> javascript

Query: "postgres" (scrubbed: "postgres")
  1.000  "Postgres" (chars 167..175)
  0.750  "PostgreSQL" (chars 131..141)

Query: "svelt" (scrubbed: "svelt")
  0.750  "Svelte" (chars 34..40)
```

## Gazetteer Tagging

### `gazetteer_tagging`: can dictionary tags survive web-text cleanup?

Decodes HTML entities, runs `FlashText` over the cleaned text, and reports
character offsets with leftmost-longest overlap resolution.

```bash
cargo run --release --example gazetteer_tagging
```

```text
raw:     "We opened a caf&#xe9; in new york, near AT&amp;T, then flew to SAN FRANCISCO for York Fashion Week."
decoded: "We opened a café in new york, near AT&T, then flew to SAN FRANCISCO for York Fashion Week."

matches (5):
  CAFE  chars 12..16  surface="café"
  NYC   chars 20..28  surface="new york"
  ATT   chars 35..39  surface="AT&T"
  SF    chars 54..67  surface="SAN FRANCISCO"
  YORK  chars 72..76  surface="York"

doc "Direct from new york to san francisco."
    -> ["NYC", "SF"]
doc "AT&T sponsored the york stage."
    -> ["ATT", "YORK"]

ok
```

The `New York` entry wins over the overlapping `York` entry, while the
standalone `York Fashion Week` mention still tags as `YORK`.
