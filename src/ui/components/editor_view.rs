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
        let line_number_width = if app.config.editor.line_numbers { 5 } else { 0 };
        let _content_width = viewport_width.saturating_sub(line_number_width);

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
            let line_number = if app.config.editor.line_numbers {
                format!("{:4} ", line_idx + 1)
            } else {
                String::new()
            };

            let style = if line_idx == cursor.line {
                Style::default().bg(self.theme.current_line)
            } else {
                Style::default()
            };

            let spans = vec![
                Span::styled(line_number, Style::default().fg(self.theme.line_number)),
                Span::styled(line_content, style),
            ];

            lines.push(Line::from(spans));
        }

        let paragraph = Paragraph::new(lines)
            .style(Style::default().fg(self.theme.foreground).bg(self.theme.background))
            .block(Block::default().borders(Borders::NONE))
            .wrap(Wrap { trim: false });

        frame.render_widget(paragraph, area);
    }
}
