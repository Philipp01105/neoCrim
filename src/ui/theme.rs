use ratatui::style::{Color, Style};

#[derive(Debug, Clone)]
pub struct Theme {
    pub background: Color,
    pub foreground: Color,
    pub cursor: Color,
    pub selection: Color,
    pub line_number: Color,
    pub current_line: Color,
    pub status_bg: Color,
    pub status_fg: Color,
    pub command_bg: Color,
    pub command_fg: Color,
    pub terminal_border: Style,
    pub terminal_title: Style,
    pub terminal_background: Style,
    pub terminal_command: Style,
    pub terminal_output: Style,
    pub terminal_error: Style,
    pub terminal_running: Style,
    pub scrollbar: Style,
}

impl Theme {
    pub fn default_dark() -> Self {
        Self {
            background: Color::Rgb(30, 30, 30),      // #1e1e1e
            foreground: Color::Rgb(212, 212, 212),   // #d4d4d4
            cursor: Color::Rgb(255, 255, 0),         // #ffff00
            selection: Color::Rgb(38, 79, 120),      // #264f78
            line_number: Color::Rgb(133, 133, 133),  // #858585
            current_line: Color::Rgb(45, 45, 45),    // #2d2d2d
            status_bg: Color::Rgb(0, 120, 215),      // #0078d7
            status_fg: Color::Rgb(255, 255, 255),    // #ffffff
            command_bg: Color::Rgb(30, 30, 30),      // #1e1e1e
            command_fg: Color::Rgb(212, 212, 212),   // #d4d4d4
            terminal_border: Style::default().fg(Color::Rgb(0, 120, 215)),
            terminal_title: Style::default().fg(Color::Rgb(255, 255, 255)),
            terminal_background: Style::default().bg(Color::Rgb(30, 30, 30)).fg(Color::Rgb(212, 212, 212)),
            terminal_command: Style::default().fg(Color::Rgb(0, 255, 0)),
            terminal_output: Style::default().fg(Color::Rgb(212, 212, 212)),
            terminal_error: Style::default().fg(Color::Rgb(255, 85, 85)),
            terminal_running: Style::default().fg(Color::Rgb(255, 165, 0)),
            scrollbar: Style::default().fg(Color::Rgb(133, 133, 133)),
        }
    }

    pub fn default_light() -> Self {
        Self {
            background: Color::Rgb(255, 255, 255),   // #ffffff
            foreground: Color::Rgb(0, 0, 0),         // #000000
            cursor: Color::Rgb(0, 0, 255),           // #0000ff
            selection: Color::Rgb(173, 216, 230),    // #add8e6
            line_number: Color::Rgb(128, 128, 128),  // #808080
            current_line: Color::Rgb(245, 245, 245), // #f5f5f5
            status_bg: Color::Rgb(0, 120, 215),      // #0078d7
            status_fg: Color::Rgb(255, 255, 255),    // #ffffff
            command_bg: Color::Rgb(255, 255, 255),   // #ffffff
            command_fg: Color::Rgb(0, 0, 0),         // #000000
            terminal_border: Style::default().fg(Color::Rgb(0, 120, 215)),
            terminal_title: Style::default().fg(Color::Rgb(0, 0, 0)),
            terminal_background: Style::default().bg(Color::Rgb(255, 255, 255)).fg(Color::Rgb(0, 0, 0)),
            terminal_command: Style::default().fg(Color::Rgb(0, 128, 0)),
            terminal_output: Style::default().fg(Color::Rgb(0, 0, 0)),
            terminal_error: Style::default().fg(Color::Rgb(220, 20, 60)),
            terminal_running: Style::default().fg(Color::Rgb(255, 140, 0)),
            scrollbar: Style::default().fg(Color::Rgb(128, 128, 128)),
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::default_dark()
    }
}
