use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "zipfr")]
#[command(about = "A Zipfian text analysis tool with TUI interface")]
#[command(version = "0.1.0")]
pub struct Args {
    #[arg(help = "Path to the text file to analyze")]
    pub file: String,

    #[arg(short, long, help = "Display top N words", default_value = "20")]
    pub top: usize,

    #[arg(long, help = "Disable interactive TUI mode (use CLI output)")]
    pub no_interactive: bool,

    #[arg(short, long, help = "Output results to file")]
    pub output: Option<String>,

    #[arg(short = 'n', long = "name", help = "Custom name for the dataset (overrides filename)")]
    pub name: Option<String>,
}