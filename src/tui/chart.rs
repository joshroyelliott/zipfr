use crate::analyzer::WordCount;
use crate::tui::app::{ZipfState, ZipfBasis, ZipfReference, ChartScope};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    symbols,
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType},
    Frame,
};

pub struct ChartWidget;

impl ChartWidget {
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
    pub fn render(f: &mut Frame, area: Rect, word_counts: &[WordCount], max_items: usize) {
        let visible_words = &word_counts[..max_items.min(word_counts.len())];
        Self::render_enhanced(f, area, visible_words, word_counts, word_counts, false, &ZipfState::new(), &ChartScope::Relative, 0, 0, None);
    }

    pub fn render_enhanced(
        f: &mut Frame, 
        area: Rect, 
        visible_words: &[WordCount],
        filtered_words: &[WordCount],
        original_words: &[WordCount], 
        log_scale: bool, 
        zipf_state: &ZipfState,
        chart_scope: &ChartScope,
        selected_index: usize,
        _visible_start: usize,
        selected_fit_ratio: Option<f64>
    ) {
        if visible_words.is_empty() {
            return;
        }

        // Choose data source based on chart scope (always use filtered data for plotting)
        let chart_words = match chart_scope {
            ChartScope::Relative => visible_words,
            ChartScope::Absolute => filtered_words,
        };

        // Prepare actual data
        let data: Vec<(f64, f64)> = chart_words
            .iter()
            .map(|wc| {
                let x = if log_scale {
                    (wc.rank as f64).ln().max(0.1) // log(rank), avoid log(0)
                } else {
                    wc.rank as f64
                };
                let y = if log_scale { 
                    (wc.count as f64).ln().max(0.1) // log(frequency), avoid log(0)
                } else { 
                    wc.count as f64 
                };
                (x, y)
            })
            .collect();

        // Prepare Zipf data based on state
        let zipf_data: Vec<(f64, f64)> = if !zipf_state.enabled {
            Vec::new()
        } else {
            // Choose reference dataset based on basis
            let reference_words = match zipf_state.basis {
                ZipfBasis::Filtered => filtered_words,
                ZipfBasis::Unfiltered => original_words,
            };
            
            // Calculate Zipf line based on reference type and scope
            match (&zipf_state.reference, chart_scope) {
                (ZipfReference::Absolute, _) => {
                    // Absolute reference: use reference dataset's first word as global reference
                    if let Some(global_first) = reference_words.first() {
                        let global_first_freq = global_first.count as f64;
                        chart_words
                            .iter()
                            .map(|wc| {
                                let rank = wc.rank as f64;
                                let ideal_freq = global_first_freq / rank;
                                
                                let x = if log_scale {
                                    rank.ln().max(0.1) // log(rank)
                                } else {
                                    rank
                                };
                                let y = if log_scale { 
                                    ideal_freq.ln().max(0.1) // log(ideal_freq)
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
                (ZipfReference::Relative, ChartScope::Relative) => {
                    // Relative reference in VISIBLE scope: use visible range with relative constant
                    if let Some(visible_first) = visible_words.first() {
                        let visible_first_freq = visible_first.count as f64;
                        let visible_first_rank = visible_first.rank as f64;
                        let constant = visible_first_freq * visible_first_rank;
                        
                        chart_words
                            .iter()
                            .map(|wc| {
                                let rank = wc.rank as f64;
                                let ideal_freq = constant / rank;
                                
                                let x = if log_scale {
                                    rank.ln().max(0.1) // log(rank)
                                } else {
                                    rank
                                };
                                let y = if log_scale { 
                                    ideal_freq.ln().max(0.1) // log(ideal_freq)
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
                (ZipfReference::Relative, ChartScope::Absolute) => {
                    // This shouldn't happen in ALL-DATA scope, fall back to absolute
                    if let Some(global_first) = reference_words.first() {
                        let global_first_freq = global_first.count as f64;
                        chart_words
                            .iter()
                            .map(|wc| {
                                let rank = wc.rank as f64;
                                let ideal_freq = global_first_freq / rank;
                                
                                let x = if log_scale {
                                    rank.ln().max(0.1) // log(rank)
                                } else {
                                    rank
                                };
                                let y = if log_scale { 
                                    ideal_freq.ln().max(0.1) // log(ideal_freq)
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
            }
        };

        let mut datasets = vec![Dataset::default()
            .name("Actual Frequency")
            .marker(symbols::Marker::Braille)
            .style(Style::default().fg(Color::Cyan))
            .graph_type(GraphType::Line)
            .data(&data)];

        // Add highlighted point for currently selected word
        let selected_data: Vec<(f64, f64)> = if selected_index < filtered_words.len() {
            let selected_word = &filtered_words[selected_index];
            let rank = selected_word.rank as f64;
            let freq = selected_word.count as f64;
            
            vec![(
                if log_scale { rank.ln().max(0.1) } else { rank },
                if log_scale { freq.ln().max(0.1) } else { freq }
            )]
        } else {
            Vec::new()
        };

        // Add idealized Zipf line if enabled (before selected word so it renders underneath)
        if !zipf_data.is_empty() {
            let zipf_name = match (&zipf_state.basis, &zipf_state.reference) {
                (ZipfBasis::Filtered, ZipfReference::Absolute) => "Filtered Zipf",
                (ZipfBasis::Filtered, ZipfReference::Relative) => "Filtered Relative Zipf",
                (ZipfBasis::Unfiltered, ZipfReference::Absolute) => "Unfiltered Zipf",
                (ZipfBasis::Unfiltered, ZipfReference::Relative) => "Unfiltered Relative Zipf",
            };
            
            datasets.push(Dataset::default()
                .name(zipf_name)
                .marker(symbols::Marker::Dot)
                .style(Style::default().fg(Color::Red))
                .graph_type(GraphType::Line)
                .data(&zipf_data));
        }

        // Add selected word marker LAST so it renders on top of everything
        if !selected_data.is_empty() && selected_index < filtered_words.len() {
            let selected_word = &filtered_words[selected_index];
            
            // Choose cursor color based on Zipf fit ratio if available
            let cursor_color = if let Some(fit_ratio) = selected_fit_ratio {
                Self::deviation_to_color(fit_ratio)
            } else {
                Color::Yellow // Default color when no fit ratio available
            };
            
            datasets.push(Dataset::default()
                .name(format!("Selected: {}", selected_word.word))
                .marker(symbols::Marker::Block)
                .style(Style::default().fg(cursor_color).add_modifier(Modifier::BOLD))
                .graph_type(GraphType::Scatter)
                .data(&selected_data));
        }

        // Calculate bounds
        let (min_rank, max_rank) = if log_scale {
            let min_r = chart_words.first().map(|wc| wc.rank as f64).unwrap_or(1.0);
            let max_r = chart_words.last().map(|wc| wc.rank as f64).unwrap_or(1.0);
            (min_r.ln().max(0.1), max_r.ln())
        } else {
            let min_r = chart_words.first().map(|wc| wc.rank as f64).unwrap_or(1.0);
            let max_r = chart_words.last().map(|wc| wc.rank as f64).unwrap_or(1.0);
            (min_r, max_r)
        };
        
        let (min_freq, max_freq) = if log_scale {
            let min_count = chart_words.iter().map(|wc| wc.count).min().unwrap_or(1) as f64;
            let max_count = chart_words.iter().map(|wc| wc.count).max().unwrap_or(1) as f64;
            (min_count.ln().max(0.1), max_count.ln())
        } else {
            let max_count = chart_words.iter().map(|wc| wc.count).max().unwrap_or(1) as f64;
            (0.0, max_count)
        };

        // Create title with current mode indicators
        let mut title = "Zipf Distribution".to_string();
        if log_scale { title.push_str(" (Log-Log Scale)"); }
        match chart_scope {
            ChartScope::Absolute => title.push_str(" [All Data]"),
            ChartScope::Relative => title.push_str(" [Visible Range]"),
        }
        if zipf_state.enabled {
            let suffix = match (&zipf_state.basis, &zipf_state.reference) {
                (ZipfBasis::Filtered, ZipfReference::Absolute) => " + Filtered",
                (ZipfBasis::Filtered, ZipfReference::Relative) => " + Filtered Relative",
                (ZipfBasis::Unfiltered, ZipfReference::Absolute) => " + Unfiltered",
                (ZipfBasis::Unfiltered, ZipfReference::Relative) => " + Unfiltered Relative",
            };
            title.push_str(suffix);
        }

        let chart = Chart::new(datasets)
            .block(
                Block::default()
                    .title(title)
                    .borders(Borders::ALL),
            )
            .x_axis(
                Axis::default()
                    .title(if log_scale { "Log Rank" } else { "Rank" })
                    .style(Style::default().fg(Color::Gray))
                    .bounds([min_rank, max_rank])
                    .labels(if log_scale {
                        vec![
                            format!("{:.1}", min_rank).into(),
                            format!("{:.1}", (min_rank + max_rank) / 2.0).into(),
                            format!("{:.1}", max_rank).into(),
                        ]
                    } else {
                        vec![
                            format!("{}", min_rank as usize).into(),
                            format!("{}", ((min_rank + max_rank) / 2.0) as usize).into(),
                            format!("{}", max_rank as usize).into(),
                        ]
                    }),
            )
            .y_axis(
                Axis::default()
                    .title(if log_scale { "Log Frequency" } else { "Frequency" })
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