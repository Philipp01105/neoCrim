use crate::ui::themes::NeoTheme;
use crate::Result;
use anyhow::Context;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ThemeManager {
    embedded_themes: HashMap<String, &'static str>,
    theme_names: Vec<String>,
}

impl ThemeManager {
    pub fn new() -> Self {
        let mut embedded_themes = HashMap::new();
        let mut theme_names = Vec::new();

        embedded_themes.insert(
            "dark".to_string(),
            include_str!("../../themes/dark.nctheme"),
        );
        embedded_themes.insert(
            "light".to_string(),
            include_str!("../../themes/light.nctheme"),
        );
        embedded_themes.insert(
            "monokai".to_string(),
            include_str!("../../themes/monokai.nctheme"),
        );
        embedded_themes.insert(
            "glass_dark".to_string(),
            include_str!("../../themes/glass_dark.nctheme"),
        );
        embedded_themes.insert(
            "cyberpunk".to_string(),
            include_str!("../../themes/cyberpunk.nctheme"),
        );
        embedded_themes.insert(
            "dracula".to_string(),
            include_str!("../../themes/dracula.nctheme"),
        );
        embedded_themes.insert(
            "nord".to_string(),
            include_str!("../../themes/nord.nctheme"),
        );
        embedded_themes.insert(
            "solarized_dark".to_string(),
            include_str!("../../themes/solarized_dark.nctheme"),
        );

        theme_names.extend(embedded_themes.keys().cloned());
        theme_names.sort();

        Self {
            embedded_themes,
            theme_names,
        }
    }

    pub fn get_theme_by_name(&self, name: &str) -> Result<NeoTheme> {
        let theme_content = self
            .embedded_themes
            .get(name)
            .ok_or_else(|| anyhow::anyhow!("Theme '{}' not found", name))?;

        let theme: NeoTheme = toml::from_str(theme_content)
            .with_context(|| format!("Failed to parse embedded theme: {}", name))?;

        Ok(theme)
    }

    pub fn get_theme_by_index(&self, index: usize) -> Result<NeoTheme> {
        if index >= self.theme_names.len() {
            return Err(anyhow::anyhow!(
                "Theme index {} out of range (0-{})",
                index,
                self.theme_names.len() - 1
            ));
        }

        let theme_name = &self.theme_names[index];
        self.get_theme_by_name(theme_name)
    }

    pub fn list_themes(&self) -> &Vec<String> {
        &self.theme_names
    }

    pub fn theme_count(&self) -> usize {
        self.theme_names.len()
    }

    pub fn get_theme_info(&self, name: &str) -> Result<(String, String, String)> {
        let theme = self.get_theme_by_name(name)?;
        Ok((theme.name, theme.author, theme.description))
    }
}

impl Default for ThemeManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_manager_initialization() {
        let manager = ThemeManager::new();
        assert!(manager.theme_count() > 0);
        assert!(manager.list_themes().contains(&"dark".to_string()));
        assert!(manager.list_themes().contains(&"glass_dark".to_string()));
    }

    #[test]
    fn test_get_theme_by_name() {
        let manager = ThemeManager::new();
        let theme = manager.get_theme_by_name("dark").unwrap();
        assert_eq!(theme.name, "Dark Theme");
    }

    #[test]
    fn test_get_theme_by_index() {
        let manager = ThemeManager::new();
        let theme = manager.get_theme_by_index(0).unwrap();
        assert!(!theme.name.is_empty());
    }

    #[test]
    fn test_invalid_theme_name() {
        let manager = ThemeManager::new();
        assert!(manager.get_theme_by_name("nonexistent").is_err());
    }

    #[test]
    fn test_invalid_theme_index() {
        let manager = ThemeManager::new();
        assert!(manager.get_theme_by_index(999).is_err());
    }
}
