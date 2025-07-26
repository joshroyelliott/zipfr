use crate::analyzer::WordCount;
use crate::tui::ChartWidget;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};

#[derive(Debug, Clone, PartialEq)]
pub enum ZipfMode {
    Off,
    Absolute,  // Based on global rank 1
    Relative,  // Based on visible range
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
}

impl App {
    pub fn new(
        word_counts: Vec<WordCount>, 
        total_words: usize, 
        unique_words: usize,
        parse_duration: Duration,
        analyze_duration: Duration,
        total_duration: Duration,
    ) -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));
        
        Self {
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
        }
    }

    fn update_selection(&mut self, new_index: usize) {
        self.selected_index = new_index;
        self.list_state.select(Some(new_index));
    }

    pub fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> io::Result<()> {
        loop {
            terminal.draw(|f| self.ui(f))?;

            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match (key.code, key.modifiers) {
                        (KeyCode::Char('q'), _) => {
                            self.should_quit = true;
                            break;
                        }
                        // Basic movement
                        (KeyCode::Down, _) | (KeyCode::Char('j'), _) => {
                            if self.selected_index < self.word_counts.len().saturating_sub(1) {
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
                        // Number input for line jumping
                        (KeyCode::Char(c), _) if c.is_ascii_digit() => {
                            self.number_input.push(c);
                        }
                        (KeyCode::Esc, _) => {
                            // Clear number input on escape
                            self.number_input.clear();
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
                        _ => {}
                    }
                }
            }
        }
        Ok(())
    }

    fn ui(&mut self, f: &mut Frame) {
        let mut footer_height = 3; // Base height
        if self.log_scale || self.zipf_mode != ZipfMode::Off { footer_height += 1; } // Chart status line
        if !self.number_input.is_empty() { footer_height += 1; } // Number input line
        
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(4),
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
                Span::styled("Zipfian Text Analysis", Style::default().fg(Color::Cyan)),
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
        .block(Block::default().borders(Borders::ALL).title("Statistics"));
        f.render_widget(header, area);
    }

    fn render_word_list(&mut self, f: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = self
            .word_counts
            .iter()
            .map(|word_count| {
                ListItem::new(Line::from(vec![
                    Span::styled(format!("{:4}", word_count.rank), Style::default().fg(Color::Blue)),
                    Span::raw(" | "),
                    Span::styled(format!("{:15}", word_count.word), Style::default()),
                    Span::raw(" | "),
                    Span::styled(format!("{:6}", word_count.count), Style::default().fg(Color::Magenta)),
                ]))
            })
            .collect();

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Word Frequencies"))
            .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD));

        f.render_stateful_widget(list, area, &mut self.list_state);
    }

    fn render_chart(&self, f: &mut Frame, area: Rect) {
        // Calculate visible range based on current selection and visible area
        let visible_start = self.selected_index.saturating_sub(self.visible_area_height / 2);
        let visible_end = (visible_start + self.visible_area_height).min(self.word_counts.len());
        
        // Get the visible slice of word counts
        let visible_words = &self.word_counts[visible_start..visible_end];
        
        ChartWidget::render_enhanced(
            f, 
            area, 
            visible_words, 
            &self.word_counts, // Pass full word counts for absolute Zipf calculation
            self.log_scale, 
            &self.zipf_mode,
            self.selected_index,
            visible_start
        );
    }

    fn render_footer(&self, f: &mut Frame, area: Rect) {
        let mut lines = vec![
            Line::from("Navigation: j/k/↑/↓ | g/G/[num]g | Ctrl+u/d/b/f | Chart: L(log) Z(zipf) | q(quit)")
        ];
        
        // Show current chart modes
        let mut chart_status = Vec::new();
        if self.log_scale {
            chart_status.push(Span::styled("LOG", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));
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
        if !chart_status.is_empty() {
            let mut status_line = vec![Span::styled("Chart modes: ", Style::default().fg(Color::Gray))];
            status_line.extend(chart_status);
            lines.push(Line::from(status_line));
        }
        
        if !self.number_input.is_empty() {
            lines.push(Line::from(vec![
                Span::styled("Line: ", Style::default().fg(Color::Yellow)),
                Span::styled(&self.number_input, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::styled(" (press g/G to jump, Esc to cancel)", Style::default().fg(Color::Gray)),
            ]));
        }
        
        let footer = Paragraph::new(lines)
            .block(Block::default().borders(Borders::ALL).title("Controls"));
        f.render_widget(footer, area);
    }
}