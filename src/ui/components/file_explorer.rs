use crate::ui::Theme;
use crate::Result;
use anyhow::Context;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};
use std::fs;
use std::path::{Path, PathBuf};

pub struct FileExplorer {
    pub current_dir: PathBuf,
    pub entries: Vec<FileEntry>,
    pub selected_index: usize,
    pub list_state: ListState,
    pub visible: bool,
}

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub name: String,
    pub path: PathBuf,
    pub is_directory: bool,
    pub is_hidden: bool,
}

impl FileExplorer {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let current_dir = std::fs::canonicalize(path.as_ref()).with_context(|| {
            format!(
                "Failed to get absolute path for: {}",
                path.as_ref().display()
            )
        })?;

        let mut explorer = Self {
            current_dir,
            entries: Vec::new(),
            selected_index: 0,
            list_state: ListState::default(),
            visible: false,
        };
        explorer.refresh()?;
        Ok(explorer)
    }

    pub fn go_to_parent(&mut self) -> Result<()> {
        let current_dir = self.current_dir.clone();

        if let Some(parent) = current_dir.parent() {
            let parent_path = parent.to_path_buf();

            if !parent_path.exists() {
                return Err(anyhow::anyhow!(
                    "Parent directory does not exist: {}",
                    parent_path.display()
                ));
            }

            if !parent_path.is_dir() {
                return Err(anyhow::anyhow!(
                    "Parent is not a directory: {}",
                    parent_path.display()
                ));
            }

            match std::fs::read_dir(&parent_path) {
                Ok(_) => {}
                Err(e) => {
                    return Err(anyhow::anyhow!("Cannot read parent directory: {}", e));
                }
            }

            let backup_dir = self.current_dir.clone();
            let backup_entries = self.entries.clone();
            let backup_selected = self.selected_index;

            self.current_dir = parent_path;

            match self.refresh() {
                Ok(()) => {
                    if let Some(dir_name) = current_dir.file_name() {
                        let dir_name_str = dir_name.to_string_lossy();
                        for (index, entry) in self.entries.iter().enumerate() {
                            if entry.name == dir_name_str && entry.is_directory {
                                self.selected_index = index;
                                self.list_state.select(Some(index));
                                break;
                            }
                        }
                    }
                    Ok(())
                }
                Err(e) => {
                    self.current_dir = backup_dir;
                    self.entries = backup_entries;
                    self.selected_index = backup_selected;
                    if backup_selected < self.entries.len() {
                        self.list_state.select(Some(backup_selected));
                    }
                    Err(anyhow::anyhow!("Cannot access parent directory: {}", e))
                }
            }
        } else {
            Err(anyhow::anyhow!("Already at root directory"))
        }
    }

    pub fn refresh(&mut self) -> Result<()> {
        self.entries.clear();

        if !self.current_dir.exists() {
            return Err(anyhow::anyhow!(
                "Directory does not exist: {}",
                self.current_dir.display()
            ));
        }

        if !self.current_dir.is_dir() {
            return Err(anyhow::anyhow!(
                "Path is not a directory: {}",
                self.current_dir.display()
            ));
        }

        let entries = match fs::read_dir(&self.current_dir) {
            Ok(entries) => entries,
            Err(e) => {
                return Err(anyhow::anyhow!(
                    "Cannot read directory '{}': {}",
                    self.current_dir.display(),
                    e
                ));
            }
        };

        let mut file_entries: Vec<FileEntry> = Vec::new();

        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                let name = entry.file_name().to_string_lossy().to_string();
                let is_directory = path.is_dir();
                let is_hidden = name.starts_with('.');

                file_entries.push(FileEntry {
                    name,
                    path,
                    is_directory,
                    is_hidden,
                });
            }
        }

        file_entries.sort_by(|a, b| match (a.is_directory, b.is_directory) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        });

        self.entries = file_entries;

        self.safe_reset_selection();
        Ok(())
    }

    fn safe_reset_selection(&mut self) {
        if !self.entries.is_empty() {
            self.selected_index = self.selected_index.min(self.entries.len() - 1);
            self.list_state.select(Some(self.selected_index));
        } else {
            self.selected_index = 0;
            self.list_state.select(None);
        }
    }

    pub fn toggle_visibility(&mut self) {
        self.visible = !self.visible;
    }

    pub fn move_up(&mut self) {
        if !self.entries.is_empty() && self.selected_index > 0 {
            self.selected_index -= 1;
            self.list_state.select(Some(self.selected_index));
        }
    }

    pub fn move_down(&mut self) {
        if !self.entries.is_empty() && self.selected_index < self.entries.len() - 1 {
            self.selected_index += 1;
            self.list_state.select(Some(self.selected_index));
        }
    }

    pub fn select_current(&mut self) -> Result<Option<PathBuf>> {
        if self.entries.is_empty() {
            return Ok(None);
        }

        if let Some(entry) = self.entries.get(self.selected_index) {
            if entry.is_directory {
                if !entry.path.exists() || !entry.path.is_dir() {
                    return Err(anyhow::anyhow!("Directory no longer exists"));
                }

                if let Err(e) = std::fs::read_dir(&entry.path) {
                    return Err(anyhow::anyhow!("Cannot access directory: {}", e));
                }

                self.current_dir = entry.path.clone();
                self.refresh()?;
                Ok(None)
            } else {
                if !entry.path.exists() {
                    return Err(anyhow::anyhow!("File no longer exists"));
                }
                Ok(Some(entry.path.clone()))
            }
        } else {
            Err(anyhow::anyhow!("Invalid selection index"))
        }
    }

    pub fn get_current_path(&self) -> &Path {
        &self.current_dir
    }

    pub fn navigate_to<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let target_path = path.as_ref();

        if !target_path.exists() {
            return Err(anyhow::anyhow!(
                "Path does not exist: {}",
                target_path.display()
            ));
        }

        let target_dir = if target_path.is_dir() {
            target_path.to_path_buf()
        } else if let Some(parent) = target_path.parent() {
            parent.to_path_buf()
        } else {
            return Err(anyhow::anyhow!("Invalid path: {}", target_path.display()));
        };

        if let Err(e) = std::fs::read_dir(&target_dir) {
            return Err(anyhow::anyhow!("Cannot access directory: {}", e));
        }

        self.current_dir = target_dir;
        self.refresh()?;

        if target_path.is_file() {
            if let Some(file_name) = target_path.file_name() {
                let file_name_str = file_name.to_string_lossy();
                for (index, entry) in self.entries.iter().enumerate() {
                    if entry.name == file_name_str {
                        self.selected_index = index;
                        self.list_state.select(Some(index));
                        break;
                    }
                }
            }
        }

        Ok(())
    }

    pub fn can_go_to_parent(&self) -> bool {
        if let Some(parent) = self.current_dir.parent() {
            parent.exists() && parent.is_dir()
        } else {
            false
        }
    }

    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }

    pub fn get_selected_entry(&self) -> Option<&FileEntry> {
        self.entries.get(self.selected_index)
    }

    pub fn set_selection(&mut self, index: usize) {
        if index < self.entries.len() {
            self.selected_index = index;
            self.list_state.select(Some(index));
        }
    }

    pub fn filter_entries(&self, pattern: &str) -> Vec<&FileEntry> {
        let pattern_lower = pattern.to_lowercase();
        self.entries
            .iter()
            .filter(|entry| entry.name.to_lowercase().contains(&pattern_lower))
            .collect()
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect, theme: &Theme) {
        if !self.visible {
            return;
        }

        let items: Vec<ListItem> = self
            .entries
            .iter()
            .map(|entry| {
                let icon = if entry.is_directory {
                    "📁 "
                } else {
                    match entry.name.split('.').last().unwrap_or("") {
                        "rs" => "🦀 ",
                        "md" => "📝 ",
                        "txt" => "📄 ",
                        "json" | "toml" | "yaml" | "yml" => "⚙️ ",
                        "js" | "ts" => "📜 ",
                        "py" => "🐍 ",
                        "cpp" | "c" | "h" => "⚡ ",
                        "html" | "css" => "🌐 ",
                        "png" | "jpg" | "jpeg" | "gif" => "🖼️ ",
                        _ => "📄 ",
                    }
                };

                let style = if entry.is_directory {
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD)
                } else if entry.is_hidden {
                    Style::default().fg(Color::Gray).add_modifier(Modifier::DIM)
                } else {
                    Style::default().fg(theme.foreground)
                };

                ListItem::new(Line::from(vec![
                    Span::raw(icon),
                    Span::styled(&entry.name, style),
                ]))
            })
            .collect();

        let parent_indicator = if self.can_go_to_parent() {
            " (h: ���️ parent)"
        } else {
            ""
        };
        let title = format!(
            " File Explorer - {}{} ({} items) ",
            self.current_dir.display(),
            parent_indicator,
            self.entries.len()
        );

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(title)
                    .style(Style::default().bg(theme.background)),
            )
            .highlight_style(
                Style::default()
                    .bg(theme.selection)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("▶ ");

        frame.render_stateful_widget(list, area, &mut self.list_state);
    }
}

