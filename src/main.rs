use clap::Parser;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::time::Instant;
use zipfr::{analyzer::{WordAnalyzer, TagMatcher, Dataset}, cli::Args, parser::TextParser, tui::App};

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let start_time = Instant::now();
    
    // Try to load tags configuration once for all datasets
    let tag_matcher = TagMatcher::from_config("tags.toml").ok();
    
    // Process each file into a dataset
    let mut datasets = Vec::new();
    
    for (i, file_path) in args.files.iter().enumerate() {
        let parse_start = Instant::now();
        let words = TextParser::parse_file(file_path)?;
        let parse_duration = parse_start.elapsed();
        
        let analyze_start = Instant::now();
        
        let mut analyzer = if let Some(ref tag_matcher) = tag_matcher {
            WordAnalyzer::with_tags(tag_matcher.clone())
        } else {
            WordAnalyzer::new()
        };
        
        let word_counts = analyzer.analyze(words);
        let analyze_duration = analyze_start.elapsed();
        
        // Determine dataset name (custom name or filename)
        let dataset_name = if i < args.names.len() {
            args.names[i].clone()
        } else {
            std::path::Path::new(file_path)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Unknown")
                .to_string()
        };
        
        datasets.push(Dataset {
            name: dataset_name,
            word_counts,
            total_words: analyzer.total_words(),
            unique_words: analyzer.unique_words(),
            parse_duration,
            analyze_duration,
        });
    }
    
    let total_duration = start_time.elapsed();

    if args.no_interactive {
        print_multi_results(&datasets, args.top, total_duration);
        
        if let Some(output_file) = args.output {
            write_multi_results_to_file(&datasets, &output_file)?;
        }
    } else {
        run_multi_tui(datasets, total_duration)?;
    }

    Ok(())
}

fn run_multi_tui(
    datasets: Vec<Dataset>,
    total_duration: std::time::Duration,
) -> anyhow::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(datasets, total_duration);
    let res = app.run(&mut terminal);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    res?;
    Ok(())
}

fn print_multi_results(
    datasets: &[Dataset],
    top: usize,
    total_duration: std::time::Duration,
) {
    println!("Zipfian Multi-Dataset Analysis Results");
    println!("=====================================");
    println!("Datasets analyzed: {}", datasets.len());
    println!("Total processing time: {:.2?}", total_duration);
    println!();

    for (i, dataset) in datasets.iter().enumerate() {
        println!("Dataset {}: {}", i + 1, dataset.name);
        println!("  Total words: {}", dataset.total_words);
        println!("  Unique words: {}", dataset.unique_words);
        println!("  Parse time: {:.2?}", dataset.parse_duration);
        println!("  Analysis time: {:.2?}", dataset.analyze_duration);
        println!("  Words per second: {:.0}", dataset.total_words as f64 / (dataset.parse_duration + dataset.analyze_duration).as_secs_f64());
        println!();
        println!("  {:>4} | {:20} | {:>8}", "Rank", "Word", "Count");
        println!("  {:->4}-+-{:->20}-+-{:->8}", "", "", "");

        for word_count in dataset.word_counts.iter().take(top) {
            println!(
                "  {:>4} | {:20} | {:>8}",
                word_count.rank, word_count.word, word_count.count
            );
        }
        println!();
    }
}

fn write_multi_results_to_file(
    datasets: &[Dataset],
    output_file: &str,
) -> anyhow::Result<()> {
    use std::fs::File;
    use std::io::Write;

    let mut file = File::create(output_file)?;
    writeln!(file, "Zipfian Multi-Dataset Analysis Results")?;
    writeln!(file, "=====================================")?;
    writeln!(file, "Datasets analyzed: {}", datasets.len())?;
    writeln!(file)?;

    for (i, dataset) in datasets.iter().enumerate() {
        writeln!(file, "Dataset {}: {}", i + 1, dataset.name)?;
        writeln!(file, "Total words: {}", dataset.total_words)?;
        writeln!(file, "Unique words: {}", dataset.unique_words)?;
        writeln!(file)?;
        writeln!(file, "Rank,Word,Count")?;

        for word_count in &dataset.word_counts {
            writeln!(file, "{},{},{}", word_count.rank, word_count.word, word_count.count)?;
        }
        writeln!(file)?;
    }

    println!("Results written to {}", output_file);
    Ok(())
}
