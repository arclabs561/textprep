//! Fuzzy-match query strings against tokens extracted from text.
//!
//! Demonstrates combining `textprep` tokenization + normalization with
//! `gramdex` trigram similarity for approximate entity name matching.
//!
//! Use case: finding variant spellings or inflections of entity names
//! (e.g. "react" vs "reactjs", "javascript" vs "java-script").

use gramdex::{trigram_jaccard, GramDex};
use textprep::tokenize;

/// Minimum Jaccard similarity to consider a match.
const THRESHOLD: f32 = 0.25;

fn main() {
    let text = "\
        The team migrated from ReactJS to Svelte. Meanwhile, Java-Script \
        linters were replaced by Biome. TypeScript adoption grew, and \
        the PostgreSQL database was swapped for Postgres-compatible CockroachDB. \
        Some engineers still prefer Javascript over TypeScript.";

    // 1. Tokenize and normalize: extract words, then scrub each one
    //    (lowercase + NFC + strip diacritics) so matching is case-insensitive.
    let tokens = tokenize::tokenize_with_offsets(text);
    let normalized: Vec<String> = tokens.iter().map(|t| textprep::scrub(&t.text)).collect();

    println!("Tokens ({}):", normalized.len());
    for (tok, norm) in tokens.iter().zip(&normalized) {
        println!(
            "  [{:>3}..{:<3}] {:<20} -> {}",
            tok.start, tok.end, tok.text, norm
        );
    }
    println!();

    // 2. Build a trigram index over the normalized tokens.
    let mut index = GramDex::new();
    for (id, norm) in normalized.iter().enumerate() {
        index.add_document_trigrams(id as u32, norm);
    }

    // 3. Fuzzy-match queries against the token set.
    let queries = [
        "react",
        "javascript",
        "typescript",
        "postgres",
        "svelt",
        "biome",
    ];

    for query in &queries {
        let q = textprep::scrub(query);
        println!("Query: {:?} (scrubbed: {:?})", query, q);

        // Candidate generation: get token ids sharing at least one trigram.
        let candidates = index.candidates_union_trigrams(&q);

        // Verification: rank candidates by trigram Jaccard similarity.
        let mut matches: Vec<(usize, f32)> = candidates
            .iter()
            .map(|&id| {
                let sim = trigram_jaccard(&q, &normalized[id as usize]);
                (id as usize, sim)
            })
            .filter(|&(_, sim)| sim >= THRESHOLD)
            .collect();

        matches.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        if matches.is_empty() {
            println!("  (no matches above threshold {THRESHOLD})\n");
            continue;
        }

        for (id, sim) in &matches {
            let tok = &tokens[*id];
            println!(
                "  {:.3}  {:?} (chars {}..{})",
                sim, tok.text, tok.start, tok.end
            );
        }
        println!();
    }
}
