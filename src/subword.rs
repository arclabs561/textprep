//! Subword tokenization traits (minimal).
//!
//! This is intentionally small: it exists to support simple embedding/projection
//! adapters in the workspace without pulling in heavy tokenizer stacks.
//!
//! If/when we standardize on `tokenizers` (HF) or other model tokenizers, keep
//! those as separate, opt-in layers.

use std::collections::HashMap;

/// Trait for subword tokenizers.
///
/// Returns a sequence of token IDs for input text.
pub trait SubwordTokenizer: Send + Sync {
    fn tokenize(&self, text: &str) -> Vec<u32>;
}

/// A simple vocabulary lookup tokenizer.
///
/// Note: this is not true BPE. It is a thin adapter for toy projections/tests.
pub struct BpeTokenizer {
    vocab: HashMap<String, u32>,
}

impl BpeTokenizer {
    /// Create a tokenizer from a token→id vocabulary map.
    pub fn from_vocab(vocab: HashMap<String, u32>) -> Self {
        Self { vocab }
    }
}

impl SubwordTokenizer for BpeTokenizer {
    fn tokenize(&self, text: &str) -> Vec<u32> {
        text.split_whitespace()
            .filter_map(|word| self.vocab.get(word).copied())
            .collect()
    }
}

