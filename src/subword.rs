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
    /// Tokenize text into a sequence of token IDs.
    fn tokenize(&self, text: &str) -> Vec<u32>;
}

/// A simple vocabulary lookup tokenizer.
///
/// Note: this is not true BPE. It is a thin adapter for toy projections/tests.
/// Prefer [`VocabTokenizer`] for new code.
#[deprecated(
    since = "0.1.5",
    note = "misleading name: this is a vocabulary lookup, not BPE. Use VocabTokenizer instead."
)]
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BpeTokenizer {
    vocab: HashMap<String, u32>,
}

#[allow(deprecated)]
impl BpeTokenizer {
    /// Create a tokenizer from a token→id vocabulary map.
    pub fn from_vocab(vocab: HashMap<String, u32>) -> Self {
        Self { vocab }
    }
}

#[allow(deprecated)]
impl SubwordTokenizer for BpeTokenizer {
    fn tokenize(&self, text: &str) -> Vec<u32> {
        text.split_whitespace()
            .filter_map(|word| self.vocab.get(word).copied())
            .collect()
    }
}

/// A simple vocabulary lookup tokenizer.
///
/// This is a rename of the former `BpeTokenizer` to better reflect that it
/// performs whitespace-split vocabulary lookup, not byte-pair encoding.
#[allow(deprecated)]
pub type VocabTokenizer = BpeTokenizer;
