<div align="center">

# 📊 Zipfr

**A blazingly fast Zipfian text analysis tool with interactive TUI**

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)]()

[Features](#-features) • [Installation](#-installation) • [Usage](#-usage) • [Examples](#-examples) • [Contributing](#-contributing)

</div>

---

## 🎯 What is Zipfr?

Zipfr is a high-performance command-line tool for analyzing text according to **Zipf's law** - the observation that word frequency follows a power-law distribution. Built in Rust for speed and reliability, it features an interactive terminal interface with real-time visualization.

> **Zipf's Law**: In natural language, the frequency of any word is inversely proportional to its rank. The most frequent word appears ~2× more than the 2nd, ~3× more than the 3rd, and so on.

## ✨ Features

### 🚀 **Performance**
- **Blazingly fast** word counting with HashMap-based O(1) lookups
- **Memory efficient** streaming for large files
- **Sub-second analysis** of most text files
- **Benchmarking metrics** showing words/second processing speed

### 🎨 **Interactive TUI (Default)**
- **Dynamic Zipf distribution chart** that updates as you scroll
- **Vim-like navigation** (`j/k`, `g/G`, `Ctrl+d/u`, etc.)
- **Visual cursor** highlighting selected word on chart
- **Logarithmic scale toggle** (`L`) for better power-law visualization
- **Dual Zipf line modes** (`Z`):
  - **Absolute**: Based on corpus-wide rank 1 frequency
  - **Relative**: Based on visible range for local analysis
- **Responsive layout** adapting to terminal width

### 📊 **Analysis Options**
- **Customizable output** (top N words)
- **CSV export** for further analysis
- **Clean text parsing** handling punctuation and normalization
- **Real-time performance metrics**

## 🛠 Installation

### Prerequisites
- **Rust 1.70+** - Install from [rustup.rs](https://rustup.rs/)

### From Source
```bash
git clone https://github.com/joshroyelliott/zipfr.git
cd zipfr
cargo build --release
```

### Using Cargo
```bash
cargo install zipfr
```

## 🚀 Usage

### Interactive Mode (Default)
```bash
# Launch interactive TUI (default)
zipfr document.txt

# With options
zipfr document.txt --top 100 --output analysis.csv
```

### CLI Mode
```bash
# Traditional output for scripts/automation
zipfr document.txt --no-interactive

# Quick analysis
zipfr document.txt --no-interactive --top 10
```

## 📖 Examples

### Basic Analysis
```bash
# Analyze Alice in Wonderland
curl -s https://www.gutenberg.org/files/11/11-0.txt | zipfr /dev/stdin
```

### Interactive Features
<div align="center">

| Key | Action | Description |
|-----|--------|-------------|
| `j/k` `↑/↓` | Navigate | Move through word list |
| `g` / `G` | Jump | Go to top/bottom |
| `[num]g` | Goto | Jump to specific rank |
| `Ctrl+d/u` | Page | Half page up/down |
| `L` | Log Scale | Toggle logarithmic Y-axis |
| `Z` | Zipf Mode | Cycle: Off → Absolute → Relative |
| `q` | Quit | Exit application |

</div>

### Sample Output
```
Zipfian Text Analysis Results
============================
Total words: 26,476
Unique words: 2,763

Performance Metrics:
  File parsing: 28.13ms
  Word analysis: 17.98ms  
  Total processing: 46.12ms
  Words per second: 574,038

Rank | Word                 |    Count
-----+----------------------+---------
   1 | the                  |     1640
   2 | and                  |      846
   3 | to                   |      721
   4 | a                    |      632
   5 | she                  |      537
```

## 📋 Command Line Options

```
Usage: zipfr [OPTIONS] <FILE>

Arguments:
  <FILE>  Path to the text file to analyze

Options:
  -t, --top <TOP>              Display top N words [default: 20]
      --no-interactive         Disable interactive TUI mode (use CLI output)
  -o, --output <OUTPUT>        Output results to file
  -h, --help                   Print help
  -V, --version                Print version
```

## 🏗 Architecture

<details>
<summary>Project Structure</summary>

```
src/
├── main.rs          # CLI entry point and application logic
├── lib.rs           # Library interface  
├── parser.rs        # Text parsing and word extraction
├── analyzer.rs      # Word counting and frequency analysis
├── cli.rs           # Command-line argument parsing
└── tui/             # Terminal user interface
    ├── mod.rs       # TUI module exports
    ├── app.rs       # Main TUI application
    └── chart.rs     # Chart visualization
```

</details>

## 🔧 Development

### Quick Start
```bash
git clone https://github.com/joshroyelliott/zipfr.git
cd zipfr
cargo run -- sample.txt
```

### Commands
```bash
cargo build --release    # Optimized build
cargo test               # Run tests  
cargo clippy             # Lint code
cargo fmt                # Format code
```

### Dependencies
- **[clap](https://crates.io/crates/clap)** - CLI argument parsing
- **[ratatui](https://crates.io/crates/ratatui)** - Terminal UI framework
- **[crossterm](https://crates.io/crates/crossterm)** - Cross-platform terminal
- **[anyhow](https://crates.io/crates/anyhow)** - Error handling

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

1. **Fork** the repository
2. **Create** a feature branch
3. **Make** your changes
4. **Add** tests
5. **Submit** a pull request

## 🗺 Roadmap

- [ ] **Multi-format support** (PDF, DOCX, EPUB)
- [ ] **Statistical analysis** (R², goodness of fit)
- [ ] **Language detection** and stop-word filtering  
- [ ] **N-gram analysis** (bigrams, trigrams)
- [ ] **Comparative analysis** between multiple texts
- [ ] **Export formats** (JSON, XML, LaTeX)

## 📄 License

Licensed under the [MIT License](LICENSE).

## 🙏 Acknowledgments

- **George Kingsley Zipf** - Pioneer of quantitative linguistics
- **Rust Community** - For excellent crates and tooling

---

<div align="center">

**[⭐ Star this repo](https://github.com/joshroyelliott/zipfr)** if you find it useful!

</div>
