
pub mod help;

pub fn pad_to_length(input: &str, length: usize) -> String {
    format!("{:<width$}", input, width = length)
}
pub fn leftpad_to_length(input: &str, length: usize) -> String {
  format!("{:>width$}", input, width = length)
}


use ratatui::{prelude::*, widgets::*};

pub fn centered_rect(r: Rect, percent_x: u16, percent_y: u16) -> Rect {
  let popup_layout = Layout::default()
    .direction(Direction::Vertical)
    .constraints([
      Constraint::Percentage((100 - percent_y) / 2),
      Constraint::Percentage(percent_y),
      Constraint::Percentage((100 - percent_y) / 2),
    ])
    .split(r);

  Layout::default()
    .direction(Direction::Horizontal)
    .constraints([
    Constraint::Percentage((100 - percent_x) / 2),
    Constraint::Percentage(percent_x),
    Constraint::Percentage((100 - percent_x) / 2),
    ])
    .split(popup_layout[1])[1]
}

pub fn centered_rect_inner_fixed(r: Rect, x: u16, y: u16) -> Rect {
  let popup_layout = Layout::default()
    .direction(Direction::Vertical)
    .constraints([
      Constraint::Length((r.height.saturating_sub(y)) / 2), // Top padding to center
      Constraint::Length(y.saturating_add(2)), // Fixed height for the text area
      Constraint::Min(0), // Bottom padding (fill remaining)
    ])
    .split(r);

  Layout::default()
    .direction(Direction::Horizontal)
    .constraints([
      Constraint::Length((r.width.saturating_sub(x)) / 2), // Left padding to center
      Constraint::Length(x), // Fixed width for the text area
      Constraint::Min(0), // Right padding (fill remaining)
    ])
    .split(popup_layout[1])[1]
}
