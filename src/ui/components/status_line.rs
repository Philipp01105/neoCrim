use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};
use crate::app::App;
use crate::ui::Theme;

pub struct StatusLine {
    theme: Theme,
}

impl StatusLine {
    pub fn new(theme: Theme) -> Self {
        Self { theme }
    }

    pub fn render(&self, frame: &mut Frame, app: &App, area: Rect) {
        let buffer = app.current_buffer();
        let cursor = &app.cursor;

        let mode_text = format!(" {} ", app.mode.name());
        let file_info = if let Some(filename) = buffer.file_name() {
            format!(" {} {}", filename, if buffer.is_modified { "[+]" } else { "" })
        } else {
            " [No Name] ".to_string()
        };

        let cursor_info = format!(" {}:{} ", cursor.line + 1, cursor.col + 1);

        let spans = vec![
            Span::styled(mode_text, Style::default()
                .fg(Color::Black)
                .bg(Color::Yellow)
                .add_modifier(Modifier::BOLD)),
            Span::styled(file_info, Style::default().fg(self.theme.status_fg))
        ];

        let status_paragraph = Paragraph::new(Line::from(spans))
            .style(Style::default().bg(self.theme.status_bg));

        frame.render_widget(status_paragraph, area);

        let cursor_area = Rect {
            x: area.x + area.width.saturating_sub(cursor_info.len() as u16),
            y: area.y,
            width: cursor_info.len() as u16,
            height: 1,
        };

        let cursor_paragraph = Paragraph::new(cursor_info)
            .style(Style::default().fg(self.theme.status_fg).bg(self.theme.status_bg));

        frame.render_widget(cursor_paragraph, cursor_area);
    }
}
