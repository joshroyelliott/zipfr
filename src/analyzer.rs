use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::time::Duration;
use serde::{Deserialize, Serialize};
use anyhow::{Result, Context};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Tag {
    pub name: String,
    pub color: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone)]
pub struct WordCount {
    pub word: String,
    pub count: usize,
    pub rank: usize,
    pub tags: HashSet<Tag>,
}

#[derive(Debug, Clone)]
pub struct Dataset {
    pub name: String,
    pub word_counts: Vec<WordCount>,
    pub total_words: usize,
    pub unique_words: usize,
    pub parse_duration: Duration,
    pub analyze_duration: Duration,
}

#[derive(Debug, Deserialize)]
struct TagConfig {
    tags: HashMap<String, TagDefinition>,
}

#[derive(Debug, Deserialize)]
struct TagDefinition {
    name: String,
    color: Option<String>,
    description: Option<String>,
    words: Vec<String>,
}

#[derive(Clone)]
pub struct TagMatcher {
    word_to_tags: HashMap<String, HashSet<Tag>>,
    available_tags: Vec<Tag>,
}

impl TagMatcher {
    pub fn new() -> Self {
        Self {
            word_to_tags: HashMap::new(),
            available_tags: Vec::new(),
        }
    }

    pub fn from_config<P: AsRef<Path>>(config_path: P) -> Result<Self> {
        let config_content = std::fs::read_to_string(config_path)
            .context("Failed to read tags configuration file")?;
        
        let config: TagConfig = toml::from_str(&config_content)
            .context("Failed to parse tags configuration")?;
        
        let mut word_to_tags: HashMap<String, HashSet<Tag>> = HashMap::new();
        let mut available_tags = Vec::new();
        
        for (_tag_id, tag_def) in config.tags {
            let tag = Tag {
                name: tag_def.name,
                color: tag_def.color,
                description: tag_def.description,
            };
            
            available_tags.push(tag.clone());
            
            for word in tag_def.words {
                word_to_tags
                    .entry(word.to_lowercase())
                    .or_insert_with(HashSet::new)
                    .insert(tag.clone());
            }
        }
        
        Ok(Self {
            word_to_tags,
            available_tags,
        })
    }

    pub fn get_tags(&self, word: &str) -> HashSet<Tag> {
        self.word_to_tags
            .get(&word.to_lowercase())
            .cloned()
            .unwrap_or_default()
    }

    pub fn available_tags(&self) -> &[Tag] {
        &self.available_tags
    }

    pub fn get_tag_by_name(&self, name: &str) -> Option<&Tag> {
        self.available_tags.iter().find(|tag| tag.name == name)
    }
}

pub struct WordAnalyzer {
    word_counts: HashMap<String, usize>,
    tag_matcher: Option<TagMatcher>,
}

impl WordAnalyzer {
    pub fn new() -> Self {
        Self {
            word_counts: HashMap::new(),
            tag_matcher: None,
        }
    }

    pub fn with_tags(tag_matcher: TagMatcher) -> Self {
        Self {
            word_counts: HashMap::new(),
            tag_matcher: Some(tag_matcher),
        }
    }

    pub fn analyze(&mut self, words: Vec<String>) -> Vec<WordCount> {
        self.word_counts.clear();
        
        for word in words {
            *self.word_counts.entry(word).or_insert(0) += 1;
        }

        self.get_ranked_words()
    }

    fn get_ranked_words(&self) -> Vec<WordCount> {
        let mut word_counts: Vec<(String, usize)> = self.word_counts
            .iter()
            .map(|(word, count)| (word.clone(), *count))
            .collect();

        word_counts.sort_by(|a, b| b.1.cmp(&a.1));

        word_counts
            .into_iter()
            .enumerate()
            .map(|(index, (word, count))| {
                let tags = if let Some(ref tag_matcher) = self.tag_matcher {
                    tag_matcher.get_tags(&word)
                } else {
                    HashSet::new()
                };

                WordCount {
                    word,
                    count,
                    rank: index + 1,
                    tags,
                }
            })
            .collect()
    }

    pub fn tag_matcher(&self) -> Option<&TagMatcher> {
        self.tag_matcher.as_ref()
    }

    pub fn total_words(&self) -> usize {
        self.word_counts.values().sum()
    }

    pub fn unique_words(&self) -> usize {
        self.word_counts.len()
    }
}

impl Default for WordAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_word_analysis() {
        let mut analyzer = WordAnalyzer::new();
        let words = vec![
            "the".to_string(),
            "quick".to_string(),
            "brown".to_string(),
            "fox".to_string(),
            "jumps".to_string(),
            "over".to_string(),
            "the".to_string(),
            "lazy".to_string(),
            "dog".to_string(),
            "the".to_string(),
        ];

        let results = analyzer.analyze(words);
        
        assert_eq!(results[0].word, "the");
        assert_eq!(results[0].count, 3);
        assert_eq!(results[0].rank, 1);
        assert!(results[0].tags.is_empty()); // No tags when no TagMatcher
        
        assert_eq!(analyzer.total_words(), 10);
        assert_eq!(analyzer.unique_words(), 8);
    }
}