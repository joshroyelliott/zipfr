use crate::analyzer::{WordCount, Tag, Dataset};
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
pub struct FilterSet {
    pub exclude_tags: Vec<Tag>,
    pub include_only_tags: Vec<Tag>,  // OR logic - match ANY of these
    pub exclude_single: bool,
}

impl FilterSet {
    fn new() -> Self {
        Self {
            exclude_tags: Vec::new(),
            include_only_tags: Vec::new(),
            exclude_single: false,
        }
    }
    
    fn is_empty(&self) -> bool {
        self.exclude_tags.is_empty() && self.include_only_tags.is_empty() && !self.exclude_single
    }
    
    fn matches(&self, word_count: &WordCount) -> bool {
        // 1. Exclude singles check
        if self.exclude_single && word_count.count == 1 {
            return false;
        }
        
        // 2. Exclude tags check (exclude if word has ANY excluded tag)
        if self.exclude_tags.iter().any(|tag| word_count.tags.contains(tag)) {
            return false;
        }
        
        // 3. Include only tags check (OR logic - include if word has ANY include tag, or if no include filters)
        if !self.include_only_tags.is_empty() {
            return self.include_only_tags.iter().any(|tag| word_count.tags.contains(tag));
        }
        
        true
    }
    
    // Conflict prevention methods
    fn add_exclude_tag(&mut self, tag: Tag) {
        // Remove from include list if present (prevent conflicts)
        self.include_only_tags.retain(|t| t != &tag);
        // Add to exclude list if not already present
        if !self.exclude_tags.contains(&tag) {
            self.exclude_tags.push(tag);
        }
    }
    
    fn add_include_tag(&mut self, tag: Tag) {
        // Remove from exclude list if present (prevent conflicts)
        self.exclude_tags.retain(|t| t != &tag);
        // Add to include list if not already present
        if !self.include_only_tags.contains(&tag) {
            self.include_only_tags.push(tag);
        }
    }
    
    
    fn clear(&mut self) {
        self.exclude_tags.clear();
        self.include_only_tags.clear();
        self.exclude_single = false;
    }
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

#[derive(Debug, Clone, PartialEq)]
pub enum NormalizationMode {
    Raw,        // Show raw counts (default)
    Percentage, // Show as percentage of total words
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
    pub datasets: Vec<Dataset>,
    pub active_dataset_index: usize,
    pub visible_dataset_start: usize,
    pub chart_mode: bool,
    pub word_counts: Vec<WordCount>, // Current active dataset's word counts
    pub filtered_word_counts: Vec<WordCount>, // Current active dataset's filtered words
    pub per_dataset_filtered_words: Vec<Vec<WordCount>>, // Cached filtered words for each dataset
    pub per_dataset_list_states: Vec<ListState>, // Position memory for each dataset
    pub selected_index: usize,
    pub should_quit: bool,
    pub total_words: usize, // Current active dataset's stats
    pub unique_words: usize,
    pub parse_duration: Duration,
    pub analyze_duration: Duration,
    pub total_duration: Duration,
    pub visible_area_height: usize,
    pub number_input: String,
    pub list_state: ListState, // Active dataset's list state (reference to per_dataset_list_states)
    pub log_scale: bool,
    pub zipf_mode: ZipfMode,
    pub chart_scope: ChartScope,
    pub normalization_mode: NormalizationMode,
    // Global filter state that applies to all datasets
    pub filter_set: FilterSet,
    pub filter_dirty: bool,
    pub available_tags: Vec<Tag>,
    pub filter_input_state: FilterInputState,
    pub input_mode: InputMode,
    // Global search state that applies to active dataset
    pub search_query: String,
    pub search_results: Vec<usize>,
    pub current_search_index: usize,
}

impl App {
    // Safe Unicode-aware string truncation
    fn truncate_string(s: &str, max_chars: usize) -> String {
        if s.chars().count() <= max_chars {
            s.to_string()
        } else {
            s.chars().take(max_chars.saturating_sub(3)).collect::<String>() + "..."
        }
    }

