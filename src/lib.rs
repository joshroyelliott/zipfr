pub mod parser;
pub mod analyzer;
pub mod cli;
pub mod tui;

pub use analyzer::{WordCount, WordAnalyzer};
pub use parser::TextParser;
pub use cli::Args;