impl Default for FileExplorer {
    fn default() -> Self {
        Self::new(".").unwrap_or_else(|_| {
            if let Some(home_dir) = dirs::home_dir() {
                Self::new(home_dir).unwrap_or_else(|_| Self {
                    current_dir: PathBuf::from("/"),
                    entries: Vec::new(),
                    selected_index: 0,
                    list_state: ListState::default(),
                    visible: false,
                })
            } else {
                Self {
                    current_dir: PathBuf::from("/"),
                    entries: Vec::new(),
                    selected_index: 0,
                    list_state: ListState::default(),
                    visible: false,
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_file_explorer_creation() {
        let temp_dir = TempDir::new().unwrap();
        let explorer = FileExplorer::new(temp_dir.path()).unwrap();
        assert_eq!(explorer.current_dir, temp_dir.path());
        assert!(!explorer.visible);
    }

    #[test]
    fn test_navigation() {
        let temp_dir = TempDir::new().unwrap();
        let mut explorer = FileExplorer::new(temp_dir.path()).unwrap();

        let sub_dir = temp_dir.path().join("subdir");
        fs::create_dir(&sub_dir).unwrap();
        explorer.refresh().unwrap();

        assert!(!explorer.entries.is_empty());
    }

    #[test]
    fn test_bounds_checking() {
        let temp_dir = TempDir::new().unwrap();
        let mut explorer = FileExplorer::new(temp_dir.path()).unwrap();

        explorer.move_down();
        assert_eq!(explorer.selected_index, 0);

        explorer.move_up();
        assert_eq!(explorer.selected_index, 0);
    }
}
