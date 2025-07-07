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

#[derive(Debug, Clone)]
pub struct EnhancedTheme {
    // Basic colors
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

    // Glass and transparency effects
    pub background_opacity: f32,
    pub blur_radius: f32,
    pub enable_glass: bool,

    // Additional colors
    pub accent_color: Color,
    pub border_color: Color,
    pub inactive_border: Color,
    pub highlight: Color,
    pub shadow: Color,
    pub error_color: Color,
    pub warning_color: Color,
    pub info_color: Color,
    pub success_color: Color,
}

impl Theme {
    pub fn default_dark() -> Self {
        Self {
            background: Color::Rgb(30, 30, 30),     // #1e1e1e
            foreground: Color::Rgb(212, 212, 212),  // #d4d4d4
            cursor: Color::Rgb(255, 255, 0),        // #ffff00
            selection: Color::Rgb(70, 130, 180),    // #4682b4 (Steel Blue - better contrast)
            line_number: Color::Rgb(133, 133, 133), // #858585
            current_line: Color::Rgb(45, 45, 45),   // #2d2d2d
            status_bg: Color::Rgb(0, 120, 215),     // #0078d7
            status_fg: Color::Rgb(255, 255, 255),   // #ffffff
            command_bg: Color::Rgb(30, 30, 30),     // #1e1e1e
            command_fg: Color::Rgb(212, 212, 212),  // #d4d4d4
            terminal_border: Style::default().fg(Color::Rgb(0, 120, 215)),
            terminal_title: Style::default().fg(Color::Rgb(255, 255, 255)),
            terminal_background: Style::default()
                .bg(Color::Rgb(30, 30, 30))
                .fg(Color::Rgb(212, 212, 212)),
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
            selection: Color::Rgb(0, 120, 215),      // #0078d7 (Blue - good contrast on white)
            line_number: Color::Rgb(128, 128, 128),  // #808080
            current_line: Color::Rgb(245, 245, 245), // #f5f5f5
            status_bg: Color::Rgb(0, 120, 215),      // #0078d7
            status_fg: Color::Rgb(255, 255, 255),    // #ffffff
            command_bg: Color::Rgb(255, 255, 255),   // #ffffff
            command_fg: Color::Rgb(0, 0, 0),         // #000000
            terminal_border: Style::default().fg(Color::Rgb(0, 120, 215)),
            terminal_title: Style::default().fg(Color::Rgb(0, 0, 0)),
            terminal_background: Style::default()
                .bg(Color::Rgb(255, 255, 255))
                .fg(Color::Rgb(0, 0, 0)),
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

impl EnhancedTheme {
    pub fn default_dark() -> Self {
        Self {
            background: Color::Rgb(30, 30, 30),
            foreground: Color::Rgb(212, 212, 212),
            cursor: Color::Rgb(255, 255, 0),
            selection: Color::Rgb(70, 130, 180),
            line_number: Color::Rgb(133, 133, 133),
            current_line: Color::Rgb(45, 45, 45),
            status_bg: Color::Rgb(0, 120, 215),
            status_fg: Color::Rgb(255, 255, 255),
            command_bg: Color::Rgb(30, 30, 30),
            command_fg: Color::Rgb(212, 212, 212),
            terminal_border: Style::default().fg(Color::Rgb(0, 120, 215)),
            terminal_title: Style::default().fg(Color::Rgb(255, 255, 255)),
            terminal_background: Style::default()
                .bg(Color::Rgb(30, 30, 30))
                .fg(Color::Rgb(212, 212, 212)),
            terminal_command: Style::default().fg(Color::Rgb(0, 255, 0)),
            terminal_output: Style::default().fg(Color::Rgb(212, 212, 212)),
            terminal_error: Style::default().fg(Color::Rgb(244, 67, 54)),
            terminal_running: Style::default().fg(Color::Rgb(255, 152, 0)),
            scrollbar: Style::default().fg(Color::Rgb(133, 133, 133)),

            background_opacity: 1.0,
            blur_radius: 0.0,
            enable_glass: false,

            accent_color: Color::Rgb(0, 120, 215),
            border_color: Color::Rgb(100, 100, 100),
            inactive_border: Color::Rgb(60, 60, 60),
            highlight: Color::Rgb(255, 255, 255),
            shadow: Color::Rgb(0, 0, 0),
            error_color: Color::Rgb(244, 67, 54),
            warning_color: Color::Rgb(255, 152, 0),
            info_color: Color::Rgb(33, 150, 243),
            success_color: Color::Rgb(76, 175, 80),
        }
    }

    pub fn to_legacy(&self) -> Theme {
        Theme {
            background: self.background,
            foreground: self.foreground,
            cursor: self.cursor,
            selection: self.selection,
            line_number: self.line_number,
            current_line: self.current_line,
            status_bg: self.status_bg,
            status_fg: self.status_fg,
            command_bg: self.command_bg,
            command_fg: self.command_fg,
            terminal_border: self.terminal_border,
            terminal_title: self.terminal_title,
            terminal_background: self.terminal_background,
            terminal_command: self.terminal_command,
            terminal_output: self.terminal_output,
            terminal_error: self.terminal_error,
            terminal_running: self.terminal_running,
            scrollbar: self.scrollbar,
        }
    }
}

impl Default for EnhancedTheme {
    fn default() -> Self {
        Self::default_dark()
    }
}
