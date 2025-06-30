use serde::{Deserialize, Serialize};
use std::path::{PathBuf};
use crate::ui::{Theme, NeoTheme, ThemeManager};
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
    #[serde(skip)]
    pub theme_manager: ThemeManager,
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
               
                config.theme_manager = ThemeManager::new();
                
                config.current_theme = if let Some(ref theme_path) = config.theme_path {
                    NeoTheme::load_from_file(theme_path).unwrap_or_else(|_| NeoTheme::default())
                } else {
                    config.theme_manager.get_theme_by_name(&config.ui.theme).unwrap_or_else(|_| NeoTheme::default())
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
        self.ui.theme = "custom".to_string();
        
        self.save()
            .with_context(|| "Failed to save config after setting theme")?;
        
        Ok(())
    }

    pub fn set_theme_by_name(&mut self, theme_name: &str) -> Result<()> {
        let new_theme = self.theme_manager.get_theme_by_name(theme_name)
            .with_context(|| format!("Failed to load theme: {}", theme_name))?;
        
        self.current_theme = new_theme;
        self.theme = self.current_theme.to_legacy_theme();
        self.ui.theme = theme_name.to_string();
        self.theme_path = None; 
        
        self.save()
            .with_context(|| "Failed to save config after setting theme")?;
        
        Ok(())
    }

    pub fn set_theme_by_index(&mut self, index: usize) -> Result<()> {
        let theme_names = self.theme_manager.list_themes().clone();
        if index >= theme_names.len() {
            return Err(anyhow::anyhow!("Theme index {} out of range (0-{})", index, theme_names.len() - 1));
        }
        
        let theme_name = theme_names[index].clone();
        self.set_theme_by_name(&theme_name)
    }

    pub fn set_theme_to_default(&mut self) -> Result<()> {
        self.set_theme_by_name("dark")
    }

    pub fn get_default_themes(&self) -> Vec<String> {
        self.theme_manager.list_themes().clone()
    }

    pub fn set_default_theme_by_index(&mut self, index: usize) -> Result<()> {
        self.set_theme_by_index(index)
    }

    pub fn list_available_themes(&self) -> Vec<(usize, String, String, String)> {
        let mut themes = Vec::new();
        for (index, theme_name) in self.theme_manager.list_themes().iter().enumerate() {
            if let Ok((name, author, description)) = self.theme_manager.get_theme_info(theme_name) {
                themes.push((index, name, author, description));
            }
        }
        themes
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
        let theme_manager = ThemeManager::new();
        let current_theme = theme_manager.get_theme_by_name("dark").unwrap_or_else(|_| NeoTheme::default());
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
                theme: "dark".to_string(),
                show_status_line: true,
                show_command_line: true,
                cursor_blink: true,
            },
            keybindings: KeybindingsConfig {
                leader: " ".to_string(),
            },
            theme: legacy_theme,
            current_theme,
            theme_manager,
            theme_path: None,
        }
    }
}
