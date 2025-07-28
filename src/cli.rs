use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "zipfr")]
#[command(about = "A Zipfian text analysis tool with TUI interface")]
#[command(version = "0.1.0")]
pub struct Args {
    #[arg(help = "Path(s) to the text file(s) to analyze", required = true)]
    pub files: Vec<String>,

    #[arg(short, long, help = "Display top N words", default_value = "20")]
    pub top: usize,

    #[arg(long, help = "Disable interactive TUI mode (use CLI output)")]
    pub no_interactive: bool,

    #[arg(short, long, help = "Output results to file")]
    pub output: Option<String>,

    #[arg(short = 'n', long = "name", help = "Custom names for datasets (one per file, overrides filenames)")]
    pub names: Vec<String>,
}