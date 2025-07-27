use crate::analyzer::{WordCount, Tag};
use crate::tui::ChartWidget;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};

#[derive(Debug, Clone, PartialEq)]
pub enum ZipfMode {
    Off,
    Absolute,  // Based on global rank 1
    Relative,  // Based on visible range
}

#[derive(Debug, Clone, PartialEq)]
pub enum ChartScope {
    Relative,  // Show only visible list range
    Absolute,  // Show entire dataset
}

#[derive(Debug, Clone, PartialEq)]
pub enum FilterMode {
    None,                    // Show all words
    ExcludeTag(Tag),        // Hide words with this tag
    IncludeOnlyTag(Tag),    // Show only words with this tag
}

#[derive(Debug, Clone, PartialEq)]
pub enum FilterInputState {
    SelectingTag,           // Step 1: Show available tags
    SelectingAction(Tag),   // Step 2: Show include/exclude for selected tag
}

#[derive(Debug, Clone, PartialEq)]
pub enum InputMode {
    Normal,
    Search,
    NumberInput,
    Filter,
}
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame, Terminal,
};
use std::io;
use std::time::Duration;

pub struct App {
    pub word_counts: Vec<WordCount>,
    pub filtered_word_counts: Vec<WordCount>,
    pub selected_index: usize,
    pub should_quit: bool,
    pub total_words: usize,
    pub unique_words: usize,
    pub parse_duration: Duration,
    pub analyze_duration: Duration,
    pub total_duration: Duration,
    pub visible_area_height: usize,
    pub number_input: String,
    pub list_state: ListState,
    pub log_scale: bool,
    pub zipf_mode: ZipfMode,
    pub chart_scope: ChartScope,
    pub filter_mode: FilterMode,
    pub filter_dirty: bool,
    pub available_tags: Vec<Tag>,
    pub filter_input_state: FilterInputState,
    pub file_title: String,
    pub input_mode: InputMode,
    pub search_query: String,
    pub search_results: Vec<usize>,
    pub current_search_index: usize,
}

