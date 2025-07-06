use crate::Result;
use anyhow::Context;
use ratatui::style::Color;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeoTheme {
    pub name: String,
    pub author: String,
    pub description: String,
    pub colors: ThemeColors,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeColors {
    // Editor colors
    pub background: ColorValue,
    pub foreground: ColorValue,
    pub cursor: ColorValue,
    pub selection: ColorValue,
    pub line_number: ColorValue,
    pub current_line: ColorValue,

    // Glass/Transparency effects
    #[serde(default = "default_opacity")]
    pub background_opacity: f32,
    #[serde(default = "default_blur")]
    pub blur_radius: f32,
    #[serde(default = "default_false")]
    pub enable_glass: bool,

    // Status bar colors
    pub status_bg: ColorValue,
    pub status_fg: ColorValue,
    pub command_bg: ColorValue,
    pub command_fg: ColorValue,

    // File explorer colors
    pub explorer_bg: ColorValue,
    pub explorer_fg: ColorValue,
    pub explorer_selected: ColorValue,
    pub explorer_directory: ColorValue,
    pub explorer_file: ColorValue,

    // Border and accent colors
    #[serde(default = "default_accent")]
    pub accent_color: ColorValue,
    #[serde(default = "default_border")]
    pub border_color: ColorValue,
    #[serde(default = "default_inactive_border")]
    pub inactive_border: ColorValue,
    #[serde(default = "default_highlight")]
    pub highlight: ColorValue,
    #[serde(default = "default_shadow")]
    pub shadow: ColorValue,

    // Syntax highlighting colors
    pub syntax_keyword: ColorValue,
    pub syntax_string: ColorValue,
    pub syntax_comment: ColorValue,
    pub syntax_function: ColorValue,
    pub syntax_type: ColorValue,
    pub syntax_constant: ColorValue,
    pub syntax_variable: ColorValue,
    pub syntax_number: ColorValue,
    pub syntax_operator: ColorValue,
    pub syntax_punctuation: ColorValue,

    // Additional syntax colors
    #[serde(default = "default_syntax_attribute")]
    pub syntax_attribute: ColorValue,
    #[serde(default = "default_syntax_special")]
    pub syntax_special: ColorValue,
    #[serde(default = "default_syntax_tag")]
    pub syntax_tag: ColorValue,
    #[serde(default = "default_syntax_link")]
    pub syntax_link: ColorValue,
    #[serde(default = "default_error")]
    pub error_color: ColorValue,
    #[serde(default = "default_warning")]
    pub warning_color: ColorValue,
    #[serde(default = "default_info")]
    pub info_color: ColorValue,
    #[serde(default = "default_success")]
    pub success_color: ColorValue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ColorValue {
    Rgba { r: u8, g: u8, b: u8, a: f32 },
    Rgb { r: u8, g: u8, b: u8 },
    Hex(String),
    Named(String),
}

fn default_opacity() -> f32 {
    1.0
}
fn default_blur() -> f32 {
    0.0
}
fn default_false() -> bool {
    false
}
fn default_accent() -> ColorValue {
    ColorValue::Rgb {
        r: 0,
        g: 120,
        b: 215,
    }
}
fn default_border() -> ColorValue {
    ColorValue::Rgb {
        r: 100,
        g: 100,
        b: 100,
    }
}
fn default_inactive_border() -> ColorValue {
    ColorValue::Rgb {
        r: 60,
        g: 60,
        b: 60,
    }
}
fn default_highlight() -> ColorValue {
    ColorValue::Rgb {
        r: 255,
        g: 255,
        b: 255,
    }
}
fn default_shadow() -> ColorValue {
    ColorValue::Rgb { r: 0, g: 0, b: 0 }
}
fn default_syntax_attribute() -> ColorValue {
    ColorValue::Rgb {
        r: 156,
        g: 220,
        b: 254,
    }
}
fn default_syntax_special() -> ColorValue {
    ColorValue::Rgb {
        r: 255,
        g: 215,
        b: 0,
    }
}
fn default_syntax_tag() -> ColorValue {
    ColorValue::Rgb {
        r: 86,
        g: 156,
        b: 214,
    }
}
fn default_syntax_link() -> ColorValue {
    ColorValue::Rgb {
        r: 86,
        g: 156,
        b: 214,
    }
}
fn default_error() -> ColorValue {
    ColorValue::Rgb {
        r: 244,
        g: 67,
        b: 54,
    }
}
fn default_warning() -> ColorValue {
    ColorValue::Rgb {
        r: 255,
        g: 152,
        b: 0,
    }
}
fn default_info() -> ColorValue {
    ColorValue::Rgb {
        r: 33,
        g: 150,
        b: 243,
    }
}
fn default_success() -> ColorValue {
    ColorValue::Rgb {
        r: 76,
        g: 175,
        b: 80,
    }
}

impl ColorValue {
    pub fn to_ratatui_color(&self) -> Color {
        match self {
            ColorValue::Rgba { r, g, b, a } => {
                if *a < 1.0 {
                    self.blend_with_background(*r, *g, *b, *a)
                } else {
                    Color::Rgb(*r, *g, *b)
                }
            }
            ColorValue::Rgb { r, g, b } => Color::Rgb(*r, *g, *b),
            ColorValue::Hex(hex) => {
                let hex = hex.trim_start_matches('#');
                if hex.len() == 6 {
                    if let (Ok(r), Ok(g), Ok(b)) = (
                        u8::from_str_radix(&hex[0..2], 16),
                        u8::from_str_radix(&hex[2..4], 16),
                        u8::from_str_radix(&hex[4..6], 16),
                    ) {
                        return Color::Rgb(r, g, b);
                    }
                } else if hex.len() == 8 {
                    if let (Ok(r), Ok(g), Ok(b), Ok(_a)) = (
                        u8::from_str_radix(&hex[0..2], 16),
                        u8::from_str_radix(&hex[2..4], 16),
                        u8::from_str_radix(&hex[4..6], 16),
                        u8::from_str_radix(&hex[6..8], 16),
                    ) {
                        return Color::Rgb(r, g, b);
                    }
                }
                Color::White
            }
            ColorValue::Named(name) => match name.to_lowercase().as_str() {
                "black" => Color::Black,
                "red" => Color::Red,
                "green" => Color::Green,
                "yellow" => Color::Yellow,
                "blue" => Color::Blue,
                "magenta" => Color::Magenta,
                "cyan" => Color::Cyan,
                "white" => Color::White,
                "gray" | "grey" => Color::Gray,
                "darkgray" | "darkgrey" => Color::DarkGray,
                "lightred" => Color::LightRed,
                "lightgreen" => Color::LightGreen,
                "lightyellow" => Color::LightYellow,
                "lightblue" => Color::LightBlue,
                "lightmagenta" => Color::LightMagenta,
                "lightcyan" => Color::LightCyan,
                _ => Color::White,
            },
        }
    }

    pub fn get_alpha(&self) -> f32 {
        match self {
            ColorValue::Rgba { a, .. } => *a,
            _ => 1.0,
        }
    }

    pub fn with_alpha(&self, alpha: f32) -> ColorValue {
        match self {
            ColorValue::Rgba { r, g, b, a: _ } => ColorValue::Rgba {
                r: *r,
                g: *g,
                b: *b,
                a: alpha,
            },
            ColorValue::Rgb { r, g, b } => ColorValue::Rgba {
                r: *r,
                g: *g,
                b: *b,
                a: alpha,
            },
            ColorValue::Hex(_hex) => {
                let color = self.to_ratatui_color();
                if let Color::Rgb(r, g, b) = color {
                    ColorValue::Rgba { r, g, b, a: alpha }
                } else {
                    ColorValue::Rgba {
                        r: 255,
                        g: 255,
                        b: 255,
                        a: alpha,
                    }
                }
            }
            ColorValue::Named(_) => {
                let color = self.to_ratatui_color();
                if let Color::Rgb(r, g, b) = color {
                    ColorValue::Rgba { r, g, b, a: alpha }
                } else {
                    ColorValue::Rgba {
                        r: 255,
                        g: 255,
                        b: 255,
                        a: alpha,
                    }
                }
            }
        }
    }

    fn blend_with_background(&self, r: u8, g: u8, b: u8, alpha: f32) -> Color {
        if alpha < 0.1 {
            return Color::Reset;
        }

        let bg_r = 30u8;
        let bg_g = 30u8;
        let bg_b = 30u8;

        let blended_r = ((r as f32 * alpha) + (bg_r as f32 * (1.0 - alpha))) as u8;
        let blended_g = ((g as f32 * alpha) + (bg_g as f32 * (1.0 - alpha))) as u8;
        let blended_b = ((b as f32 * alpha) + (bg_b as f32 * (1.0 - alpha))) as u8;

        Color::Rgb(blended_r, blended_g, blended_b)
    }

    pub fn to_transparent_color(&self) -> Color {
        match self {
            ColorValue::Rgba { r, g, b, a } => {
                if *a < 0.5 {
                    Color::Reset
                } else {
                    self.blend_with_background(*r, *g, *b, *a)
                }
            }
            _ => self.to_ratatui_color(),
        }
    }
}

impl NeoTheme {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read theme file: {}", path.display()))?;

        let theme: NeoTheme = toml::from_str(&content)
            .with_context(|| format!("Failed to parse theme file: {}", path.display()))?;

        Ok(theme)
    }

    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let path = path.as_ref();
        let content = toml::to_string_pretty(self).with_context(|| "Failed to serialize theme")?;

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).with_context(|| {
                format!("Failed to create theme directory: {}", parent.display())
            })?;
        }

        std::fs::write(path, content)
            .with_context(|| format!("Failed to write theme file: {}", path.display()))?;

        Ok(())
    }

    pub fn default_dark() -> Self {
        Self {
            name: "Default Dark".to_string(),
            author: "NeoCrim Team".to_string(),
            description: "Default dark theme for NeoCrim".to_string(),
            colors: ThemeColors {
                background: ColorValue::Rgb {
                    r: 30,
                    g: 30,
                    b: 30,
                },
                foreground: ColorValue::Rgb {
                    r: 212,
                    g: 212,
                    b: 212,
                },
                cursor: ColorValue::Rgb {
                    r: 255,
                    g: 255,
                    b: 0,
                },
                selection: ColorValue::Rgb {
                    r: 38,
                    g: 79,
                    b: 120,
                },
                line_number: ColorValue::Rgb {
                    r: 133,
                    g: 133,
                    b: 133,
                },
                current_line: ColorValue::Rgb {
                    r: 45,
                    g: 45,
                    b: 45,
                },

                background_opacity: 1.0,
                blur_radius: 0.0,
                enable_glass: false,

                status_bg: ColorValue::Rgb {
                    r: 0,
                    g: 120,
                    b: 215,
                },
                status_fg: ColorValue::Rgb {
                    r: 255,
                    g: 255,
                    b: 255,
                },
                command_bg: ColorValue::Rgb {
                    r: 30,
                    g: 30,
                    b: 30,
                },
                command_fg: ColorValue::Rgb {
                    r: 212,
                    g: 212,
                    b: 212,
                },

                explorer_bg: ColorValue::Rgb {
                    r: 37,
                    g: 37,
                    b: 38,
                },
                explorer_fg: ColorValue::Rgb {
                    r: 212,
                    g: 212,
                    b: 212,
                },
                explorer_selected: ColorValue::Rgb {
                    r: 0,
                    g: 120,
                    b: 215,
                },
                explorer_directory: ColorValue::Rgb {
                    r: 78,
                    g: 201,
                    b: 176,
                },
                explorer_file: ColorValue::Rgb {
                    r: 212,
                    g: 212,
                    b: 212,
                },

                accent_color: ColorValue::Rgb {
                    r: 0,
                    g: 120,
                    b: 215,
                },
                border_color: ColorValue::Rgb {
                    r: 100,
                    g: 100,
                    b: 100,
                },
                inactive_border: ColorValue::Rgb {
                    r: 60,
                    g: 60,
                    b: 60,
                },
                highlight: ColorValue::Rgb {
                    r: 255,
                    g: 255,
                    b: 255,
                },
                shadow: ColorValue::Rgb { r: 0, g: 0, b: 0 },

                syntax_keyword: ColorValue::Rgb {
                    r: 86,
                    g: 156,
                    b: 214,
                },
                syntax_string: ColorValue::Rgb {
                    r: 206,
                    g: 145,
                    b: 120,
                },
                syntax_comment: ColorValue::Rgb {
                    r: 106,
                    g: 153,
                    b: 85,
                },
                syntax_function: ColorValue::Rgb {
                    r: 220,
                    g: 220,
                    b: 170,
                },
                syntax_type: ColorValue::Rgb {
                    r: 78,
                    g: 201,
                    b: 176,
                },
                syntax_constant: ColorValue::Rgb {
                    r: 100,
                    g: 102,
                    b: 149,
                },
                syntax_variable: ColorValue::Rgb {
                    r: 156,
                    g: 220,
                    b: 254,
                },
                syntax_number: ColorValue::Rgb {
                    r: 181,
                    g: 206,
                    b: 168,
                },
                syntax_operator: ColorValue::Rgb {
                    r: 212,
                    g: 212,
                    b: 212,
                },
                syntax_punctuation: ColorValue::Rgb {
                    r: 212,
                    g: 212,
                    b: 212,
                },

                syntax_attribute: ColorValue::Rgb {
                    r: 156,
                    g: 220,
                    b: 254,
                },
                syntax_special: ColorValue::Rgb {
                    r: 255,
                    g: 215,
                    b: 0,
                },
                syntax_tag: ColorValue::Rgb {
                    r: 86,
                    g: 156,
                    b: 214,
                },
                syntax_link: ColorValue::Rgb {
                    r: 86,
                    g: 156,
                    b: 214,
                },
                error_color: ColorValue::Rgb {
                    r: 244,
                    g: 67,
                    b: 54,
                },
                warning_color: ColorValue::Rgb {
                    r: 255,
                    g: 152,
                    b: 0,
                },
                info_color: ColorValue::Rgb {
                    r: 33,
                    g: 150,
                    b: 243,
                },
                success_color: ColorValue::Rgb {
                    r: 76,
                    g: 175,
                    b: 80,
                },
            },
        }
    }

    pub fn default_light() -> Self {
        Self {
            name: "Default Light".to_string(),
            author: "NeoCrim Team".to_string(),
            description: "Default light theme for NeoCrim".to_string(),
            colors: ThemeColors {
                background: ColorValue::Rgb {
                    r: 255,
                    g: 255,
                    b: 255,
                },
                foreground: ColorValue::Rgb {
                    r: 51,
                    g: 51,
                    b: 51,
                },
                cursor: ColorValue::Rgb { r: 0, g: 0, b: 255 },
                selection: ColorValue::Rgb {
                    r: 173,
                    g: 214,
                    b: 255,
                },
                line_number: ColorValue::Rgb {
                    r: 133,
                    g: 133,
                    b: 133,
                },
                current_line: ColorValue::Rgb {
                    r: 245,
                    g: 245,
                    b: 245,
                },

                background_opacity: 1.0,
                blur_radius: 0.0,
                enable_glass: false,

                status_bg: ColorValue::Rgb {
                    r: 0,
                    g: 120,
                    b: 215,
                },
                status_fg: ColorValue::Rgb {
                    r: 255,
                    g: 255,
                    b: 255,
                },
                command_bg: ColorValue::Rgb {
                    r: 255,
                    g: 255,
                    b: 255,
                },
                command_fg: ColorValue::Rgb {
                    r: 51,
                    g: 51,
                    b: 51,
                },

                explorer_bg: ColorValue::Rgb {
                    r: 246,
                    g: 246,
                    b: 246,
                },
                explorer_fg: ColorValue::Rgb {
                    r: 51,
                    g: 51,
                    b: 51,
                },
                explorer_selected: ColorValue::Rgb {
                    r: 0,
                    g: 120,
                    b: 215,
                },
                explorer_directory: ColorValue::Rgb {
                    r: 0,
                    g: 120,
                    b: 215,
                },
                explorer_file: ColorValue::Rgb {
                    r: 51,
                    g: 51,
                    b: 51,
                },

                accent_color: ColorValue::Rgb {
                    r: 0,
                    g: 120,
                    b: 215,
                },
                border_color: ColorValue::Rgb {
                    r: 200,
                    g: 200,
                    b: 200,
                },
                inactive_border: ColorValue::Rgb {
                    r: 220,
                    g: 220,
                    b: 220,
                },
                highlight: ColorValue::Rgb { r: 0, g: 0, b: 0 },
                shadow: ColorValue::Rgb {
                    r: 128,
                    g: 128,
                    b: 128,
                },

                syntax_keyword: ColorValue::Rgb { r: 0, g: 0, b: 255 },
                syntax_string: ColorValue::Rgb {
                    r: 163,
                    g: 21,
                    b: 21,
                },
                syntax_comment: ColorValue::Rgb { r: 0, g: 128, b: 0 },
                syntax_function: ColorValue::Rgb {
                    r: 121,
                    g: 94,
                    b: 38,
                },
                syntax_type: ColorValue::Rgb {
                    r: 43,
                    g: 145,
                    b: 175,
                },
                syntax_constant: ColorValue::Rgb {
                    r: 111,
                    g: 66,
                    b: 193,
                },
                syntax_variable: ColorValue::Rgb {
                    r: 0,
                    g: 112,
                    b: 193,
                },
                syntax_number: ColorValue::Rgb {
                    r: 9,
                    g: 134,
                    b: 88,
                },
                syntax_operator: ColorValue::Rgb {
                    r: 51,
                    g: 51,
                    b: 51,
                },
                syntax_punctuation: ColorValue::Rgb {
                    r: 51,
                    g: 51,
                    b: 51,
                },

                syntax_attribute: ColorValue::Rgb {
                    r: 0,
                    g: 112,
                    b: 193,
                },
                syntax_special: ColorValue::Rgb {
                    r: 255,
                    g: 140,
                    b: 0,
                },
                syntax_tag: ColorValue::Rgb { r: 0, g: 0, b: 255 },
                syntax_link: ColorValue::Rgb { r: 0, g: 0, b: 255 },
                error_color: ColorValue::Rgb {
                    r: 244,
                    g: 67,
                    b: 54,
                },
                warning_color: ColorValue::Rgb {
                    r: 255,
                    g: 152,
                    b: 0,
                },
                info_color: ColorValue::Rgb {
                    r: 33,
                    g: 150,
                    b: 243,
                },
                success_color: ColorValue::Rgb {
                    r: 76,
                    g: 175,
                    b: 80,
                },
            },
        }
    }

    pub fn to_legacy_theme(&self) -> crate::ui::theme::Theme {
        use ratatui::style::Style;

        crate::ui::theme::Theme {
            background: self.colors.background.to_ratatui_color(),
            foreground: self.colors.foreground.to_ratatui_color(),
            cursor: self.colors.cursor.to_ratatui_color(),
            selection: self.colors.selection.to_ratatui_color(),
            line_number: self.colors.line_number.to_ratatui_color(),
            current_line: self.colors.current_line.to_ratatui_color(),
            status_bg: self.colors.status_bg.to_ratatui_color(),
            status_fg: self.colors.status_fg.to_ratatui_color(),
            command_bg: self.colors.command_bg.to_ratatui_color(),
            command_fg: self.colors.command_fg.to_ratatui_color(),
            terminal_border: Style::default().fg(self.colors.border_color.to_ratatui_color()),
            terminal_title: Style::default().fg(self.colors.status_fg.to_ratatui_color()),
            terminal_background: Style::default()
                .bg(self.colors.background.to_ratatui_color())
                .fg(self.colors.foreground.to_ratatui_color()),
            terminal_command: Style::default().fg(self.colors.syntax_keyword.to_ratatui_color()),
            terminal_output: Style::default().fg(self.colors.foreground.to_ratatui_color()),
            terminal_error: Style::default().fg(self.colors.error_color.to_ratatui_color()),
            terminal_running: Style::default().fg(self.colors.warning_color.to_ratatui_color()),
            scrollbar: Style::default().fg(self.colors.line_number.to_ratatui_color()),
        }
    }
}

impl Default for NeoTheme {
    fn default() -> Self {
        Self::default_dark()
    }
}
