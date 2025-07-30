<div align="center">

# Zipfr

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


<div align="center">
  <img src="images/zipfr_screenshot_chrt.png" alt="TUI chart screenshot">
</div>

<div align="center">
  <img src="images/zipfr_screenshot_cmp.png" alt="TUI comparison screenshot">
</div>

## ✨ Features

### 🚀 **Performance**
- **Blazingly fast** word counting with HashMap-based O(1) lookups
- **Memory efficient** streaming for large files
- **Sub-second analysis** of most text files
- **Benchmarking metrics** showing words/second processing speed

### 🎨 **Interactive TUI (Default)**
- **Multi-dataset comparative analysis** with side-by-side view (up to 4 datasets)
- **Dual viewing modes**:
  - **Multi-Dataset View**: Side-by-side comparison of multiple datasets
  - **Chart Mode**: Single dataset with full visualization capabilities
- **Dynamic Zipf distribution chart** with perfect list-chart synchronization
- **Advanced Vim-like navigation** (`j/k`, `g/G`, `Ctrl+u/d/f/b`, `h/l`)
- **Dataset navigation**: `Tab`/`Shift+Tab` for cycling, `[`/`]` in chart mode
- **Intelligent search** (`/`) with fuzzy matching and `n/N` navigation
- **Visual cursor** with color-coded Zipf fit indicators
- **Log-log scale visualization** (`L`) - proper academic standard for power laws
- **Dual chart scope modes** (`A`):
  - **Visible Range**: Detailed analysis of current view
  - **All Data**: Complete corpus overview from rank 1 to N
- **Context-aware Zipf analysis** (`Z`):
  - **VISIBLE scope**: Absolute vs Relative reference lines for micro-analysis
  - **ALL-DATA scope**: Filtered vs Unfiltered basis for macro-analysis
- **Goodness of fit analysis** with color-coded deviation indicators
- **Horizontal scrolling** for more than 4 datasets
- **Active dataset highlighting** with visual indicators
- **Responsive layout** adapting to terminal width
- **Percentage normalization** (`%`): Toggle between raw counts and percentage display
- **Intelligent filtering feedback**: Real-time display of filtering impact:
  - **Inline display**: `Total Words: 63456/123456 (51%) | Unique Words: 950/1000 (95%)`
  - **Contextual information**: Shows both total word and unique word filtering effects

### 📊 **Advanced Analysis Features**
- **Log-log scale visualization** - Academic standard for power law analysis
- **Context-aware Zipf analysis** with intelligent mode switching:
  - **VISIBLE scope**: Absolute vs Relative reference lines for detailed range analysis
  - **ALL-DATA scope**: Filtered vs Unfiltered basis for comprehensive corpus analysis
- **Dual-scope analysis**:
  - **Micro-analysis**: Focus on specific rank ranges with visible scope
  - **Macro-analysis**: Complete corpus-wide distribution with all-data scope
- **Zipf law adherence analysis** with color-coded fit indicators:
  - 🟢 **Green**: Perfect fit (±10%)
  - 🟡 **Yellow**: Good fit (±30%)
  - 🔵 **Blue/Red**: Extreme deviations
- **Real-time chart synchronization** between list and visualization
- **Persistent cursor tracking** - chart cursor remains visible even when scrolling outside initial range
- **Customizable output** (top N words)
- **CSV export** for further analysis
- **Clean text parsing** handling punctuation and normalization
- **Real-time performance metrics**

### 🏷️ **Advanced Multi-Filter System**
- **Comprehensive word tagging** with 6 built-in categories:
  - **Stop Words**: Common function words (the, and, of, etc.)
  - **Sentiment**: Positive and negative emotional words
  - **Academic**: Scholarly and technical terminology
  - **Temporal**: Time-related words (now, then, during, etc.)
  - **Quantitative**: Numbers and measurement words
- **Multi-filter support**: Combine multiple filters simultaneously
  - **Additive filtering**: Build complex filter combinations
  - **Conflict prevention**: Automatic resolution of contradictory filters
  - **Global application**: Consistent filtering across all datasets
- **Quick filter toggles**:
  - **Stop words** (`S`): Instant common word exclusion
  - **Single-occurrence words** (`U`): Hide/show words appearing only once
- **Cross-dataset filtering** (`X`): Multi-dataset comparative analysis
  - **Common words**: Show only words that appear in ALL datasets
  - **Unique words**: Show only words unique to each dataset
  - **Efficient computation**: O(1) lookups with cached word sets
- **Advanced filter interface** (`F`): Two-step tag-based filtering
- **Real-time filter feedback**: Header shows filtering impact on both total and unique word counts
- **Visual tag indicators** showing `[S,P,A]` letters with color coding
- **TOML-based configuration** for easy tag customization
- **Zero performance impact** - tags applied once during analysis

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
# Single dataset analysis
zipfr document.txt

