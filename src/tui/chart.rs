use crate::analyzer::WordCount;
use crate::tui::app::ZipfMode;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    symbols,
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType},
    Frame,
};

pub struct ChartWidget;

impl ChartWidget {
    pub fn render(f: &mut Frame, area: Rect, word_counts: &[WordCount], max_items: usize) {
        let visible_words = &word_counts[..max_items.min(word_counts.len())];
        Self::render_enhanced(f, area, visible_words, word_counts, false, &ZipfMode::Off, 0, 0);
    }

    pub fn render_enhanced(
        f: &mut Frame, 
        area: Rect, 
        visible_words: &[WordCount],
        all_words: &[WordCount], 
        log_scale: bool, 
        zipf_mode: &ZipfMode,
        selected_index: usize,
        visible_start: usize
    ) {
        if visible_words.is_empty() {
            return;
        }

        // Prepare actual data
        let data: Vec<(f64, f64)> = visible_words
            .iter()
            .map(|wc| {
                let x = wc.rank as f64;
                let y = if log_scale { 
                    (wc.count as f64).ln().max(0.1) // Avoid log(0)
                } else { 
                    wc.count as f64 
                };
                (x, y)
            })
            .collect();

        // Prepare Zipf data based on mode
        let zipf_data: Vec<(f64, f64)> = match zipf_mode {
            ZipfMode::Off => Vec::new(),
            ZipfMode::Absolute => {
                // Use global rank 1 word as reference
                if let Some(global_first) = all_words.first() {
                    let global_first_freq = global_first.count as f64;
                    visible_words
                        .iter()
                        .map(|wc| {
                            let x = wc.rank as f64;
                            let ideal_freq = global_first_freq / x;
                            let y = if log_scale { 
                                ideal_freq.ln().max(0.1)
                            } else { 
                                ideal_freq 
                            };
                            (x, y)
                        })
                        .collect()
                } else {
                    Vec::new()
                }
            },
            ZipfMode::Relative => {
                // Use first visible word as reference
                if let Some(visible_first) = visible_words.first() {
                    let visible_first_freq = visible_first.count as f64;
                    let visible_first_rank = visible_first.rank as f64;
                    let constant = visible_first_freq * visible_first_rank;
                    
                    visible_words
                        .iter()
                        .map(|wc| {
                            let x = wc.rank as f64;
                            let ideal_freq = constant / x;
                            let y = if log_scale { 
                                ideal_freq.ln().max(0.1)
                            } else { 
                                ideal_freq 
                            };
                            (x, y)
                        })
                        .collect()
                } else {
                    Vec::new()
                }
            }
        };

        let mut datasets = vec![Dataset::default()
            .name("Actual Frequency")
            .marker(symbols::Marker::Braille)
            .style(Style::default().fg(Color::Cyan))
            .graph_type(GraphType::Line)
            .data(&data)];

        // Add highlighted point for currently selected word
        let selected_relative_index = selected_index.saturating_sub(visible_start);
        let selected_data: Vec<(f64, f64)> = if selected_relative_index < visible_words.len() {
            let selected_word = &visible_words[selected_relative_index];
            vec![(
                selected_word.rank as f64,
                if log_scale { 
                    (selected_word.count as f64).ln().max(0.1)
                } else { 
                    selected_word.count as f64 
                }
            )]
        } else {
            Vec::new()
        };

        // Add idealized Zipf line if enabled (before selected word so it renders underneath)
        if !zipf_data.is_empty() {
            let zipf_name = match zipf_mode {
                ZipfMode::Absolute => "Absolute Zipf",
                ZipfMode::Relative => "Relative Zipf", 
                ZipfMode::Off => "Zipf", // Shouldn't happen
            };
            
            datasets.push(Dataset::default()
                .name(zipf_name)
                .marker(symbols::Marker::Dot)
                .style(Style::default().fg(Color::Red))
                .graph_type(GraphType::Line)
                .data(&zipf_data));
        }

        // Add selected word marker LAST so it renders on top of everything
        if !selected_data.is_empty() && selected_relative_index < visible_words.len() {
            let selected_word = &visible_words[selected_relative_index];
            datasets.push(Dataset::default()
                .name(format!("Selected: {}", selected_word.word))
                .marker(symbols::Marker::Block)
                .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
                .graph_type(GraphType::Scatter)
                .data(&selected_data));
        }

        // Calculate bounds
        let min_rank = visible_words.first().map(|wc| wc.rank as f64).unwrap_or(1.0);
        let max_rank = visible_words.last().map(|wc| wc.rank as f64).unwrap_or(1.0);
        
        let (min_freq, max_freq) = if log_scale {
            let min_count = visible_words.iter().map(|wc| wc.count).min().unwrap_or(1) as f64;
            let max_count = visible_words.iter().map(|wc| wc.count).max().unwrap_or(1) as f64;
            (min_count.ln().max(0.1), max_count.ln())
        } else {
            let max_count = visible_words.iter().map(|wc| wc.count).max().unwrap_or(1) as f64;
            (0.0, max_count)
        };

        // Create title with current mode indicators
        let mut title = "Zipf Distribution".to_string();
        if log_scale { title.push_str(" (Log Scale)"); }
        match zipf_mode {
            ZipfMode::Absolute => title.push_str(" + Absolute"),
            ZipfMode::Relative => title.push_str(" + Relative"),
            ZipfMode::Off => {},
        }

        let y_title = if log_scale { "Log Frequency" } else { "Frequency" };

        let chart = Chart::new(datasets)
            .block(
                Block::default()
                    .title(title)
                    .borders(Borders::ALL),
            )
            .x_axis(
                Axis::default()
                    .title("Rank")
                    .style(Style::default().fg(Color::Gray))
                    .bounds([min_rank, max_rank])
                    .labels(vec![
                        format!("{}", min_rank as usize).into(),
                        format!("{}", ((min_rank + max_rank) / 2.0) as usize).into(),
                        format!("{}", max_rank as usize).into(),
                    ]),
            )
            .y_axis(
                Axis::default()
                    .title(y_title)
                    .style(Style::default().fg(Color::Gray))
                    .bounds([min_freq, max_freq])
                    .labels(if log_scale {
                        vec![
                            format!("{:.1}", min_freq).into(),
                            format!("{:.1}", (min_freq + max_freq) / 2.0).into(),
                            format!("{:.1}", max_freq).into(),
                        ]
                    } else {
                        vec![
                            "0".into(),
                            format!("{}", (max_freq / 2.0) as usize).into(),
                            format!("{}", max_freq as usize).into(),
                        ]
                    }),
            );

        f.render_widget(chart, area);
    }
}