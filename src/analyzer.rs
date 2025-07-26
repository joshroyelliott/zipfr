use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct WordCount {
    pub word: String,
    pub count: usize,
    pub rank: usize,
}

pub struct WordAnalyzer {
    word_counts: HashMap<String, usize>,
}

impl WordAnalyzer {
    pub fn new() -> Self {
        Self {
            word_counts: HashMap::new(),
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
            .map(|(index, (word, count))| WordCount {
                word,
                count,
                rank: index + 1,
            })
            .collect()
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
        
        assert_eq!(analyzer.total_words(), 10);
        assert_eq!(analyzer.unique_words(), 8);
    }
}