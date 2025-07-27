use clap::Parser;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::time::Instant;
use zipfr::{analyzer::WordAnalyzer, cli::Args, parser::TextParser, tui::App};

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let start_time = Instant::now();
    
    // Extract file title for display
    let file_title = std::path::Path::new(&args.file)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Unknown")
        .to_string();
    
    let parse_start = Instant::now();
    let words = TextParser::parse_file(&args.file)?;
    let parse_duration = parse_start.elapsed();
    
    let analyze_start = Instant::now();
    let mut analyzer = WordAnalyzer::new();
    let word_counts = analyzer.analyze(words);
    let analyze_duration = analyze_start.elapsed();
    
    let total_duration = start_time.elapsed();

    if args.no_interactive {
        print_results(&word_counts, args.top, &analyzer, parse_duration, analyze_duration, total_duration);
        
        if let Some(output_file) = args.output {
            write_results_to_file(&word_counts, &output_file, &analyzer)?;
        }
    } else {
        run_tui(word_counts, analyzer.total_words(), analyzer.unique_words(), parse_duration, analyze_duration, total_duration, file_title)?;
    }

    Ok(())
}

fn run_tui(
    word_counts: Vec<zipfr::WordCount>, 
    total_words: usize, 
    unique_words: usize,
    parse_duration: std::time::Duration,
    analyze_duration: std::time::Duration,
    total_duration: std::time::Duration,
    file_title: String,
) -> anyhow::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(word_counts, total_words, unique_words, parse_duration, analyze_duration, total_duration, file_title);
    let res = app.run(&mut terminal);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    res?;
    Ok(())
}

fn print_results(
    word_counts: &[zipfr::WordCount], 
    top: usize, 
    analyzer: &WordAnalyzer,
    parse_duration: std::time::Duration,
    analyze_duration: std::time::Duration,
    total_duration: std::time::Duration,
) {
    println!("Zipfian Text Analysis Results");
    println!("============================");
    println!("Total words: {}", analyzer.total_words());
    println!("Unique words: {}", analyzer.unique_words());
    println!();
    println!("Performance Metrics:");
    println!("  File parsing: {:.2?}", parse_duration);
    println!("  Word analysis: {:.2?}", analyze_duration);
    println!("  Total processing: {:.2?}", total_duration);
    println!("  Words per second: {:.0}", analyzer.total_words() as f64 / total_duration.as_secs_f64());
    println!();
    println!("{:>4} | {:20} | {:>8}", "Rank", "Word", "Count");
    println!("{:->4}-+-{:->20}-+-{:->8}", "", "", "");

    for word_count in word_counts.iter().take(top) {
        println!(
            "{:>4} | {:20} | {:>8}",
            word_count.rank, word_count.word, word_count.count
        );
    }
}

fn write_results_to_file(
    word_counts: &[zipfr::WordCount],
    output_file: &str,
    analyzer: &WordAnalyzer,
) -> anyhow::Result<()> {
    use std::fs::File;
    use std::io::Write;

    let mut file = File::create(output_file)?;
    writeln!(file, "Zipfian Text Analysis Results")?;
    writeln!(file, "============================")?;
    writeln!(file, "Total words: {}", analyzer.total_words())?;
    writeln!(file, "Unique words: {}", analyzer.unique_words())?;
    writeln!(file)?;
    writeln!(file, "{:>4},{:20},{:>8}", "Rank", "Word", "Count")?;

    for word_count in word_counts {
        writeln!(file, "{},{},{}", word_count.rank, word_count.word, word_count.count)?;
    }

    println!("Results written to {}", output_file);
    Ok(())
}