# Multi-dataset comparative analysis
zipfr alice.txt dracula.txt frankenstein.txt

# With custom names
zipfr file1.txt file2.txt file3.txt --name "Dataset A" --name "Dataset B" --name "Dataset C"

# With options
zipfr document.txt --top 100 --output analysis.csv
```

### CLI Mode
```bash
# Single dataset analysis
zipfr document.txt --no-interactive

# Multi-dataset analysis
zipfr file1.txt file2.txt file3.txt --no-interactive --top 10

# Custom dataset names
zipfr data.txt --name "Customer Feedback Analysis"
zipfr corpus1.txt corpus2.txt --name "19th Century" --name "20th Century"

# Piped input with meaningful name
cat document.txt | zipfr /dev/stdin --name "Alice in Wonderland"
```

## 📖 Examples

### Basic Analysis
```bash
# Single dataset analysis
curl -s https://www.gutenberg.org/files/11/11-0.txt | zipfr /dev/stdin --name "Alice's Adventures in Wonderland"

# Multi-dataset comparative analysis
zipfr alice.txt dracula.txt frankenstein.txt --name "Alice" --name "Dracula" --name "Frankenstein"

# Research corpus comparison
zipfr corpus1.txt corpus2.txt corpus3.txt --name "19th Century" --name "20th Century" --name "Modern"
```

### Advanced Analysis Workflow
```bash
# 1. Launch multi-dataset comparative analysis
zipfr alice.txt dracula.txt frankenstein.txt --name "Alice" --name "Dracula" --name "Frankenstein"

# 2. In the TUI:
#    - View side-by-side comparison (default multi-dataset mode)
#    - Press 'Tab'/'Shift+Tab' to navigate between datasets
#    - Press 'C' to toggle to chart mode for detailed analysis
#    - In chart mode: use '['/']' to switch between datasets
#    - Press 'L' to enable log-log scale (academic standard)
#    - Press 'A' to toggle between VISIBLE and ALL-DATA scope
#    - Press 'Z' to cycle through context-aware Zipf reference lines
#    - Press '%' to toggle between raw counts and percentage display
#    - Press 'S' to exclude stopwords, 'U' to exclude single-occurrence words
#    - Use '/' to search for specific words across datasets

# 3. Export results for further analysis
zipfr alice.txt dracula.txt frankenstein.txt --no-interactive --output comparison.csv
```

### Tag Filtering Examples
```bash
# 1. Launch analysis and filter out common words
zipfr document.txt
# In TUI: Press 'S' to quickly hide stop words

# 2. Focus on positive sentiment words only
zipfr document.txt  
# In TUI: Press 'F' → '2' → 'i' to show only positive words

# 3. Exclude academic jargon for readability analysis
zipfr document.txt
# In TUI: Press 'F' → '4' → 'e' to hide academic terms

# 4. Analyze temporal language patterns
zipfr document.txt
# In TUI: Press 'F' → '5' → 'i' to show only temporal words

# 5. Multi-filter analysis: exclude stopwords AND single-occurrence words
zipfr document.txt
# In TUI: Press 'S' (exclude stopwords), then 'U' (exclude singles)
# Header shows: Total Words: 45000/100000 (45%) | Unique Words: 800/2000 (40%)
```

### Interactive Features
<div align="center">

| Key | Action | Description |
|-----|--------|-------------|
| **Navigation** | | |
| `j/k` `↑/↓` | Move | Line by line navigation |
| `h/l` | Move | Alternative line navigation |
| `g` / `G` | Jump | Go to top/bottom |
| `[num]g` | Goto | Jump to specific rank |
| `Ctrl+u/d` | Page | Half page up/down |
| `Ctrl+f/b` | Page | Full page up/down |
| **Multi-Dataset** | | |
| `C` | Chart Mode | Toggle: Multi-Dataset ↔ Chart Mode |
| `Tab` / `Shift+Tab` | Dataset Nav | Cycle through datasets |
| `[` / `]` | Chart Nav | Navigate datasets in chart mode |
| **Search** | | |
| `/` | Search | Fuzzy search with live results |
| `n/N` | Navigate | Next/previous search match |
| **Chart Controls** | | |
| `L` | Log Scale | Toggle log-log visualization |
| `A` | Chart Scope | Toggle: Visible Range ↔ All Data |
| `Z` | Zipf Toggle | Toggle Zipf reference lines on/off |
| `z` | Zipf Mode | Context-aware: VISIBLE(Abs→Rel) / ALL-DATA(Filt→Unfilt) |
| `%` | Normalize | Toggle: Raw counts ↔ Percentage display |
| **Filtering** | | |
| `F` | Filter Menu | Two-step tag filtering interface |
| `S` | Stop Words | Quick toggle stop word filter |
| `U` | Single Words | Toggle exclusion of single-occurrence words |
| `X` | Cross-Dataset | Cycle: Off → Common Words → Unique Words (multi-dataset only) |
| `c` | Clear | Clear all active filters (in filter menu) |
| **General** | | |
| `q` | Quit | Exit application |

</div>

### Sample Output

#### CLI Mode Output
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

#### TUI Mode Features
- **Single dataset**: Automatically opens in chart view with visualization
- **Multiple datasets**: Opens in comparison view, press `C` to switch to chart mode
- **Real-time filtering feedback**: `Total Words: 15000/26476 (57%) | Unique Words: 1200/2763 (43%)`
- **Context-aware Zipf analysis**: Z key behavior adapts to current chart scope (VISIBLE vs ALL-DATA)
- **Multi-filter combinations**: Apply multiple filters simultaneously with automatic conflict resolution

## 📋 Command Line Options

```
Usage: zipfr [OPTIONS] <FILES>...

