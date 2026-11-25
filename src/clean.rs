//! Document cleaning with optional UTF-8 byte provenance.
use std::ops::Range;

#[cfg(feature = "source-map")]
mod mapped;
#[cfg(feature = "source-map")]
pub use textprep_source_map::{SourceKind, SourceMap, SourceRun};

/// Cleaning operations, in the order they are applied.
/// Both are enabled by default. Disable Unicode normalization when joiners are
/// significant in the source language or emoji sequences.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct CleanOptions {
    /// Remove the common zero-width characters listed by [`crate::unicode::remove_zero_width`],
    /// then normalize to NFC.
    pub normalize_unicode: bool,
    /// Trim edges and collapse interior whitespace. A line break is preserved as
    /// one LF, along with one space if horizontal whitespace precedes that break.
    pub clean_whitespace: bool,
}

impl Default for CleanOptions {
    fn default() -> Self {
        Self {
            normalize_unicode: true,
            clean_whitespace: true,
        }
    }
}

impl CleanOptions {
    /// Prepare a document as plain text.
    pub fn prepare(&self, text: &str) -> String {
        let normalized = if self.normalize_unicode {
            crate::unicode::nfc(&crate::unicode::remove_zero_width(text))
        } else {
            text.to_owned()
        };
        if self.clean_whitespace {
            clean_whitespace(&normalized, |_, _, _| {})
        } else {
            normalized
        }
    }

    /// Prepare the same text as [`Self::prepare`], with a map from analyzed UTF-8
    /// byte ranges to their original contributors. Reordered or disjoint input
    /// ranges retain their original positions.
    #[cfg(feature = "source-map")]
    pub fn prepare_mapped(&self, text: &str) -> Result<(String, SourceMap), String> {
        mapped::prepare(self, text)
    }
}

/// Emit the same interior whitespace as a newline-normalize, collapse, trim
/// pipeline. A horizontal prefix before the first line break survives as one space.
fn clean_whitespace(
    text: &str,
    mut record: impl FnMut(Range<usize>, Range<usize>, bool),
) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.char_indices().peekable();
    while let Some((start, ch)) = chars.next() {
        let whitespace = ch.is_whitespace();
        while chars
            .next_if(|(_, c)| c.is_whitespace() == whitespace)
            .is_some()
        {}
        let end = chars.peek().map_or(text.len(), |(offset, _)| *offset);
        if !whitespace {
            let pos = out.len();
            out.push_str(&text[start..end]);
            record(pos..out.len(), start..end, true);
        } else if start != 0 && end != text.len() {
            let newline = text[start..end]
                .find(['\r', '\n'])
                .map_or(end, |n| start + n);
            if start < newline {
                let pos = out.len();
                out.push(' ');
                record(pos..out.len(), start..newline, &text[start..newline] == " ");
            }
            if newline < end {
                let pos = out.len();
                out.push('\n');
                record(pos..out.len(), newline..end, &text[newline..end] == "\n");
            }
        }
    }
    out
}
