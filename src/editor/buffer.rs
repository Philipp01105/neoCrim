use ropey::Rope;
use std::path::{Path, PathBuf};
use crate::Result;
use anyhow::Context;

#[derive(Debug, Clone)]
pub struct Buffer {
    pub content: Rope,
    pub file_path: Option<PathBuf>,
    pub is_modified: bool,
    pub is_readonly: bool,
}

impl Buffer {
    pub fn empty() -> Self {
        Self {
            content: Rope::new(),
            file_path: None,
            is_modified: false,
            is_readonly: false,
        }
    }

    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read file: {}", path.display()))?;
        
        Ok(Self {
            content: Rope::from_str(&content),
            file_path: Some(path.to_path_buf()),
            is_modified: false,
            is_readonly: false,
        })
    }

    pub fn save(&mut self) -> Result<()> {
        if let Some(path) = &self.file_path {
            let content = self.content.to_string();
            std::fs::write(path, content)
                .with_context(|| format!("Failed to save file: {}", path.display()))?;
            self.is_modified = false;
        }
        Ok(())
    }

    pub fn save_as<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let path = path.as_ref();
        let content = self.content.to_string();
        std::fs::write(path, content)
            .with_context(|| format!("Failed to save file: {}", path.display()))?;
        self.file_path = Some(path.to_path_buf());
        self.is_modified = false;
        Ok(())
    }

    pub fn insert_char(&mut self, line: usize, col: usize, ch: char) {
        if let Some(char_idx) = self.line_col_to_char_idx(line, col) {
            self.content.insert_char(char_idx, ch);
            self.is_modified = true;
        }
    }

    pub fn insert_str(&mut self, line: usize, col: usize, s: &str) {
        if let Some(char_idx) = self.line_col_to_char_idx(line, col) {
            self.content.insert(char_idx, s);
            self.is_modified = true;
        }
    }

    pub fn delete_char(&mut self, line: usize, col: usize) {
        if let Some(char_idx) = self.line_col_to_char_idx(line, col) {
            if char_idx < self.content.len_chars() {
                self.content.remove(char_idx..char_idx + 1);
                self.is_modified = true;
            }
        }
    }

    pub fn delete_range(&mut self, start_line: usize, start_col: usize, end_line: usize, end_col: usize) {
        if let (Some(start_idx), Some(end_idx)) = (
            self.line_col_to_char_idx(start_line, start_col),
            self.line_col_to_char_idx(end_line, end_col)
        ) {
            if start_idx < end_idx && end_idx <= self.content.len_chars() {
                self.content.remove(start_idx..end_idx);
                self.is_modified = true;
            }
        }
    }

    pub fn line_count(&self) -> usize {
        self.content.len_lines()
    }

    pub fn line(&self, line_idx: usize) -> Option<String> {
        if line_idx < self.content.len_lines() {
            Some(self.content.line(line_idx).to_string())
        } else {
            None
        }
    }

    pub fn line_len(&self, line_idx: usize) -> usize {
        if line_idx < self.content.len_lines() {
            self.content.line(line_idx).len_chars().saturating_sub(1) 
        } else {
            0
        }
    }

    pub fn is_empty(&self) -> bool {
        self.content.len_chars() == 0
    }

    pub fn file_path(&self) -> Option<&Path> {
        self.file_path.as_deref()
    }

    pub fn file_name(&self) -> Option<String> {
        self.file_path
            .as_ref()
            .and_then(|path| path.file_name())
            .and_then(|name| name.to_str())
            .map(|s| s.to_string())
    }

    fn line_col_to_char_idx(&self, line: usize, col: usize) -> Option<usize> {
        if line >= self.content.len_lines() {
            return None;
        }
        
        let line_start = self.content.line_to_char(line);
        let line_len = self.content.line(line).len_chars();
        
        if col > line_len {
            None
        } else {
            Some(line_start + col)
        }
    }
}