Arguments:
  <FILES>...  Path(s) to the text file(s) to analyze

Options:
  -n, --name <NAMES>...        Custom names for datasets (one per file, overrides filenames)
  -t, --top <TOP>              Display top N words [default: 20]
      --no-interactive         Disable interactive TUI mode (use CLI output)
  -o, --output <OUTPUT>        Output results to file
  -h, --help                   Print help
  -V, --version                Print version
```

## 🏷️ Tag Configuration

Zipfr uses a `tags.toml` file to define word categories. The default configuration includes:

```toml
[[tags]]
name = "Stop Words"
letter = "S"
words = ["the", "and", "of", "to", "a", "in", "is", "it", "you", "that", ...]

[[tags]]
name = "Positive"
letter = "P" 
words = ["good", "great", "excellent", "amazing", "wonderful", "fantastic", ...]

[[tags]]
name = "Negative"
letter = "N"
words = ["bad", "terrible", "awful", "horrible", "disappointing", ...]

# ... additional tag categories
```

### Customizing Tags
1. **Edit existing tags**: Modify word lists in `tags.toml`
2. **Add new categories**: Create new `[[tags]]` sections
3. **Visual indicators**: Each tag shows its letter in the word list (`[S,P,A]`)
4. **Performance**: Tags are loaded once at startup for optimal speed

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
- **[serde](https://crates.io/crates/serde)** - Serialization for tag configuration
- **[toml](https://crates.io/crates/toml)** - TOML parsing for tag definitions

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

1. **Fork** the repository
2. **Create** a feature branch
3. **Make** your changes
4. **Add** tests
5. **Submit** a pull request

## 🗺 Roadmap

### ✅ **Completed**
- [x] **Multi-dataset comparative analysis** - Side-by-side comparison of up to 4 datasets
- [x] **Dual viewing modes** - Multi-dataset view and chart mode with seamless switching
- [x] **Advanced dataset navigation** - Tab/Shift+Tab cycling with horizontal scrolling
- [x] **Log-log scale visualization** - Academic standard for power law analysis
- [x] **Goodness of fit analysis** - Color-coded Zipf law adherence indicators
- [x] **Dual-scope analysis** - Micro and macro view capabilities
- [x] **Advanced search** - Fuzzy matching with navigation
- [x] **Chart-list synchronization** - Perfect spatial alignment
- [x] **Intelligent tagging system** - 6 built-in tag categories with visual indicators
- [x] **Two-step filtering interface** - Intuitive exclude/include workflow
- [x] **TOML-based tag configuration** - Easy customization and extension
- [x] **Multi-filter system** - Combine multiple tag filters simultaneously with conflict prevention
- [x] **Context-aware Zipf analysis** - Intelligent mode switching based on chart scope
- [x] **Percentage normalization** - Toggle between raw counts and percentage display
- [x] **Real-time filtering feedback** - Inline display of filtering impact on total and unique words
- [x] **Single-occurrence word filtering** - Quick toggle to exclude/include words appearing once
- [x] **Enhanced cursor tracking** - Chart cursor remains visible when scrolling outside initial range
- [x] **Smart default behaviors** - Single datasets default to chart view, intelligent Zipf basis selection

### 🚧 **Planned**
- [ ] **Custom tag creation** - Runtime tag definition without editing files
- [ ] **Multi-format support** (PDF, DOCX, EPUB)
- [ ] **Statistical analysis** (R², correlation coefficients)
- [ ] **Language detection** and automatic stop-word selection
- [ ] **N-gram analysis** (bigrams, trigrams)
- [ ] **Comparative analysis** between multiple texts
- [ ] **Export formats** (JSON, XML, LaTeX) with tag information
- [ ] **Regex-based tags** - Pattern matching for advanced categorization

## 📄 License

Licensed under the [MIT License](LICENSE).

## 🙏 Acknowledgments

- **George Kingsley Zipf** - Pioneer of quantitative linguistics
- **Rust Community** - For excellent crates and tooling

---

<div align="center">

**[⭐ Star this repo](https://github.com/joshroyelliott/zipfr)** if you find it useful!

</div>