impl App {
    pub fn new(
        word_counts: Vec<WordCount>, 
        total_words: usize, 
        unique_words: usize,
        parse_duration: Duration,
        analyze_duration: Duration,
        total_duration: Duration,
        file_title: String,
    ) -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));
        
        // Extract available tags from word counts
        let mut available_tags = Vec::new();
        for word_count in &word_counts {
            for tag in &word_count.tags {
                if !available_tags.contains(tag) {
                    available_tags.push(tag.clone());
                }
            }
        }

        Self {
            filtered_word_counts: word_counts.clone(),
            word_counts,
            selected_index: 0,
            should_quit: false,
            total_words,
            unique_words,
            parse_duration,
            analyze_duration,
            total_duration,
            visible_area_height: 20, // Default, will be updated
            number_input: String::new(),
            list_state,
            log_scale: false,
            zipf_mode: ZipfMode::Off,
            chart_scope: ChartScope::Relative,
            filter_mode: FilterMode::None,
            filter_dirty: false,
            available_tags,
            filter_input_state: FilterInputState::SelectingTag,
            file_title,
            input_mode: InputMode::Normal,
            search_query: String::new(),
            search_results: Vec::new(),
            current_search_index: 0,
        }
    }

    fn update_selection(&mut self, new_index: usize) {
        let active_words_len = {
            if self.filter_dirty || self.filtered_word_counts.is_empty() {
                self.apply_current_filter();
                self.filter_dirty = false;
            }
            self.filtered_word_counts.len()
        };
        let max_index = active_words_len.saturating_sub(1);
        self.selected_index = new_index.min(max_index);
        self.list_state.select(Some(self.selected_index));
    }



    fn fuzzy_match(query: &str, word: &str) -> Option<f32> {
        if query.is_empty() {
            return None;
        }
        
        let query_lower = query.to_lowercase();
        let word_lower = word.to_lowercase();
        
        if word_lower.contains(&query_lower) {
            // Score based on position and length - earlier matches score higher
            let pos = word_lower.find(&query_lower).unwrap();
            let score = 1.0 - (pos as f32 / word_lower.len() as f32);
            Some(score)
        } else {
            None
        }
    }

    fn update_search_results(&mut self) {
        self.search_results.clear();
        
        if self.search_query.is_empty() {
            return;
        }

        // Find all matching words with scores
        let mut matches: Vec<(usize, f32)> = self.word_counts
            .iter()
            .enumerate()
            .filter_map(|(i, word_count)| {
                Self::fuzzy_match(&self.search_query, &word_count.word)
                    .map(|score| (i, score))
            })
            .collect();

        // Sort by score (highest first)
        matches.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        
        // Extract indices
        self.search_results = matches.into_iter().map(|(i, _)| i).collect();
        self.current_search_index = 0;
    }

    fn calculate_zipf_fit(&self, word_count: &WordCount, visible_words: &[WordCount]) -> Option<f64> {
        match self.zipf_mode {
            ZipfMode::Off => None,
            ZipfMode::Absolute => {
                // Compare to global Zipf: ideal_freq = first_word_freq / rank
                if let Some(global_first) = self.word_counts.first() {
                    let global_first_freq = global_first.count as f64;
                    let ideal_freq = global_first_freq / word_count.rank as f64;
                    let actual_freq = word_count.count as f64;
                    Some(actual_freq / ideal_freq)
                } else {
                    None
                }
            },
            ZipfMode::Relative => {
                // Compare to relative Zipf within visible range
                if let Some(visible_first) = visible_words.first() {
                    let visible_first_freq = visible_first.count as f64;
                    let visible_first_rank = visible_first.rank as f64;
                    let constant = visible_first_freq * visible_first_rank;
                    let ideal_freq = constant / word_count.rank as f64;
                    let actual_freq = word_count.count as f64;
                    Some(actual_freq / ideal_freq)
                } else {
                    None
                }
            }
        }
    }

    fn deviation_to_color(ratio: f64) -> Color {
        match ratio {
            r if r >= 0.9 && r <= 1.1 => Color::Green,      // Perfect fit (±10%)
            r if r >= 0.7 && r < 0.9 => Color::Yellow,       // Good fit (underperforming)
            r if r > 1.1 && r <= 1.3 => Color::Yellow,       // Good fit (overperforming)
            r if r >= 0.5 && r < 0.7 => Color::Cyan,         // Moderate underperforming
            r if r > 1.3 && r <= 2.0 => Color::Magenta,      // Moderate overperforming
            r if r < 0.5 => Color::Blue,                     // Extreme underperforming
            r if r > 2.0 => Color::Red,                      // Extreme overperforming
            _ => Color::Gray,                                // Fallback
        }
    }

    pub fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> io::Result<()> {
        loop {
            terminal.draw(|f| self.ui(f))?;

            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match self.input_mode {
                        InputMode::Search => self.handle_search_input(key),
                        InputMode::NumberInput => self.handle_number_input(key),
                        InputMode::Filter => self.handle_filter_input(key),
                        InputMode::Normal => self.handle_normal_input(key),
                    }
                    
                    if self.should_quit {
                        return Ok(());
                    }
                }
            }
        }
    }

    fn handle_search_input(&mut self, key: crossterm::event::KeyEvent) {
        match key.code {
            KeyCode::Enter => {
                // Jump to first match and exit search mode
                if !self.search_results.is_empty() {
                    self.update_selection(self.search_results[0]);
                }
                self.input_mode = InputMode::Normal;
            }
            KeyCode::Esc => {
                // Cancel search
                self.search_query.clear();
                self.search_results.clear();
                self.input_mode = InputMode::Normal;
            }
            KeyCode::Backspace => {
                self.search_query.pop();
                self.update_search_results();
            }
            KeyCode::Char(c) => {
                self.search_query.push(c);
                self.update_search_results();
            }
            _ => {}
        }
    }

    fn handle_number_input(&mut self, key: crossterm::event::KeyEvent) {
        match key.code {
            KeyCode::Char(c) if c.is_ascii_digit() => {
                self.number_input.push(c);
            }
            KeyCode::Esc => {
                self.number_input.clear();
                self.input_mode = InputMode::Normal;
            }
            KeyCode::Char('g') | KeyCode::Char('G') => {
                if !self.number_input.is_empty() {
                    if let Ok(line_num) = self.number_input.parse::<usize>() {
                        let active_words_len = {
                            if self.filter_dirty || self.filtered_word_counts.is_empty() {
                                self.apply_current_filter();
                                self.filter_dirty = false;
                            }
                            self.filtered_word_counts.len()
                        };
                        let new_index = (line_num.saturating_sub(1))
                            .min(active_words_len.saturating_sub(1));
                        self.update_selection(new_index);
                    }
                    self.number_input.clear();
                }
                self.input_mode = InputMode::Normal;
            }
            _ => {}
        }
    }

    fn handle_filter_input(&mut self, key: crossterm::event::KeyEvent) {
        match &self.filter_input_state.clone() {
            FilterInputState::SelectingTag => {
                match key.code {
                    KeyCode::Esc => {
                        // Exit filter mode
                        self.input_mode = InputMode::Normal;
                    }
                    KeyCode::Char('c') => {
                        // Clear all filters and exit
                        self.filter_mode = FilterMode::None;
                        self.filter_dirty = true;
                        self.input_mode = InputMode::Normal;
                    }
                    KeyCode::Char(c) if c.is_ascii_digit() => {
                        // Select tag by number - move to step 2
                        if let Some(digit) = c.to_digit(10) {
                            let index = (digit as usize).saturating_sub(1);
                            if index < self.available_tags.len() {
                                let tag = self.available_tags[index].clone();
                                self.filter_input_state = FilterInputState::SelectingAction(tag);
                            }
                        }
                    }
                    _ => {}
                }
            }
            FilterInputState::SelectingAction(tag) => {
                match key.code {
                    KeyCode::Esc => {
                        // Go back to tag selection
                        self.filter_input_state = FilterInputState::SelectingTag;
                    }
                    KeyCode::Char('e') => {
                        // Exclude this tag
                        self.filter_mode = FilterMode::ExcludeTag(tag.clone());
                        self.filter_dirty = true;
                        self.input_mode = InputMode::Normal;
                    }
                    KeyCode::Char('i') => {
                        // Include only this tag
                        self.filter_mode = FilterMode::IncludeOnlyTag(tag.clone());
                        self.filter_dirty = true;
                        self.input_mode = InputMode::Normal;
                    }
                    _ => {}
                }
            }
        }
    }

    fn handle_normal_input(&mut self, key: crossterm::event::KeyEvent) {
        match (key.code, key.modifiers) {
                        (KeyCode::Char('q'), _) => {
                            self.should_quit = true;
                        }
                        // Basic movement
                        (KeyCode::Down, _) | (KeyCode::Char('j'), _) => {
                            let active_words_len = {
                                if self.filter_dirty || self.filtered_word_counts.is_empty() {
                                    self.apply_current_filter();
                                    self.filter_dirty = false;
                                }
                                self.filtered_word_counts.len()
                            };
                            if self.selected_index < active_words_len.saturating_sub(1) {
                                self.update_selection(self.selected_index + 1);
                            }
                        }
                        (KeyCode::Up, _) | (KeyCode::Char('k'), _) => {
                            if self.selected_index > 0 {
                                self.update_selection(self.selected_index - 1);
                            }
                        }
                        // Vim-like navigation
                        (KeyCode::Char('g'), _) => {
                            // Handle 'gg' - go to top, or number+g to go to line
                            if !self.number_input.is_empty() {
                                if let Ok(line_num) = self.number_input.parse::<usize>() {
                                    let new_index = (line_num.saturating_sub(1))
                                        .min(self.word_counts.len().saturating_sub(1));
                                    self.update_selection(new_index);
                                }
                                self.number_input.clear();
                            } else {
                                self.update_selection(0);
                            }
                        }
                        (KeyCode::Char('G'), _) => {
                            // Go to bottom, or number+G to go to specific line
                            if !self.number_input.is_empty() {
                                if let Ok(line_num) = self.number_input.parse::<usize>() {
                                    let new_index = (line_num.saturating_sub(1))
                                        .min(self.word_counts.len().saturating_sub(1));
                                    self.update_selection(new_index);
                                }
                                self.number_input.clear();
                            } else {
                                self.update_selection(self.word_counts.len().saturating_sub(1));
                            }
                        }
                        (KeyCode::Char('d'), KeyModifiers::CONTROL) => {
                            // Ctrl+d - half page down
                            let half_page = self.visible_area_height / 2;
                            let new_index = (self.selected_index + half_page)
                                .min(self.word_counts.len().saturating_sub(1));
                            self.update_selection(new_index);
                        }
                        (KeyCode::Char('u'), KeyModifiers::CONTROL) => {
                            // Ctrl+u - half page up
                            let half_page = self.visible_area_height / 2;
                            let new_index = self.selected_index.saturating_sub(half_page);
                            self.update_selection(new_index);
                        }
                        (KeyCode::Char('f'), KeyModifiers::CONTROL) => {
                            // Ctrl+f - full page down
                            let full_page = self.visible_area_height;
                            let new_index = (self.selected_index + full_page)
                                .min(self.word_counts.len().saturating_sub(1));
                            self.update_selection(new_index);
                        }
                        (KeyCode::Char('b'), KeyModifiers::CONTROL) => {
                            // Ctrl+b - full page up
                            let full_page = self.visible_area_height;
                            let new_index = self.selected_index.saturating_sub(full_page);
                            self.update_selection(new_index);
                        }
                        (KeyCode::Char('h'), _) => {
                            // h - move left (same as up in this context)
                            if self.selected_index > 0 {
                                self.update_selection(self.selected_index - 1);
                            }
                        }
                        (KeyCode::Char('l'), _) => {
                            // l - move right (same as down in this context)
                            if self.selected_index < self.word_counts.len().saturating_sub(1) {
                                self.update_selection(self.selected_index + 1);
                            }
                        }
                        // Traditional keys
                        (KeyCode::Home, _) => self.update_selection(0),
                        (KeyCode::End, _) => {
                            self.update_selection(self.word_counts.len().saturating_sub(1));
                        }
                        (KeyCode::PageDown, _) => {
                            let full_page = self.visible_area_height;
                            let new_index = (self.selected_index + full_page)
                                .min(self.word_counts.len().saturating_sub(1));
                            self.update_selection(new_index);
                        }
                        (KeyCode::PageUp, _) => {
                            let full_page = self.visible_area_height;
                            let new_index = self.selected_index.saturating_sub(full_page);
                            self.update_selection(new_index);
                        }
                        // Search mode
                        (KeyCode::Char('/'), _) => {
                            self.input_mode = InputMode::Search;
                            self.search_query.clear();
                        }
                        // Search navigation
                        (KeyCode::Char('n'), _) => {
                            if !self.search_results.is_empty() {
                                self.current_search_index = (self.current_search_index + 1) % self.search_results.len();
                                let result_index = self.search_results[self.current_search_index];
                                self.update_selection(result_index);
                            }
                        }
                        (KeyCode::Char('N'), _) => {
                            if !self.search_results.is_empty() {
                                self.current_search_index = if self.current_search_index == 0 {
                                    self.search_results.len() - 1
                                } else {
                                    self.current_search_index - 1
                                };
                                let result_index = self.search_results[self.current_search_index];
                                self.update_selection(result_index);
                            }
                        }
                        // Number input for line jumping
                        (KeyCode::Char(c), _) if c.is_ascii_digit() => {
                            self.number_input.push(c);
                            self.input_mode = InputMode::NumberInput;
                        }
                        // Chart toggles
                        (KeyCode::Char('L'), _) => {
                            self.log_scale = !self.log_scale;
                        }
                        (KeyCode::Char('Z'), _) => {
                            self.zipf_mode = match self.zipf_mode {
                                ZipfMode::Off => ZipfMode::Absolute,
                                ZipfMode::Absolute => ZipfMode::Relative,
                                ZipfMode::Relative => ZipfMode::Off,
                            };
                        }
                        (KeyCode::Char('A'), _) => {
                            self.chart_scope = match self.chart_scope {
                                ChartScope::Relative => ChartScope::Absolute,
                                ChartScope::Absolute => ChartScope::Relative,
                            };
                        }
                        (KeyCode::Char('S'), _) => {
                            // Toggle stop word filter
                            self.toggle_stopword_filter();
                        }
                        (KeyCode::Char('F'), _) => {
                            // Enter filter mode
                            self.filter_input_state = FilterInputState::SelectingTag;
                            self.input_mode = InputMode::Filter;
                        }

                        _ => {}
        }
    }

    fn ui(&mut self, f: &mut Frame) {
        // Calculate footer height dynamically based on what will be displayed
        let mut footer_height = 2; // Base height for borders
        
        // Always show navigation line
        footer_height += 1;
        
        // Chart/status line (when any chart mode is active OR filter is active)
        if self.log_scale || self.zipf_mode != ZipfMode::Off || self.chart_scope != ChartScope::Relative || self.filter_mode != FilterMode::None {
            footer_height += 1;
        }
        
        // Input mode lines
        match self.input_mode {
            InputMode::Search => footer_height += 1,
            InputMode::NumberInput => {
                if !self.number_input.is_empty() {
                    footer_height += 1;
                }
            },
            InputMode::Filter => {
                match &self.filter_input_state {
                    FilterInputState::SelectingTag => {
                        footer_height += 1; // Just the main filter line
                    },
                    FilterInputState::SelectingAction(_) => footer_height += 1,
                }
            },
            InputMode::Normal => {},
        }
        
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(5), // Increased for file title
                Constraint::Min(10),
                Constraint::Length(footer_height),
            ])
            .split(f.size());

        // Split the main content area horizontally - dynamic width based on terminal size
        let terminal_width = f.size().width;
        let word_list_width = if terminal_width > 120 { 25 } else if terminal_width > 80 { 30 } else { 35 };
        
        let content_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(word_list_width), // Word list
                Constraint::Percentage(100 - word_list_width), // Chart
            ])
            .split(main_chunks[1]);

        // Update visible area height for dynamic page sizing
        self.visible_area_height = content_chunks[0].height.saturating_sub(2) as usize; // Subtract border

        self.render_header(f, main_chunks[0]);
        self.render_word_list(f, content_chunks[0]);
        self.render_chart(f, content_chunks[1]);
        self.render_footer(f, main_chunks[2]);
    }

    fn render_header(&self, f: &mut Frame, area: Rect) {
        let words_per_sec = self.total_words as f64 / self.total_duration.as_secs_f64();
        let header = Paragraph::new(vec![
            Line::from(vec![
                Span::styled(&self.file_title, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(vec![
                Span::styled("Zipfian Text Analysis", Style::default().fg(Color::Gray)),
                Span::raw(" | "),
                Span::styled(
                    format!("Total Words: {}", self.total_words),
                    Style::default().fg(Color::Yellow),
                ),
                Span::raw(" | "),
                Span::styled(
                    format!("Unique Words: {}", self.unique_words),
                    Style::default().fg(Color::Green),
                ),
            ]),
            Line::from(vec![
                Span::styled("Performance: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!("Parse: {:.2?}", self.parse_duration),
                    Style::default().fg(Color::Blue),
                ),
                Span::raw(" | "),
                Span::styled(
                    format!("Analysis: {:.2?}", self.analyze_duration),
                    Style::default().fg(Color::Blue),
                ),
                Span::raw(" | "),
                Span::styled(
                    format!("Total: {:.2?}", self.total_duration),
                    Style::default().fg(Color::Blue),
                ),
                Span::raw(" | "),
                Span::styled(
                    format!("{:.0} words/sec", words_per_sec),
                    Style::default().fg(Color::Magenta),
                ),
            ]),
        ])
        .block(Block::default().borders(Borders::ALL).title("Dataset"));
        f.render_widget(header, area);
    }

    fn render_word_list(&mut self, f: &mut Frame, area: Rect) {
        // Ensure filtered words are up to date
        if self.filter_dirty || self.filtered_word_counts.is_empty() {
            self.apply_current_filter();
            self.filter_dirty = false;
        }
        
        // Use the same bounds calculation as the chart for perfect synchronization
        let (visible_start, visible_end) = {
            let list_offset = self.list_state.offset();
            let visible_start = list_offset;
            let visible_end = (visible_start + self.visible_area_height).min(self.filtered_word_counts.len());
            (visible_start, visible_end)
        };
        
        let visible_words = if visible_end <= self.filtered_word_counts.len() {
            &self.filtered_word_counts[visible_start..visible_end]
        } else {
            &[]
        };

        let items: Vec<ListItem> = self.filtered_word_counts
            .iter()
            .enumerate()
            .map(|(i, word_count)| {
                // Check if this word is a search match
                let is_search_match = self.search_results.contains(&i);
                let word_style = if is_search_match {
                    Style::default().bg(Color::DarkGray).fg(Color::Yellow)
                } else {
                    Style::default()
                };

                let mut spans = vec![
                    Span::styled(format!("{:4}", word_count.rank), Style::default().fg(Color::Blue)),
                    Span::raw(" | "),
                    Span::styled(format!("{:12}", word_count.word), word_style),
                    Span::raw(" | "),
                    Span::styled(format!("{:6}", word_count.count), Style::default().fg(Color::Magenta)),
                ];

                // Add fit column if Zipf mode is active
                if self.zipf_mode != ZipfMode::Off {
                    if let Some(fit_ratio) = self.calculate_zipf_fit(word_count, visible_words) {
                        let fit_color = Self::deviation_to_color(fit_ratio);
                        let fit_display = if fit_ratio >= 10.0 {
                            "9+".to_string()
                        } else if fit_ratio < 0.1 {
                            "0.1".to_string()
                        } else {
                            format!("{:.1}", fit_ratio)
                        };
                        
                        spans.push(Span::raw(" |"));
                        spans.push(Span::styled(format!("{:>3}", fit_display), Style::default().fg(fit_color)));
                    } else {
                        spans.push(Span::raw(" | -"));
                    }
                }

                // Add tag indicators
                if !word_count.tags.is_empty() {
                    spans.push(Span::raw(" ["));
                    for (i, tag) in word_count.tags.iter().enumerate() {
                        if i > 0 { spans.push(Span::raw(",")); }
                        let tag_color = match tag.color.as_deref() {
                            Some("gray") => Color::Gray,
                            Some("green") => Color::Green,
                            Some("red") => Color::Red,
                            Some("blue") => Color::Blue,
                            Some("yellow") => Color::Yellow,
                            Some("cyan") => Color::Cyan,
                            _ => Color::Gray,
                        };
                        let first_char = tag.name.chars().next().unwrap_or('?');
                        spans.push(Span::styled(
                            first_char.to_string(),
                            Style::default().fg(tag_color)
                        ));
                    }
                    spans.push(Span::raw("]"));
                }

                ListItem::new(Line::from(spans))
            })
            .collect();

        // Create title with fit column indicator
        let title = if self.zipf_mode != ZipfMode::Off {
            match self.zipf_mode {
                ZipfMode::Absolute => "Word Frequencies (Absolute Fit)",
                ZipfMode::Relative => "Word Frequencies (Relative Fit)",
                ZipfMode::Off => "Word Frequencies", // Won't reach here
            }
        } else {
            "Word Frequencies"
        };

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title(title))
            .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD));

        f.render_stateful_widget(list, area, &mut self.list_state);
    }

    fn apply_current_filter(&mut self) {
        self.filtered_word_counts = match &self.filter_mode {
            FilterMode::None => self.word_counts.clone(),
            FilterMode::ExcludeTag(tag) => {
                self.word_counts.iter()
                    .filter(|wc| !wc.tags.contains(tag))
                    .cloned()
                    .collect()
            },
            FilterMode::IncludeOnlyTag(tag) => {
                self.word_counts.iter()
                    .filter(|wc| wc.tags.contains(tag))
                    .cloned()
                    .collect()
            },
        };

        // Re-rank the filtered words
        for (index, word_count) in self.filtered_word_counts.iter_mut().enumerate() {
            word_count.rank = index + 1;
        }

        // Reset selection if it's out of bounds
        if self.selected_index >= self.filtered_word_counts.len() {
            self.selected_index = 0;
            self.list_state.select(Some(0));
        }
    }

    fn toggle_stopword_filter(&mut self) {
        if let Some(stopword_tag) = self.available_tags.iter().find(|tag| tag.name == "Stop Words") {
            match &self.filter_mode {
                FilterMode::ExcludeTag(tag) if tag == stopword_tag => {
                    self.filter_mode = FilterMode::None;
                },
                _ => {
                    self.filter_mode = FilterMode::ExcludeTag(stopword_tag.clone());
                }
            }
            self.filter_dirty = true;
        }
    }

    fn render_chart(&mut self, f: &mut Frame, area: Rect) {
        // Ensure filtered words are up to date
        if self.filter_dirty || self.filtered_word_counts.is_empty() {
            self.apply_current_filter();
            self.filter_dirty = false;
        }
        
        // Use the exact same bounds as the visible list
        let (visible_start, visible_end) = {
            let list_offset = self.list_state.offset();
            let visible_start = list_offset;
            let visible_end = (visible_start + self.visible_area_height).min(self.filtered_word_counts.len());
            (visible_start, visible_end)
        };
        
        // Get the visible slice of word counts that exactly matches the list
        let visible_words = if visible_end <= self.filtered_word_counts.len() {
            &self.filtered_word_counts[visible_start..visible_end]
        } else {
            &[]
        };
        
        // Calculate fit ratio for the selected word if in Zipf mode
        let selected_fit_ratio = if self.selected_index < self.filtered_word_counts.len() {
            let selected_word = &self.filtered_word_counts[self.selected_index];
            self.calculate_zipf_fit(selected_word, visible_words)
        } else {
            None
        };
        
        ChartWidget::render_enhanced(
            f, 
            area, 
            visible_words, 
            &self.filtered_word_counts, // Pass active (filtered) word counts
            self.log_scale, 
            &self.zipf_mode,
            &self.chart_scope,
            self.selected_index,
            visible_start,
            selected_fit_ratio
        );
    }

    fn render_footer(&self, f: &mut Frame, area: Rect) {
        let mut lines = vec![
            Line::from("Navigation: j/k | g/G/[num]g | Ctrl+u/d/b/f | Chart: L(log) Z(zipf) A(scope) | Filter: S(stopwords) F(filter) | /(search) n/N | q(quit)")
        ];
        
        // Show current chart modes and filter status on one line
        let mut chart_status = Vec::new();
        if self.log_scale {
            chart_status.push(Span::styled("LOG", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));
        }
        match self.chart_scope {
            ChartScope::Absolute => {
                if !chart_status.is_empty() { chart_status.push(Span::raw(" | ")); }
                chart_status.push(Span::styled("ALL-DATA", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)));
            },
            ChartScope::Relative => {
                if !chart_status.is_empty() { chart_status.push(Span::raw(" | ")); }
                chart_status.push(Span::styled("VISIBLE", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)));
            },
        }
        match self.zipf_mode {
            ZipfMode::Absolute => {
                if !chart_status.is_empty() { chart_status.push(Span::raw(" | ")); }
                chart_status.push(Span::styled("ZIPF-ABS", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)));
            },
            ZipfMode::Relative => {
                if !chart_status.is_empty() { chart_status.push(Span::raw(" | ")); }
                chart_status.push(Span::styled("ZIPF-REL", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)));
            },
            ZipfMode::Off => {},
        }
        
        // Add filter status to the same line
        match &self.filter_mode {
            FilterMode::None => {},
            FilterMode::ExcludeTag(tag) => {
                if !chart_status.is_empty() { chart_status.push(Span::raw(" | ")); }
                chart_status.push(Span::styled("Filter: ", Style::default().fg(Color::Gray)));
                chart_status.push(Span::styled("Excluding ", Style::default().fg(Color::Red)));
                chart_status.push(Span::styled(&tag.name, Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)));
            },
            FilterMode::IncludeOnlyTag(tag) => {
                if !chart_status.is_empty() { chart_status.push(Span::raw(" | ")); }
                chart_status.push(Span::styled("Filter: ", Style::default().fg(Color::Gray)));
                chart_status.push(Span::styled("Only ", Style::default().fg(Color::Green)));
                chart_status.push(Span::styled(&tag.name, Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)));
            },
        }
        
        // Show the combined status line if there's anything to show
        if !chart_status.is_empty() {
            let mut status_line = vec![Span::styled("Chart modes: ", Style::default().fg(Color::Gray))];
            status_line.extend(chart_status);
            lines.push(Line::from(status_line));
        }
        
        // Show search UI
        match self.input_mode {
            InputMode::Search => {
                let mut search_line = vec![
                    Span::styled("Search: ", Style::default().fg(Color::Yellow)),
                    Span::styled(&self.search_query, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                    Span::raw("_"), // Cursor
                ];

                if self.search_results.is_empty() && !self.search_query.is_empty() {
                    search_line.push(Span::styled(" | No matches", Style::default().fg(Color::Gray)));
                } else if !self.search_results.is_empty() {
                    search_line.push(Span::styled(
                        format!(" | Match {} of {}", self.current_search_index + 1, self.search_results.len()),
                        Style::default().fg(Color::Gray)
                    ));
                }
                
                search_line.push(Span::styled(" | Enter(jump) Esc(cancel)", Style::default().fg(Color::Gray)));
                lines.push(Line::from(search_line));
            },
            InputMode::NumberInput => {
                if !self.number_input.is_empty() {
                    lines.push(Line::from(vec![
                        Span::styled("Line: ", Style::default().fg(Color::Yellow)),
                        Span::styled(&self.number_input, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                        Span::styled(" (press g/G to jump, Esc to cancel)", Style::default().fg(Color::Gray)),
                    ]));
                }
            },
            InputMode::Filter => {
                match &self.filter_input_state {
                    FilterInputState::SelectingTag => {
                        if self.available_tags.is_empty() {
                            lines.push(Line::from(vec![
                                Span::styled("No tags available", Style::default().fg(Color::Red)),
                                Span::styled(" | Esc:cancel", Style::default().fg(Color::Gray)),
                            ]));
                        } else {
                            // Step 1: Show available tags
                            let mut filter_line = vec![
                                Span::styled("Filter Tags: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                            ];
                            
                            for (i, tag) in self.available_tags.iter().take(9).enumerate() {
                                if i > 0 { filter_line.push(Span::raw(" | ")); }
                                let tag_name = if tag.name.len() > 10 {
                                    format!("{}:{}", i + 1, &tag.name[..10])
                                } else {
                                    format!("{}:{}", i + 1, tag.name)
                                };
                                filter_line.push(Span::styled(
                                    tag_name,
                                    Style::default().fg(Color::Cyan)
                                ));
                            }
                            
                            filter_line.push(Span::styled(" | c:clear | Esc:cancel", Style::default().fg(Color::Gray)));
                            lines.push(Line::from(filter_line));
                        }
                    }
                    FilterInputState::SelectingAction(tag) => {
                        // Step 2: Show include/exclude options for selected tag
                        lines.push(Line::from(vec![
                            Span::styled("Filter \"", Style::default().fg(Color::Yellow)),
                            Span::styled(&tag.name, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                            Span::styled("\": ", Style::default().fg(Color::Yellow)),
                            Span::styled("e", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                            Span::styled(":exclude (hide) | ", Style::default().fg(Color::Gray)),
                            Span::styled("i", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                            Span::styled(":include (show only) | ", Style::default().fg(Color::Gray)),
                            Span::styled("Esc:back", Style::default().fg(Color::Gray)),
                        ]));
                    }
                }
            },
            InputMode::Normal => {},
        }
        
        let footer = Paragraph::new(lines)
            .block(Block::default().borders(Borders::ALL).title("Controls"));
        f.render_widget(footer, area);
    }
}