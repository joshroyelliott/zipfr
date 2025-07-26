use std::fs::File;
use std::io::{BufRead, BufReader};
use anyhow::Result;

pub struct TextParser;

impl TextParser {
    pub fn parse_file(file_path: &str) -> Result<Vec<String>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut words = Vec::new();

        for line in reader.lines() {
            let line = line?;
            let line_words = Self::extract_words(&line);
            words.extend(line_words);
        }

        Ok(words)
    }

    fn extract_words(text: &str) -> Vec<String> {
        text.split_whitespace()
            .map(|word| {
                word.chars()
                    .filter(|c| c.is_alphabetic())
                    .collect::<String>()
                    .to_lowercase()
            })
            .filter(|word| !word.is_empty())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_words() {
        let text = "Hello, world! This is a test.";
        let words = TextParser::extract_words(text);
        assert_eq!(words, vec!["hello", "world", "this", "is", "a", "test"]);
    }

    #[test]
    fn test_extract_words_with_numbers() {
        let text = "Test123 with numbers456 and symbols!@#";
        let words = TextParser::extract_words(text);
        assert_eq!(words, vec!["test", "with", "numbers", "and", "symbols"]);
    }
}