    pub fn new(
        datasets: Vec<Dataset>,
        total_duration: Duration,
    ) -> Self {
        let word_counts = datasets[0].word_counts.clone();
        let total_words = datasets[0].total_words;
        let unique_words = datasets[0].unique_words;
        let parse_duration = datasets[0].parse_duration;
        let analyze_duration = datasets[0].analyze_duration;
        
        // Initialize per-dataset list states
        let mut per_dataset_list_states: Vec<ListState> = Vec::new();
        for _ in 0..datasets.len() {
            let mut list_state = ListState::default();
            list_state.select(Some(0));
            per_dataset_list_states.push(list_state);
        }
        
        // Initialize per-dataset filtered words (initially same as original)
        let per_dataset_filtered_words: Vec<Vec<WordCount>> = datasets
            .iter()
            .map(|dataset| dataset.word_counts.clone())
            .collect();
        
        let mut list_state = ListState::default();
        list_state.select(Some(0));
        
        // Extract available tags from all datasets
        let mut available_tags: Vec<Tag> = Vec::new();
        for dataset in &datasets {
            for word_count in &dataset.word_counts {
                for tag in &word_count.tags {
                    if !available_tags.contains(tag) {
                        available_tags.push(tag.clone());
                    }
                }
            }
        }

        let chart_mode = datasets.len() == 1; // Default to chart mode for single dataset
        
        let mut app = Self {
            datasets,
            active_dataset_index: 0,
            visible_dataset_start: 0,
            chart_mode,
            filtered_word_counts: word_counts.clone(),
            word_counts,
            per_dataset_filtered_words,
            per_dataset_list_states,
            selected_index: 0,
            should_quit: false,
            total_words,
            unique_words,
            parse_duration,
            analyze_duration,
            total_duration,
            visible_area_height: 20,
            number_input: String::new(),
            list_state,
            log_scale: false,
            zipf_mode: ZipfMode::Off,
            chart_scope: ChartScope::Relative,
            normalization_mode: NormalizationMode::Raw,
            filter_set: FilterSet::new(),
            filter_dirty: false,
            available_tags,
            filter_input_state: FilterInputState::SelectingTag,
            input_mode: InputMode::Normal,
            search_query: String::new(),
            search_results: Vec::new(),
            current_search_index: 0,
        };
        
        // Initialize all datasets with no filter (synchronized state)
        app.apply_current_filter_to_all_datasets();
        
        app
    }

    fn update_selection(&mut self, new_index: usize) {
        self.selected_index = new_index;
        self.list_state.select(Some(new_index));
        
        // Also update the stored list state for the current dataset
        if self.active_dataset_index < self.per_dataset_list_states.len() {
            self.per_dataset_list_states[self.active_dataset_index].select(Some(new_index));
        }
    }

    fn switch_to_dataset(&mut self, dataset_index: usize) {
        if dataset_index < self.datasets.len() {
            // Save current dataset's list state
            if self.active_dataset_index < self.per_dataset_list_states.len() {
                self.per_dataset_list_states[self.active_dataset_index] = self.list_state.clone();
            }
            
            self.active_dataset_index = dataset_index;
            let active_dataset = &self.datasets[dataset_index];
            
            // Update current dataset state
            self.word_counts = active_dataset.word_counts.clone();
            self.total_words = active_dataset.total_words;
            self.unique_words = active_dataset.unique_words;
            self.parse_duration = active_dataset.parse_duration;
            self.analyze_duration = active_dataset.analyze_duration;
            
            // Restore this dataset's list state (position memory)
            if dataset_index < self.per_dataset_list_states.len() {
                self.list_state = self.per_dataset_list_states[dataset_index].clone();
                self.selected_index = self.list_state.selected().unwrap_or(0);
            } else {
                self.selected_index = 0;
                self.list_state.select(Some(0));
            }
            
            // Use cached filtered words or apply filter if dirty
            if self.filter_dirty || dataset_index >= self.per_dataset_filtered_words.len() {
                self.apply_current_filter_to_all_datasets();
            }
            
            // Get filtered words for this dataset
            if dataset_index < self.per_dataset_filtered_words.len() {
                self.filtered_word_counts = self.per_dataset_filtered_words[dataset_index].clone();
            } else {
                self.filtered_word_counts = self.word_counts.clone();
            }
            
            // Update search results for the new dataset
            self.update_search_results();
        }
    }

    fn next_dataset(&mut self) {
        let next_index = (self.active_dataset_index + 1) % self.datasets.len();
        self.switch_to_dataset(next_index);
        
        // Update visible window if needed
        if !self.chart_mode {
            self.update_visible_datasets_for_tab();
        }
    }

    fn prev_dataset(&mut self) {
        let prev_index = if self.active_dataset_index == 0 {
            self.datasets.len() - 1
        } else {
            self.active_dataset_index - 1
        };
        self.switch_to_dataset(prev_index);
        
        // Update visible window if needed
        if !self.chart_mode {
            self.update_visible_datasets_for_tab();
        }
    }

