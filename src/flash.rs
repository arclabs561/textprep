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

    pub fn find(&mut self, text: &str) -> Vec<KeywordMatch> {
        self.ensure_built();
        let matcher = self.matcher.as_ref().unwrap();
        let mut matches = Vec::new();

        for mat in matcher.find_iter(text) {
            let pattern = &self.pattern_list[mat.pattern()];
            let value = self.keywords.get(pattern).cloned().unwrap_or_else(|| pattern.clone());
            
            let start = text[..mat.start()].chars().count();
            let len = text[mat.start()..mat.end()].chars().count();
            
            matches.push(KeywordMatch {
                keyword: pattern.clone(),
                value,
                start,
                end: start + len,
            });
        }

        matches
    }
}

impl Default for FlashText {
    fn default() -> Self {
        Self::new()
    }
}
