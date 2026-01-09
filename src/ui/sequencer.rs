// src/ui/sequencer.rs
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn draw_sequencer(f: &mut Frame, pattern: &Pattern, current_step: usize) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),   // Header
            Constraint::Min(10),     // Sequencer grid
            Constraint::Length(3),   // Controls
        ])
        .split(f.area());

    // Header
    let header = Paragraph::new("🎵 Rust Rhythm Box")
        .style(Style::default().fg(Color::Cyan))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(header, chunks[0]);

    // Sequencer Grid
    draw_grid(f, chunks[1], pattern, current_step);
}

fn draw_grid(f: &mut Frame, area: ratatui::layout::Rect, pattern: &Pattern, current_step: usize) {
    // 16ステップ × 4トラックのグリッド表示
    // current_stepをハイライト
    // アクティブなステップは "●"、非アクティブは "○"
    
    let grid_text = pattern.steps.iter().enumerate().map(|(track_idx, track)| {
        track.iter().enumerate().map(|(step_idx, step)| {
            let symbol = if step.active { "●" } else { "○" };
            let style = if step_idx == current_step {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            (symbol, style)
        }).collect::<Vec<_>>()
    }).collect::<Vec<_>>();
    
    // TODO: 実際の描画処理
}