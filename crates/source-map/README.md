# textprep-source-map

Attach byte provenance to UTF-8 output from a renderer, normalizer, or extractor.
Each run describes how an output interval relates to the exact original edition.
For example, decoding an HTML entity points to the entire reference. Unchanged
text supports exact byte clipping. Removed markup can leave disjoint original
ranges contributing to a single output character.

```rust
use textprep_source_map::{SourceKind, SourceMap, SourceRun};

let decoded = SourceMap::new(
    "&amp;!",
    "&!",
    vec![
        SourceRun::new(0..1, Some(0..5), SourceKind::Transformed),
        SourceRun::new(1..2, Some(5..6), SourceKind::Direct),
    ],
)?;
let fragment = &decoded.project_chars("&!", 0..1)?[0];
assert_eq!(fragment.source_bytes, Some(0..5));
# Ok::<(), String>(())
```

Compose a downstream map with its upstream map using
`downstream.compose(&upstream)`. Composition checks the length and SHA-256 digest
of the intermediate edition. A partial query inside an indivisible transformation
returns approximate evidence. When composing several stages, the grouping can
affect precision. An early split of an indivisible run records uncertainty in
each resulting fragment, even if a later transform reunites them. Use `SourceMap::unmapped` when the output has no
recoverable coordinates.

Enable the optional `serde` feature for interchange. After deserializing a map,
call `validate_analyzed` with the exact analyzed text, or `verify` when both
editions are available. On an immutable, validated map, `overlaps` uses binary
search to project a sparse query without rehashing the artifact. Its input is
a half-open analyzed byte range with validated UTF-8 boundaries.
