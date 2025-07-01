use crate::app::App;
use crate::ui::Theme;
use ratatui::{
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

pub struct EditorView {
    theme: Theme,
}

impl EditorView {
    pub fn new(theme: Theme) -> Self {
        Self { theme }
    }

    pub fn render(&self, frame: &mut Frame, app: &App, area: Rect) {
        let buffer = app.current_buffer();
        let cursor = &app.cursor;

        let viewport_height = area.height as usize;
        let viewport_width = area.width as usize;

        let line_number_width = if app.config.editor.line_numbers || app.config.editor.relative_line_numbers {
            if app.config.editor.relative_line_numbers {
                5
            } else {
                4
            }
        } else {
            0
        };

        let content_width = viewport_width.saturating_sub(line_number_width);
        let horizontal_scroll = app.get_horizontal_scroll_offset();

        let scroll_offset = app.config.editor.scroll_offset;

        let start_line = if cursor.line >= scroll_offset {
            (cursor.line - scroll_offset).min(buffer.line_count().saturating_sub(viewport_height))
        } else {
            0
        };

        let end_line = (start_line + viewport_height).min(buffer.line_count());

        let mut lines = Vec::new();

        for line_idx in start_line..end_line {
            let line_content = buffer.line(line_idx).unwrap_or_default();

            let line_number = if app.config.editor.relative_line_numbers {
                if line_idx == cursor.line {
                    if app.config.editor.line_numbers {
                        format!("{:4} ", line_idx + 1)
                    } else {
                        format!("{:4} ", 0)
                    }
                } else {
                    let relative_distance = if line_idx > cursor.line {
                        line_idx - cursor.line
                    } else {
                        cursor.line - line_idx
                    };
                    format!("{:4} ", relative_distance)
                }
            } else if app.config.editor.line_numbers {
                format!("{:4} ", line_idx + 1)
            } else {
                String::new()
            };

            let style = if line_idx == cursor.line {
                Style::default().bg(self.theme.current_line)
            } else {
                Style::default()
            };

            let displayed_content = if !app.config.editor.wrap_lines {
                let start_col = horizontal_scroll;
                let end_col = (start_col + content_width).min(line_content.len());
                if start_col < line_content.len() {
                    line_content[start_col..end_col].to_string()
                } else {
                    String::new()
                }
            } else {
                line_content
            };

            let spans = if line_number.is_empty() {
                vec![Span::styled(displayed_content, style)]
            } else {
                vec![
                    Span::styled(line_number, Style::default().fg(self.theme.line_number)),
                    Span::styled(displayed_content, style),
                ]
            };

            lines.push(Line::from(spans));
        }

        let paragraph = if app.config.editor.wrap_lines {
            Paragraph::new(lines)
                .style(Style::default().fg(self.theme.foreground).bg(self.theme.background))
                .block(Block::default().borders(Borders::NONE))
                .wrap(Wrap { trim: false })
        } else {
            Paragraph::new(lines)
                .style(Style::default().fg(self.theme.foreground).bg(self.theme.background))
                .block(Block::default().borders(Borders::NONE))
        };

        frame.render_widget(paragraph, area);
    }
}
