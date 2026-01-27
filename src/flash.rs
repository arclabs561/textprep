//! Fast keyword matching using Aho-Corasick.

use aho_corasick::{AhoCorasick, MatchKind};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KeywordMatch {
    pub keyword: String,
    pub value: String,
    pub start: usize,
    pub end: usize,
}

pub struct FlashText {
    keywords: HashMap<String, String>,
    matcher: Option<AhoCorasick>,
    pattern_list: Vec<String>,
    case_insensitive: bool,
}

impl FlashText {
    pub fn new() -> Self {
        Self {
            keywords: HashMap::new(),
            matcher: None,
            pattern_list: Vec::new(),
            case_insensitive: true,
        }
    }

    pub fn add_keyword(&mut self, keyword: impl Into<String>, value: impl Into<String>) {
        let kw = keyword.into();
        let val = value.into();
        self.keywords.insert(kw.clone(), val);
        self.pattern_list.push(kw);
        self.matcher = None;
    }

    fn ensure_built(&mut self) {
        if self.matcher.is_none() {
            let ac = AhoCorasick::builder()
                .match_kind(MatchKind::LeftmostLongest)
                .ascii_case_insensitive(self.case_insensitive)
                .build(&self.pattern_list)
                .expect("failed to build Aho-Corasick matcher");
            self.matcher = Some(ac);
        }
    }

    /// Find all keyword matches, writing results into `out`.
    ///
    /// `out` is cleared first. This is useful when you want to reuse allocations in hot loops.
    pub fn find_into(&mut self, text: &str, out: &mut Vec<KeywordMatch>) {
        out.clear();
        self.ensure_built();
        let matcher = self.matcher.as_ref().unwrap();

        // `aho-corasick` yields byte offsets. Convert to char offsets in a single pass
        // by incrementally advancing from the last match boundary.
        let mut last_byte = 0usize;
        let mut last_char = 0usize;

        for mat in matcher.find_iter(text) {
            let pattern = &self.pattern_list[mat.pattern()];
            let value = self
                .keywords
                .get(pattern)
                .cloned()
                .unwrap_or_else(|| pattern.clone());

            // Advance char counter from last match end → current match start.
            if mat.start() >= last_byte {
                last_char += text[last_byte..mat.start()].chars().count();
            } else {
                // Defensive: should not happen for `find_iter` (monotonic), but keep correctness.
                last_char = text[..mat.start()].chars().count();
            }
            let start = last_char;
            let len = text[mat.start()..mat.end()].chars().count();

            out.push(KeywordMatch {
                keyword: pattern.clone(),
                value,
                start,
                end: start + len,
            });

            // Update last boundary to end of match.
            last_byte = mat.end();
            last_char = start + len;
        }
    }

    pub fn find(&mut self, text: &str) -> Vec<KeywordMatch> {
        let mut matches = Vec::new();
        self.find_into(text, &mut matches);
        matches
    }
}

impl Default for FlashText {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_char_offsets_are_correct_for_unicode() {
        let mut ft = FlashText::new();
        ft.add_keyword("東京", "tokyo");
        ft.add_keyword("Müller", "muller");

        // Mixed ASCII + multibyte.
        let text = "a 東京 b Müller c";
        let matches = ft.find(text);

        // Char offsets in `text`:
        // 0 a
        // 1 ' '
        // 2 東
        // 3 京
        // 4 ' '
        // 5 b
        // 6 ' '
        // 7 M
        // 8 ü
        // 9 l
        // 10 l
        // 11 e
        // 12 r
        // 13 ' '
        // 14 c
        assert_eq!(
            matches,
            vec![
                KeywordMatch {
                    keyword: "東京".to_string(),
                    value: "tokyo".to_string(),
                    start: 2,
                    end: 4
                },
                KeywordMatch {
                    keyword: "Müller".to_string(),
                    value: "muller".to_string(),
                    start: 7,
                    end: 13
                }
            ]
        );
    }
}
