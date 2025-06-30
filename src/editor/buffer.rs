use ropey::Rope;
use std::path::{Path, PathBuf};
use crate::Result;
use crate::ui::components::terminal::TerminalOutput;
use crate::editor::{Cursor, Selection};
use anyhow::Context;
use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub enum BufferType {
    File,
    Terminal,
}

#[derive(Debug, Clone)]
pub struct UndoState {
    pub content: Rope,
    pub cursor: Cursor,
}

#[derive(Debug, Clone)]
pub struct Buffer {
    pub content: Rope,
    pub file_path: Option<PathBuf>,
    pub is_modified: bool,
    pub is_readonly: bool,
    pub buffer_type: BufferType,
    pub terminal_output: Option<TerminalOutput>,
    pub undo_stack: VecDeque<UndoState>,
    pub redo_stack: VecDeque<UndoState>,
}

impl Buffer {
    pub fn empty() -> Self {
        Self {
            content: Rope::new(),
            file_path: None,
            is_modified: false,
            is_readonly: false,
            buffer_type: BufferType::File,
            terminal_output: None,
            undo_stack: VecDeque::new(),
            redo_stack: VecDeque::new(),
        }
    }

    pub fn terminal() -> Self {
        Self {
            content: Rope::new(),
            file_path: None,
            is_modified: false,
            is_readonly: true,
            buffer_type: BufferType::Terminal,
            terminal_output: Some(TerminalOutput::new()),
            undo_stack: VecDeque::new(),
            redo_stack: VecDeque::new(),
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
            buffer_type: BufferType::File,
            terminal_output: None,
            undo_stack: VecDeque::new(),
            redo_stack: VecDeque::new(),
        })
    }

    pub fn new_file<P: AsRef<Path>>(path: P) -> Self {
        Self {
            content: Rope::new(),
            file_path: Some(path.as_ref().to_path_buf()),
            is_modified: true, 
            is_readonly: false,
            buffer_type: BufferType::File,
            terminal_output: None,
            undo_stack: VecDeque::new(),
            redo_stack: VecDeque::new(),
        }
    }

    pub fn save(&mut self) -> Result<()> {
        if let Some(path) = &self.file_path {
            if let Some(parent) = path.parent() {
                if !parent.exists() {
                    std::fs::create_dir_all(parent)
                        .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
                }
            }
            
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

    pub fn get_selected_text(&self, selection: &Selection) -> String {
        if let Some((start, end)) = selection.get_range() {
            let start_char = self.cursor_to_char_idx(&start);
            let end_char = self.cursor_to_char_idx(&end);
            
            if start_char <= end_char && end_char <= self.content.len_chars() {
                self.content.slice(start_char..end_char).to_string()
            } else {
                String::new()
            }
        } else {
            String::new()
        }
    }

    pub fn delete_selected_text(&mut self, selection: &Selection) -> String {
        if let Some((start, end)) = selection.get_range() {
            let start_char = self.cursor_to_char_idx(&start);
            let end_char = self.cursor_to_char_idx(&end);
            
            if start_char <= end_char && end_char <= self.content.len_chars() {
                let deleted_text = self.content.slice(start_char..end_char).to_string();
                self.content.remove(start_char..end_char);
                self.is_modified = true;
                return deleted_text;
            }
        }
        String::new()
    }

    pub fn insert_text_at_cursor(&mut self, cursor: &Cursor, text: &str) {
        let char_idx = self.cursor_to_char_idx(cursor);
        if char_idx <= self.content.len_chars() {
            self.content.insert(char_idx, text);
            self.is_modified = true;
        }
    }

    fn cursor_to_char_idx(&self, cursor: &Cursor) -> usize {
        if self.content.len_lines() == 0 {
            return 0;
        }
        
        if cursor.line >= self.line_count() {
            return self.content.len_chars();
        }
        
        let line_start = self.content.line_to_char(cursor.line);
        let line_content = self.content.line(cursor.line);
        let line_len = line_content.len_chars();
        
        if line_len == 0 {
            line_start
        } else {
            line_start + cursor.col.min(line_len)
        }
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
        match self.buffer_type {
            BufferType::Terminal => Some("[Terminal]".to_string()),
            BufferType::File => self.file_path
                .as_ref()
                .and_then(|path| path.file_name())
                .and_then(|name| name.to_str())
                .map(|s| s.to_string()),
        }
    }

    pub fn is_terminal(&self) -> bool {
        matches!(self.buffer_type, BufferType::Terminal)
    }

    pub fn execute_terminal_command(&mut self, command: &str) -> Result<()> {
        if let Some(ref mut terminal_output) = self.terminal_output {
            if let Err(e) = terminal_output.execute_command(command) {
                return Err(anyhow::anyhow!("Terminal command failed: {}", e));
            }
        }
        Ok(())
    }

    pub fn handle_terminal_input_char(&mut self, ch: char) {
        if let Some(ref mut terminal_output) = self.terminal_output {
            terminal_output.handle_input_char(ch);
        }
    }

    pub fn handle_terminal_backspace(&mut self) {
        if let Some(ref mut terminal_output) = self.terminal_output {
            terminal_output.handle_backspace();
        }
    }

    pub fn handle_terminal_enter(&mut self) -> Result<()> {
        if let Some(ref mut terminal_output) = self.terminal_output {
            if let Err(e) = terminal_output.handle_enter() {
                return Err(anyhow::anyhow!("Terminal command failed: {}", e));
            }
        }
        Ok(())
    }

    pub fn handle_terminal_history_up(&mut self) {
        if let Some(ref mut terminal_output) = self.terminal_output {
            terminal_output.history_up();
        }
    }

    pub fn handle_terminal_history_down(&mut self) {
        if let Some(ref mut terminal_output) = self.terminal_output {
            terminal_output.history_down();
        }
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

    pub fn save_state(&mut self, cursor: &Cursor) {
        // Only save state for file buffers, not terminals
        if matches!(self.buffer_type, BufferType::Terminal) {
            return;
        }

        let state = UndoState {
            content: self.content.clone(),
            cursor: cursor.clone(),
        };
        
        // Limit undo stack size to prevent memory issues
        const MAX_UNDO_STEPS: usize = 100;
        if self.undo_stack.len() >= MAX_UNDO_STEPS {
            self.undo_stack.pop_front();
        }
        
        self.undo_stack.push_back(state);
        // Clear redo stack when new action is performed
        self.redo_stack.clear();
    }

    pub fn undo(&mut self) -> Option<Cursor> {
        if matches!(self.buffer_type, BufferType::Terminal) {
            return None;
        }

        if let Some(state) = self.undo_stack.pop_back() {
            // Save current state to redo stack
            let current_state = UndoState {
                content: self.content.clone(),
                cursor: Cursor::new(), // Will be updated by caller
            };
            self.redo_stack.push_back(current_state);
            
            // Restore previous state
            self.content = state.content;
            self.is_modified = true;
            Some(state.cursor)
        } else {
            None
        }
    }

    pub fn redo(&mut self) -> Option<Cursor> {
        if matches!(self.buffer_type, BufferType::Terminal) {
            return None;
        }

        if let Some(state) = self.redo_stack.pop_back() {
            // Save current state to undo stack
            let current_state = UndoState {
                content: self.content.clone(),
                cursor: Cursor::new(), // Will be updated by caller
            };
            self.undo_stack.push_back(current_state);
            
            // Restore redo state
            self.content = state.content;
            self.is_modified = true;
            Some(state.cursor)
        } else {
            None
        }
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty() && !matches!(self.buffer_type, BufferType::Terminal)
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty() && !matches!(self.buffer_type, BufferType::Terminal)
    }
}
