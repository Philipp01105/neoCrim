use serde::{Deserialize, Serialize};
use std::path::{PathBuf};
use crate::ui::{Theme, NeoTheme};
use crate::Result;
use anyhow::Context;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub editor: EditorConfig,
    pub ui: UiConfig,
    pub keybindings: KeybindingsConfig,
    #[serde(skip)]
    pub theme: Theme,
    #[serde(skip)]
    pub current_theme: NeoTheme,
    pub theme_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorConfig {
    pub line_numbers: bool,
    pub relative_line_numbers: bool,
    pub tab_size: usize,
    pub insert_tabs: bool,
    pub auto_save: bool,
    pub wrap_lines: bool,
    pub scroll_offset: usize,
    pub syntax_highlighting: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    pub theme: String,
    pub show_status_line: bool,
    pub show_command_line: bool,
    pub cursor_blink: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeybindingsConfig {
    pub leader: String,
}

impl Config {
    pub fn load() -> Result<Self> {
        if let Some(config_path) = Self::config_file_path() {
            if config_path.exists() {
                let content = std::fs::read_to_string(&config_path)
                    .with_context(|| format!("Failed to read config file: {}", config_path.display()))?;
                
                let mut config: Config = toml::from_str(&content)
                    .with_context(|| "Failed to parse config file")?;
               
                config.current_theme = if let Some(ref theme_path) = config.theme_path {
                    NeoTheme::load_from_file(theme_path).unwrap_or_else(|_| NeoTheme::default())
                } else {
                    NeoTheme::default()
                };
                
                config.theme = config.current_theme.to_legacy_theme();
                
                return Ok(config);
            }
        }
        
        Ok(Self::default())
    }

    pub fn set_theme<P: AsRef<std::path::Path>>(&mut self, theme_path: P) -> Result<()> {
        let theme_path = theme_path.as_ref();
        
        let new_theme = NeoTheme::load_from_file(theme_path)
            .with_context(|| format!("Failed to load theme from: {}", theme_path.display()))?;
        
        self.current_theme = new_theme;
        self.theme = self.current_theme.to_legacy_theme();
        self.theme_path = Some(theme_path.to_path_buf());
        
        self.save()
            .with_context(|| "Failed to save config after setting theme")?;
        
        Ok(())
    }

    pub fn save(&self) -> Result<()> {
        if let Some(config_path) = Self::config_file_path() {
            if let Some(parent) = config_path.parent() {
                std::fs::create_dir_all(parent)
                    .with_context(|| format!("Failed to create config directory: {}", parent.display()))?;
            }

            let content = toml::to_string_pretty(self)
                .with_context(|| "Failed to serialize config")?;

            std::fs::write(&config_path, content)
                .with_context(|| format!("Failed to write config file: {}", config_path.display()))?;
        }
        Ok(())
    }

    pub fn config_file_path() -> Option<PathBuf> {
        dirs::config_dir().map(|dir| dir.join("neocrim").join("config.toml"))
    }

    pub fn reload(&mut self) -> Result<()> {
        *self = Self::load()?;
        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        let current_theme = NeoTheme::default();
        let legacy_theme = current_theme.to_legacy_theme();
        
        Self {
            editor: EditorConfig {
                line_numbers: true,
                relative_line_numbers: false,
                tab_size: 4,
                insert_tabs: false,
                auto_save: false,
                wrap_lines: false,
                scroll_offset: 5,
                syntax_highlighting: true,
            },
            ui: UiConfig {
                theme: "default".to_string(),
                show_status_line: true,
                show_command_line: true,
                cursor_blink: true,
            },
            keybindings: KeybindingsConfig {
                leader: " ".to_string(),
            },
            theme: legacy_theme,
            current_theme,
            theme_path: None,
        }
    }
}
