//! Explicit coordinate conversion at extended grapheme cluster boundaries.
//!
//! Existing `textprep` token and match offsets count Unicode scalar values.
//! This module does not change that contract. It provides distinct offset types
//! and a precomputed map for callers that also need user-perceived character
//! boundaries.

use unicode_segmentation::UnicodeSegmentation;

/// A UTF-8 byte offset into a string.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ByteOffset(usize);

impl ByteOffset {
    /// Construct a byte offset.
    pub const fn new(value: usize) -> Self {
        Self(value)
    }

    /// Return the underlying offset.
    pub const fn get(self) -> usize {
        self.0
    }
}

/// A Unicode scalar-value (Rust `char`) offset into a string.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CharOffset(usize);

impl CharOffset {
    /// Construct a character offset.
    pub const fn new(value: usize) -> Self {
        Self(value)
    }

    /// Return the underlying offset.
    pub const fn get(self) -> usize {
        self.0
    }
}

/// An extended grapheme cluster offset into a string.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GraphemeOffset(usize);

impl GraphemeOffset {
    /// Construct a grapheme offset.
    pub const fn new(value: usize) -> Self {
        Self(value)
    }

    /// Return the underlying offset.
    pub const fn get(self) -> usize {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Boundary {
    byte: ByteOffset,
    character: CharOffset,
}

/// A reusable map between byte, character, and extended grapheme boundaries.
///
/// Conversions return `None` when an offset is out of range or falls inside a
/// UTF-8 code point or extended grapheme cluster. The end of the string is a
/// valid boundary in every coordinate system.
#[derive(Debug, Clone)]
pub struct GraphemeMap<'a> {
    text: &'a str,
    boundaries: Vec<Boundary>,
}

impl<'a> GraphemeMap<'a> {
    /// Build a boundary map for `text`.
    pub fn new(text: &'a str) -> Self {
        let mut boundaries = Vec::new();
        let mut character = 0;

        for (byte, grapheme) in text.grapheme_indices(true) {
            boundaries.push(Boundary {
                byte: ByteOffset::new(byte),
                character: CharOffset::new(character),
            });
            character += grapheme.chars().count();
        }
        boundaries.push(Boundary {
            byte: ByteOffset::new(text.len()),
            character: CharOffset::new(character),
        });

        Self { text, boundaries }
    }

    /// Return the source text.
    pub const fn text(&self) -> &'a str {
        self.text
    }

    /// Return the number of extended grapheme clusters.
    pub fn grapheme_count(&self) -> usize {
        self.boundaries.len() - 1
    }

    /// Convert a grapheme boundary to its UTF-8 byte offset.
    pub fn grapheme_to_byte(&self, offset: GraphemeOffset) -> Option<ByteOffset> {
        self.boundaries.get(offset.get()).map(|entry| entry.byte)
    }

    /// Convert a grapheme boundary to its character offset.
    pub fn grapheme_to_char(&self, offset: GraphemeOffset) -> Option<CharOffset> {
        self.boundaries
            .get(offset.get())
            .map(|entry| entry.character)
    }

    /// Convert a UTF-8 byte offset to a grapheme boundary.
    pub fn byte_to_grapheme(&self, offset: ByteOffset) -> Option<GraphemeOffset> {
        self.boundaries
            .binary_search_by_key(&offset, |entry| entry.byte)
            .ok()
            .map(GraphemeOffset::new)
    }

    /// Convert a character offset to a grapheme boundary.
    pub fn char_to_grapheme(&self, offset: CharOffset) -> Option<GraphemeOffset> {
        self.boundaries
            .binary_search_by_key(&offset, |entry| entry.character)
            .ok()
            .map(GraphemeOffset::new)
    }

    /// Borrow the text between two grapheme boundaries.
    pub fn slice(&self, start: GraphemeOffset, end: GraphemeOffset) -> Option<&'a str> {
        if start > end {
            return None;
        }
        let start = self.grapheme_to_byte(start)?.get();
        let end = self.grapheme_to_byte(end)?.get();
        self.text.get(start..end)
    }
}
