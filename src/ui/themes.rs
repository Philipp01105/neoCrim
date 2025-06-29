use serde::{Deserialize, Serialize};
use ratatui::style::Color;
use std::path::Path;
use crate::Result;
use anyhow::Context;

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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ColorValue {
    Rgb { r: u8, g: u8, b: u8 },
    Hex(String),
    Named(String),
}

impl ColorValue {
    pub fn to_ratatui_color(&self) -> Color {
        match self {
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
            }
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
        let content = toml::to_string_pretty(self)
            .with_context(|| "Failed to serialize theme")?;
        
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create theme directory: {}", parent.display()))?;
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
                background: ColorValue::Rgb { r: 30, g: 30, b: 30 },
                foreground: ColorValue::Rgb { r: 212, g: 212, b: 212 },
                cursor: ColorValue::Rgb { r: 255, g: 255, b: 0 },
                selection: ColorValue::Rgb { r: 38, g: 79, b: 120 },
                line_number: ColorValue::Rgb { r: 133, g: 133, b: 133 },
                current_line: ColorValue::Rgb { r: 45, g: 45, b: 45 },
                
                status_bg: ColorValue::Rgb { r: 0, g: 120, b: 215 },
                status_fg: ColorValue::Rgb { r: 255, g: 255, b: 255 },
                command_bg: ColorValue::Rgb { r: 30, g: 30, b: 30 },
                command_fg: ColorValue::Rgb { r: 212, g: 212, b: 212 },
                
                explorer_bg: ColorValue::Rgb { r: 37, g: 37, b: 38 },
                explorer_fg: ColorValue::Rgb { r: 212, g: 212, b: 212 },
                explorer_selected: ColorValue::Rgb { r: 0, g: 120, b: 215 },
                explorer_directory: ColorValue::Rgb { r: 78, g: 201, b: 176 },
                explorer_file: ColorValue::Rgb { r: 212, g: 212, b: 212 },
                
                syntax_keyword: ColorValue::Rgb { r: 86, g: 156, b: 214 },
                syntax_string: ColorValue::Rgb { r: 206, g: 145, b: 120 },
                syntax_comment: ColorValue::Rgb { r: 106, g: 153, b: 85 },
                syntax_function: ColorValue::Rgb { r: 220, g: 220, b: 170 },
                syntax_type: ColorValue::Rgb { r: 78, g: 201, b: 176 },
                syntax_constant: ColorValue::Rgb { r: 100, g: 102, b: 149 },
                syntax_variable: ColorValue::Rgb { r: 156, g: 220, b: 254 },
                syntax_number: ColorValue::Rgb { r: 181, g: 206, b: 168 },
                syntax_operator: ColorValue::Rgb { r: 212, g: 212, b: 212 },
                syntax_punctuation: ColorValue::Rgb { r: 212, g: 212, b: 212 },
            },
        }
    }

    pub fn default_light() -> Self {
        Self {
            name: "Default Light".to_string(),
            author: "NeoCrim Team".to_string(),
            description: "Default light theme for NeoCrim".to_string(),
            colors: ThemeColors {
                background: ColorValue::Rgb { r: 255, g: 255, b: 255 },
                foreground: ColorValue::Rgb { r: 51, g: 51, b: 51 },
                cursor: ColorValue::Rgb { r: 0, g: 0, b: 255 },
                selection: ColorValue::Rgb { r: 173, g: 214, b: 255 },
                line_number: ColorValue::Rgb { r: 133, g: 133, b: 133 },
                current_line: ColorValue::Rgb { r: 245, g: 245, b: 245 },
                
                status_bg: ColorValue::Rgb { r: 0, g: 120, b: 215 },
                status_fg: ColorValue::Rgb { r: 255, g: 255, b: 255 },
                command_bg: ColorValue::Rgb { r: 255, g: 255, b: 255 },
                command_fg: ColorValue::Rgb { r: 51, g: 51, b: 51 },
                
                explorer_bg: ColorValue::Rgb { r: 246, g: 246, b: 246 },
                explorer_fg: ColorValue::Rgb { r: 51, g: 51, b: 51 },
                explorer_selected: ColorValue::Rgb { r: 0, g: 120, b: 215 },
                explorer_directory: ColorValue::Rgb { r: 0, g: 120, b: 215 },
                explorer_file: ColorValue::Rgb { r: 51, g: 51, b: 51 },
                
                syntax_keyword: ColorValue::Rgb { r: 0, g: 0, b: 255 },
                syntax_string: ColorValue::Rgb { r: 163, g: 21, b: 21 },
                syntax_comment: ColorValue::Rgb { r: 0, g: 128, b: 0 },
                syntax_function: ColorValue::Rgb { r: 121, g: 94, b: 38 },
                syntax_type: ColorValue::Rgb { r: 43, g: 145, b: 175 },
                syntax_constant: ColorValue::Rgb { r: 111, g: 66, b: 193 },
                syntax_variable: ColorValue::Rgb { r: 0, g: 112, b: 193 },
                syntax_number: ColorValue::Rgb { r: 9, g: 134, b: 88 },
                syntax_operator: ColorValue::Rgb { r: 51, g: 51, b: 51 },
                syntax_punctuation: ColorValue::Rgb { r: 51, g: 51, b: 51 },
            },
        }
    }

    pub fn to_legacy_theme(&self) -> crate::ui::Theme {
        crate::ui::Theme {
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
        }
    }
}

impl Default for NeoTheme {
    fn default() -> Self {
        Self::default_dark()
    }
}