    fn update_visible_datasets_for_tab(&mut self) {
        let max_visible = 4.min(self.datasets.len());
        
        // Ensure active dataset is visible
        if self.active_dataset_index < self.visible_dataset_start {
            self.visible_dataset_start = self.active_dataset_index;
        } else if self.active_dataset_index >= self.visible_dataset_start + max_visible {
            self.visible_dataset_start = self.active_dataset_index - max_visible + 1;
        }
    }

    fn toggle_chart_mode(&mut self) {
        self.chart_mode = !self.chart_mode;
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

        // Find all matching words with scores in filtered words
        let mut matches: Vec<(usize, f32)> = self.filtered_word_counts
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
                        let active_words_len = self.filtered_word_counts.len();
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
                        self.filter_set.clear();
                        self.apply_current_filter_to_all_datasets();
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
                        self.filter_set.add_exclude_tag(tag.clone());
                        self.apply_current_filter_to_all_datasets();
                        self.input_mode = InputMode::Normal;
                    }
                    KeyCode::Char('i') => {
                        // Include only this tag
                        self.filter_set.add_include_tag(tag.clone());
                        self.apply_current_filter_to_all_datasets();
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
                            let active_words_len = self.filtered_word_counts.len();
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
                                        .min(self.filtered_word_counts.len().saturating_sub(1));
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
                                        .min(self.filtered_word_counts.len().saturating_sub(1));
                                    self.update_selection(new_index);
                                }
                                self.number_input.clear();
                            } else {
                                self.update_selection(self.filtered_word_counts.len().saturating_sub(1));
                            }
                        }
                        (KeyCode::Char('d'), KeyModifiers::CONTROL) => {
                            // Ctrl+d - half page down
                            let half_page = self.visible_area_height / 2;
                            let new_index = (self.selected_index + half_page)
                                .min(self.filtered_word_counts.len().saturating_sub(1));
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
                                .min(self.filtered_word_counts.len().saturating_sub(1));
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
                            if self.selected_index < self.filtered_word_counts.len().saturating_sub(1) {
                                self.update_selection(self.selected_index + 1);
                            }
                        }
                        // Traditional keys
                        (KeyCode::Home, _) => self.update_selection(0),
                        (KeyCode::End, _) => {
                            self.update_selection(self.filtered_word_counts.len().saturating_sub(1));
                        }
                        (KeyCode::PageDown, _) => {
                            let full_page = self.visible_area_height;
                            let new_index = (self.selected_index + full_page)
                                .min(self.filtered_word_counts.len().saturating_sub(1));
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
                        (KeyCode::Char('%'), _) => {
                            self.normalization_mode = match self.normalization_mode {
                                NormalizationMode::Raw => NormalizationMode::Percentage,
                                NormalizationMode::Percentage => NormalizationMode::Raw,
                            };
                        }
                        // Multi-dataset controls
                        (KeyCode::Char('C'), _) => {
                            self.toggle_chart_mode();
                        }
                        (KeyCode::Tab, _) => {
                            self.next_dataset();
                        }
                        (KeyCode::BackTab, _) => {
                            self.prev_dataset();
                        }
                        (KeyCode::Char('['), _) => {
                            if self.chart_mode {
                                self.prev_dataset();
                            }
                        }
                        (KeyCode::Char(']'), _) => {
                            if self.chart_mode {
                                self.next_dataset();
                            }
                        }
                        (KeyCode::Char('S'), _) => {
                            // Toggle stop word filter
                            self.toggle_stopword_filter();
                        }
                        (KeyCode::Char('U'), _) => {
                            // Toggle single words filter
                            self.toggle_single_words_filter();
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
        if self.log_scale || self.zipf_mode != ZipfMode::Off || self.chart_scope != ChartScope::Relative || !self.filter_set.is_empty() {
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

        if self.chart_mode {
            // Chart mode: single dataset with chart
            let terminal_width = f.size().width;
            let word_list_width = if terminal_width > 120 { 25 } else if terminal_width > 80 { 30 } else { 35 };
            
            let content_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(word_list_width),
                    Constraint::Min(20),
                ])
                .split(main_chunks[1]);

            self.render_header(f, main_chunks[0]);
            self.render_word_list(f, content_chunks[0]);
            self.render_chart(f, content_chunks[1]);
        } else {
            // Multi-dataset mode: side-by-side datasets
            self.render_header(f, main_chunks[0]);
            self.render_multi_datasets(f, main_chunks[1]);
        }
        
        self.render_footer(f, main_chunks[2]);
    }

    fn render_header(&self, f: &mut Frame, area: Rect) {
        let title = if self.datasets.len() > 1 {
            if self.chart_mode {
                format!("Zipfr - {} (Dataset {} of {})", 
                    self.datasets[self.active_dataset_index].name,
                    self.active_dataset_index + 1,
                    self.datasets.len())
            } else {
                format!("Zipfr - Multi-Dataset Analysis ({} datasets)", self.datasets.len())
            }
        } else {
            format!("Zipfr - {}", self.datasets[0].name)
        };
        
        let words_per_sec = self.total_words as f64 / self.total_duration.as_secs_f64();
        
        // Calculate filtered vs original totals
        let original_total_words = self.word_counts.iter().map(|wc| wc.count).sum::<usize>();
        let filtered_total_words = self.filtered_word_counts.iter().map(|wc| wc.count).sum::<usize>();
        let original_unique_words = self.word_counts.len();
        let filtered_unique_words = self.filtered_word_counts.len();
        
        // Format displays based on filtering state
        let total_display = if !self.filter_set.is_empty() {
            let percentage = if original_total_words > 0 {
                (filtered_total_words as f64 / original_total_words as f64) * 100.0
            } else {
                0.0
            };
            format!("{}/{} ({:.0}%)", filtered_total_words, original_total_words, percentage)
        } else {
            format!("{}", self.total_words)
        };
        
        let unique_display = if !self.filter_set.is_empty() {
            let percentage = if original_unique_words > 0 {
                (filtered_unique_words as f64 / original_unique_words as f64) * 100.0
            } else {
                0.0
            };
            format!("{}/{} ({:.0}%)", filtered_unique_words, original_unique_words, percentage)
        } else {
            format!("{}", self.unique_words)
        };
        
        // Build the analysis line with inline filtering display
        let analysis_line = vec![
            Span::styled("Zipfian Text Analysis", Style::default().fg(Color::Gray)),
            Span::raw(" | "),
            Span::styled(
                format!("Total Words: {}", total_display),
                Style::default().fg(Color::Yellow),
            ),
            Span::raw(" | "),
            Span::styled(
                format!("Unique Words: {}", unique_display),
                Style::default().fg(Color::Green),
            ),
        ];
        
        let header = Paragraph::new(vec![
            Line::from(vec![
                Span::styled(&title, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(analysis_line),
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

    fn format_word_list_items(
        words: &[WordCount],
        search_results: &[usize],
        visible_words: &[WordCount],
        zipf_mode: &ZipfMode,
        normalization_mode: &NormalizationMode,
        total_words: usize,
        calculate_zipf_fit: impl Fn(&WordCount, &[WordCount]) -> Option<f64>,
    ) -> Vec<ListItem<'static>> {
        words
            .iter()
            .enumerate()
            .map(|(i, word_count)| {
                // Check if this word is a search match
                let is_search_match = search_results.contains(&i);
                let word_style = if is_search_match {
                    Style::default().bg(Color::DarkGray).fg(Color::Yellow)
                } else {
                    Style::default()
                };

                // Use unified chart view format for all contexts
                let count_display = match normalization_mode {
                    NormalizationMode::Raw => format!("{:6}", word_count.count),
                    NormalizationMode::Percentage => {
                        if total_words > 0 {
                            let percentage = (word_count.count as f64 / total_words as f64) * 100.0;
                            format!("{:5.1}%", percentage)
                        } else {
                            format!("{:6}", word_count.count)
                        }
                    }
                };
                
                let mut spans = vec![
                    Span::styled(format!("{:4}", word_count.rank), Style::default().fg(Color::Blue)),
                    Span::raw(" | "),
                    Span::styled(format!("{:12}", word_count.word), word_style),
                    Span::raw(" | "),
                    Span::styled(count_display, Style::default().fg(Color::Magenta)),
                ];

                // Add fit column if Zipf mode is active
                if *zipf_mode != ZipfMode::Off {
                    if let Some(fit_ratio) = calculate_zipf_fit(word_count, visible_words) {
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
            .collect()
    }

    fn render_word_list(&mut self, f: &mut Frame, area: Rect) {
        // Filtered words should already be up to date from global filter management
        
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

        // Create local copies to avoid borrow checker issues
        let filtered_word_counts = self.filtered_word_counts.clone();
        let search_results = self.search_results.clone();
        let zipf_mode = self.zipf_mode.clone();
        
        let items = Self::format_word_list_items(
            &filtered_word_counts,
            &search_results,
            visible_words,
            &zipf_mode,
            &self.normalization_mode,
            self.total_words,
            |word_count, visible_words| self.calculate_zipf_fit(word_count, visible_words),
        );

        // Create title with fit column indicator
        let title = if zipf_mode != ZipfMode::Off {
            match zipf_mode {
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



    fn apply_current_filter_to_all_datasets(&mut self) {
        // Apply the current filter to all datasets and cache the results
        for (dataset_index, dataset) in self.datasets.iter().enumerate() {
            let filtered_words = if self.filter_set.is_empty() {
                dataset.word_counts.clone()
            } else {
                dataset.word_counts.iter()
                    .filter(|wc| self.filter_set.matches(wc))
                    .cloned()
                    .collect()
            };

            // Re-rank the filtered words
            let mut ranked_words = filtered_words;
            for (index, word_count) in ranked_words.iter_mut().enumerate() {
                word_count.rank = index + 1;
            }

            // Store in cache
            if dataset_index < self.per_dataset_filtered_words.len() {
                self.per_dataset_filtered_words[dataset_index] = ranked_words;
            } else {
                self.per_dataset_filtered_words.push(ranked_words);
            }
        }

        // Update current dataset's filtered words
        if self.active_dataset_index < self.per_dataset_filtered_words.len() {
            self.filtered_word_counts = self.per_dataset_filtered_words[self.active_dataset_index].clone();
        }

        // Reset selection if it's out of bounds for current dataset
        if self.selected_index >= self.filtered_word_counts.len() {
            self.selected_index = 0;
            self.list_state.select(Some(0));
        }

        self.filter_dirty = false;
    }

    fn toggle_stopword_filter(&mut self) {
        if let Some(stopword_tag) = self.available_tags.iter().find(|tag| tag.name == "Stop Words") {
            if self.filter_set.exclude_tags.contains(stopword_tag) {
                self.filter_set.exclude_tags.retain(|t| t != stopword_tag);
            } else {
                self.filter_set.add_exclude_tag(stopword_tag.clone());
            }
            self.apply_current_filter_to_all_datasets();
        }
    }

    fn toggle_single_words_filter(&mut self) {
        self.filter_set.exclude_single = !self.filter_set.exclude_single;
        self.apply_current_filter_to_all_datasets();
    }

    fn render_multi_datasets(&mut self, f: &mut Frame, area: Rect) {
        let max_visible = 4.min(self.datasets.len());
        let visible_end = (self.visible_dataset_start + max_visible).min(self.datasets.len());
        
        // Create constraints for visible datasets
        let visible_count = visible_end - self.visible_dataset_start;
        let constraints: Vec<Constraint> = (0..visible_count)
            .map(|_| Constraint::Percentage(100 / visible_count as u16))
            .collect();
        
        let dataset_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(constraints)
            .split(area);
        
        // Render each visible dataset
        for i in 0..visible_count {
            let dataset_index = self.visible_dataset_start + i;
            let is_active = dataset_index == self.active_dataset_index;
            
            if is_active {
                // Clone the dataset to avoid borrow checker issues
                let dataset = self.datasets[dataset_index].clone();
                self.render_active_dataset_column(f, dataset_chunks[i], &dataset);
            } else {
                self.render_inactive_dataset_column(f, dataset_chunks[i], dataset_index);
            }
        }
    }
    
    fn render_active_dataset_column(&mut self, f: &mut Frame, area: Rect, dataset: &Dataset) {
        // Filtered words should already be up to date from global filter management
        
        let border_style = Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD);
        
        let title = Self::truncate_string(&dataset.name, 15);
        
        let block = Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(border_style);
        
        let inner_area = block.inner(area);
        f.render_widget(block, area);
        
        // Update visible area height for navigation
        self.visible_area_height = inner_area.height.saturating_sub(2) as usize;
        
        // Create local copies to avoid borrow checker issues
        let filtered_word_counts = self.filtered_word_counts.clone();
        let search_results = self.search_results.clone();
        
        // Use unified formatting for consistency
        let visible_words = &[]; // Empty for comparison view (no fit calculations needed)
        let zipf_mode = ZipfMode::Off; // No fit calculations in comparison view
        let items = Self::format_word_list_items(
            &filtered_word_counts,
            &search_results,
            visible_words,
            &zipf_mode,
            &self.normalization_mode,
            self.total_words,
            |_, _| None, // No fit calculations in comparison view
        );
        
        let list = List::new(items)
            .style(Style::default().fg(Color::White))
            .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD));
        
        f.render_stateful_widget(list, inner_area, &mut self.list_state);
    }

    fn render_inactive_dataset_column(&mut self, f: &mut Frame, area: Rect, dataset_index: usize) {
        let dataset = &self.datasets[dataset_index];
        let border_style = Style::default().fg(Color::Gray);
        
        let title = Self::truncate_string(&dataset.name, 15);
        
        let block = Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(border_style);
        
        let inner_area = block.inner(area);
        f.render_widget(block, area);
        
        // Use filtered words for this dataset if available
        let words_to_show = if dataset_index < self.per_dataset_filtered_words.len() {
            self.per_dataset_filtered_words[dataset_index].clone()
        } else {
            dataset.word_counts.clone()
        };
        
        // Use unified formatting for consistency
        let visible_words = &[]; // Empty for comparison view (no fit calculations needed)
        let empty_search_results = Vec::new(); // Inactive datasets don't show search highlights
        let zipf_mode = ZipfMode::Off; // No fit calculations in comparison view
        let items = Self::format_word_list_items(
            &words_to_show,
            &empty_search_results,
            visible_words,
            &zipf_mode,
            &self.normalization_mode,
            dataset.total_words,
            |_, _| None, // No fit calculations in comparison view
        );
        
        let list = List::new(items)
            .style(Style::default().fg(Color::White));
        
        // Use stateful widget with this dataset's list state to preserve scroll position
        if dataset_index < self.per_dataset_list_states.len() {
            f.render_stateful_widget(list, inner_area, &mut self.per_dataset_list_states[dataset_index]);
        } else {
            f.render_widget(list, inner_area);
        }
    }

    fn render_chart(&mut self, f: &mut Frame, area: Rect) {
        // Filtered words should already be up to date from global filter management
        
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
        let navigation_line = if self.datasets.len() > 1 {
            if self.chart_mode {
                "Navigation: j/k | g/G/[num]g | Ctrl+u/d/b/f | Chart: L(log) Z(zipf) A(scope) %(normalize) | Datasets: [/] | Mode: C(multi) | Filter: S(stopwords) U(single) F(filter) | /(search) n/N | q(quit)"
            } else {
                "Navigation: j/k | g/G/[num]g | Ctrl+u/d/b/f | Datasets: Tab/Shift+Tab | Mode: C(chart) | Display: %(normalize) | Filter: S(stopwords) U(single) F(filter) | /(search) n/N | q(quit)"
            }
        } else {
            "Navigation: j/k | g/G/[num]g | Ctrl+u/d/b/f | Chart: L(log) Z(zipf) A(scope) %(normalize) | Filter: S(stopwords) U(single) F(filter) | /(search) n/N | q(quit)"
        };
        
        let mut lines = vec![
            Line::from(navigation_line)
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
        
        // Add normalization mode indicator
        match self.normalization_mode {
            NormalizationMode::Percentage => {
                if !chart_status.is_empty() { chart_status.push(Span::raw(" | ")); }
                chart_status.push(Span::styled("NORMALIZED", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)));
            },
            NormalizationMode::Raw => {}, // Don't show anything for raw mode (default)
        }
        
        // Add filter status to the same line
        if !self.filter_set.is_empty() {
            if !chart_status.is_empty() { chart_status.push(Span::raw(" | ")); }
            chart_status.push(Span::styled("Filter: ", Style::default().fg(Color::Gray)));
            
            let mut filter_parts = Vec::new();
            
            // Add exclude filters
            if self.filter_set.exclude_single {
                filter_parts.push("Single Words".to_string());
            }
            for tag in &self.filter_set.exclude_tags {
                filter_parts.push(tag.name.clone());
            }
            
            if !filter_parts.is_empty() {
                chart_status.push(Span::styled("Excluding ", Style::default().fg(Color::Red)));
                chart_status.push(Span::styled(filter_parts.join(", "), Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)));
            }
            
            // Add include filters
            if !self.filter_set.include_only_tags.is_empty() {
                if !filter_parts.is_empty() {
                    chart_status.push(Span::raw(" | "));
                }
                let include_parts: Vec<String> = self.filter_set.include_only_tags.iter().map(|tag| tag.name.clone()).collect();
                chart_status.push(Span::styled("Only ", Style::default().fg(Color::Green)));
                chart_status.push(Span::styled(include_parts.join(", "), Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)));
            }
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