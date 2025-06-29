use ratatui::{
    layout::Rect,
    style::Style,
    widgets::Paragraph,
    Frame,
};
use crate::app::App;
use crate::ui::Theme;

pub struct CommandLine {
    theme: Theme,
}

impl CommandLine {
    pub fn new(theme: Theme) -> Self {
        Self { theme }
    }

    pub fn render(&self, frame: &mut Frame, app: &App, area: Rect) {
        let content = if app.mode.is_command() {
            format!(":{}", app.command_line)
        } else if let Some(ref message) = app.status_message {
            message.clone()
        } else {
            String::new()
        };

        let paragraph = Paragraph::new(content)
            .style(Style::default().fg(self.theme.command_fg).bg(self.theme.command_bg));

        frame.render_widget(paragraph, area);
    }
}
