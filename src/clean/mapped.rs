use super::{clean_whitespace, CleanOptions, SourceKind, SourceMap, SourceRun};
use std::ops::Range;
use unicode_normalization::char::{canonical_combining_class, decompose_canonical};

pub(super) fn prepare(options: &CleanOptions, text: &str) -> Result<(String, SourceMap), String> {
    let (normalized, upstream) = if options.normalize_unicode {
        normalize_mapped(text)?
    } else {
        (text.to_owned(), SourceMap::identity(text))
    };
    if !options.clean_whitespace {
        return Ok((normalized, upstream));
    }
    let mut runs = Vec::new();
    let cleaned = clean_whitespace(&normalized, |out, src, direct| {
        let kind = if direct {
            SourceKind::Direct
        } else {
            SourceKind::Transformed
        };
        push_run(&mut runs, SourceRun::new(out, Some(src), kind));
    });
    let map = if cleaned == normalized {
        upstream
    } else {
        SourceMap::new(&normalized, &cleaned, runs)?.compose(&upstream)?
    };
    Ok((cleaned, map))
}

// Coalesce while building, so a document with one change does not accumulate
// a separate mapping record for every unchanged scalar, word or space.
fn push_run(runs: &mut Vec<SourceRun>, run: SourceRun) {
    if let Some(prior) = runs.last_mut() {
        if let (Some(prev), Some(next)) = (&mut prior.source_bytes, &run.source_bytes) {
            if prior.kind == SourceKind::Direct
                && run.kind == SourceKind::Direct
                && prev.end == next.start
            {
                prior.analyzed_bytes.end = run.analyzed_bytes.end;
                prev.end = next.end;
                return;
            }
        }
    }
    runs.push(run);
}

// A starter which cannot compose backwards begins an independent NFC segment.
// Inspect its first canonical decomposition scalar so this also handles Hangul.
fn nfc_boundary_before(ch: char) -> bool {
    use unicode_normalization::{is_nfc_quick, IsNormalized};
    if ch.is_ascii() {
        return true;
    }
    if crate::unicode::contains_zero_width(ch.encode_utf8(&mut [0; 4])) {
        return false;
    }
    let mut first = None;
    decompose_canonical(ch, |part| {
        first.get_or_insert(part);
    });
    first.is_some_and(|c| {
        canonical_combining_class(c) == 0 && is_nfc_quick(std::iter::once(c)) == IsNormalized::Yes
    })
}

fn normalize_mapped(original: &str) -> Result<(String, SourceMap), String> {
    let mut normalized = String::with_capacity(original.len());
    let mut runs = Vec::new();
    let mut start = 0;
    while start < original.len() {
        // Bound temporary alignment buffers to a small chunk plus one unbroken
        // combining sequence. A chunk may only end before an NFC boundary.
        let mut target = start.saturating_add(4096).min(original.len());
        while !original.is_char_boundary(target) {
            target += 1;
        }
        let end = original[target..]
            .char_indices()
            .find(|(_, ch)| nfc_boundary_before(*ch))
            .map_or(original.len(), |(i, _)| target + i);
        let raw = &original[start..end];
        let normalized_chunk = crate::unicode::nfc(&crate::unicode::remove_zero_width(raw));
        let dest = normalized.len();
        if raw == normalized_chunk {
            push_run(
                &mut runs,
                SourceRun::new(dest..dest + raw.len(), Some(start..end), SourceKind::Direct),
            );
        } else {
            align_chunk(raw, &normalized_chunk, start, dest, &mut runs)?;
        }
        normalized.push_str(&normalized_chunk);
        start = end;
    }
    let map = SourceMap::new(original, &normalized, runs)?;
    Ok((normalized, map))
}

/// Tag each canonically decomposed scalar with its originating byte interval,
/// then stably order non-starters by their canonical combining class (UAX #15).
fn tagged_nfd(text: &str, skip_zero_width: bool) -> Vec<(char, Range<usize>)> {
    let mut parts = Vec::new();
    for (start, ch) in text.char_indices() {
        let range = start..start + ch.len_utf8();
        if skip_zero_width && crate::unicode::contains_zero_width(&text[range.clone()]) {
            continue;
        }
        decompose_canonical(ch, |part| parts.push((part, range.clone())));
    }
    for marks in parts.split_mut(|(c, _)| canonical_combining_class(*c) == 0) {
        marks.sort_by_key(|(c, _)| canonical_combining_class(*c));
    }
    parts
}

fn align_chunk(
    original: &str,
    normalized: &str,
    original_offset: usize,
    analyzed_offset: usize,
    runs: &mut Vec<SourceRun>,
) -> Result<(), String> {
    // Both strings have identical NFD after filtering the configured zero-width
    // characters. Pairing tagged NFD streams attributes composition, expansion
    // and canonical reordering to the exact input scalars, including Hangul.
    let source = tagged_nfd(original, true);
    let dest = tagged_nfd(normalized, false);
    if source.len() != dest.len() {
        return Err("Canonical decompositions differ in length".into());
    }
    let mut origins = Vec::with_capacity(source.len());
    for ((from, src), (to, out)) in source.into_iter().zip(dest) {
        if from != to {
            return Err("Canonical decompositions differ after normalization".into());
        }
        origins.push((out, src));
    }
    origins.sort_by_key(|(out, src)| (out.start, src.start));
    for group in origins.chunk_by(|a, b| a.0 == b.0) {
        let (out, src) = &group[0];
        let dest = out.start + analyzed_offset..out.end + analyzed_offset;
        let single_source = group.iter().all(|(_, range)| range == src);
        let run = if single_source {
            let kind = if normalized[out.clone()] == original[src.clone()] {
                SourceKind::Direct
            } else {
                SourceKind::Transformed
            };
            let src = src.start + original_offset..src.end + original_offset;
            SourceRun::new(dest, Some(src), kind)
        } else {
            let ranges = group
                .iter()
                .map(|(_, r)| r.start + original_offset..r.end + original_offset)
                .collect();
            SourceRun::from_ranges(dest, ranges, SourceKind::Transformed)
        };
        push_run(runs, run);
    }
    Ok(())
}
