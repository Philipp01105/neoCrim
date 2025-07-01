use crate::editor::{Buffer, Clipboard, Cursor, Mode, Selection};
use crate::config::Config;
use crate::ui::components::FileExplorer;
use crate::syntax::SyntaxHighlighter;
use crate::Result;
use std::path::PathBuf;
use std::time::Instant;

pub struct App {
    pub should_quit: bool,
    pub buffers: Vec<Buffer>,
    pub current_buffer: usize,
    pub cursor: Cursor,
    pub selection: Selection,
    pub mode: Mode,
    pub config: Config,
    pub status_message: Option<String>,
    pub command_line: String,
    pub file_explorer: FileExplorer,
    pub syntax_highlighter: SyntaxHighlighter,
    pub search_state: SearchState,
    pub error_message: Option<String>,
    pub help_window: HelpWindow,
    pub cursor_blink_state: bool,
    pub last_cursor_blink: Instant,
    pub horizontal_scroll_offset: usize,
}

#[derive(Debug, Clone)]
pub struct HelpWindow {
    pub visible: bool,
    pub scroll_offset: usize,
    pub content: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct SearchState {
    pub query: String,
    pub results: Vec<SearchResult>,
    pub current_result: usize,
    pub is_active: bool,
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub line: usize,
    pub col: usize,
    pub match_length: usize,
}

impl App {
    pub fn new() -> Result<Self> {
        let config = Config::load()?;
        let file_explorer = FileExplorer::new(".")?;
        let syntax_highlighter = SyntaxHighlighter::new();
        
        Ok(Self {
            should_quit: false,
            buffers: vec![Buffer::terminal()],
            current_buffer: 0,
            cursor: Cursor::new(),
            selection: Selection::new(),
            mode: Mode::Normal,
            config,
            status_message: None,
            command_line: String::new(),
            file_explorer,
            syntax_highlighter,
            search_state: SearchState::new(),
            error_message: None,
            help_window: HelpWindow::new(),
            cursor_blink_state: true,
            last_cursor_blink: Instant::now(),
            horizontal_scroll_offset: 0,
        })
    }

    pub fn open_file(&mut self, path: PathBuf) -> Result<()> {
        let buffer = Buffer::from_file(&path)?;
        self.buffers.push(buffer);
        self.current_buffer = self.buffers.len() - 1;
        Ok(())
    }

    pub fn open_or_create_file(&mut self, filename: &str) -> Result<()> {
        let path = if std::path::Path::new(filename).is_absolute() {
            PathBuf::from(filename)
        } else {
            self.file_explorer.get_current_path().join(filename)
        };

        if path.exists() {
            let buffer = Buffer::from_file(&path)?;
            self.buffers.push(buffer);
            self.current_buffer = self.buffers.len() - 1;
            Ok(())
        } else {
            let buffer = Buffer::new_file(&path);
            self.buffers.push(buffer);
            self.current_buffer = self.buffers.len() - 1;
            Ok(())
        }
    }

    pub fn get_current_directory(&self) -> &std::path::Path {
        self.file_explorer.get_current_path()
    }

    pub fn navigate_explorer_to_current_file(&mut self) -> Result<()> {
        let file_path = self.current_buffer().file_path.clone();
        if let Some(file_path) = file_path {
            self.file_explorer.navigate_to(&file_path)?;
        }
        Ok(())
    }

    pub fn copy_selection(&self) {
        if self.selection.active {
            let buffer = self.current_buffer();
            let selected_text = buffer.get_selected_text(&self.selection);
            if !selected_text.is_empty() {
                Clipboard::set_text(selected_text);
            }
        }
    }

    pub fn cut_selection(&mut self) {
        if self.selection.active {
            let selection_copy = self.selection.clone();
            let buffer = self.current_buffer_mut();
            let deleted_text = buffer.delete_selected_text(&selection_copy);
            if !deleted_text.is_empty() {
                Clipboard::set_text(deleted_text);
                if let Some((start, _)) = selection_copy.get_range() {
                    self.cursor = start;
                }
                self.selection.clear();
            }
        }
    }

    pub fn paste(&mut self) {
        let text = Clipboard::get_text();
        if !text.is_empty() {
            if self.selection.active {
                self.cut_selection();
            }
            
            let cursor_copy = self.cursor;
            let buffer = self.current_buffer_mut();
            buffer.insert_text_at_cursor(&cursor_copy, &text);
            
            let lines: Vec<&str> = text.lines().collect();
            if lines.len() == 1 {
                self.cursor.col += text.len();
            } else {
                self.cursor.line += lines.len() - 1;
                self.cursor.col = lines.last().map(|line| line.len()).unwrap_or(0);
            }
            self.cursor.desired_col = self.cursor.col;
        }
    }

    pub fn start_selection(&mut self) {
        self.selection.start_selection(self.cursor);
    }

    pub fn update_selection(&mut self) {
        self.selection.update_selection(self.cursor);
    }

    pub fn clear_selection(&mut self) {
        self.selection.clear();
    }

    pub fn delete_selection(&mut self) -> bool {
        if !self.selection.active {
            return false;
        }

        if let Some((start, end)) = self.selection.get_range() {
            self.save_undo_state();
            self.current_buffer_mut().delete_range(start.line, start.col, end.line, end.col);
            self.cursor = start;
            self.clear_selection();
            true
        } else {
            false
        }
    }

    pub fn open_terminal(&mut self) {
        for (i, buffer) in self.buffers.iter().enumerate() {
            if buffer.is_terminal() {
                self.current_buffer = i;
                return;
            }
        }
        
        let terminal_buffer = Buffer::terminal();
        self.buffers.push(terminal_buffer);
        self.current_buffer = self.buffers.len() - 1;
    }

    pub fn switch_to_previous_buffer(&mut self) {
        if self.buffers.len() > 1 {
            for (i, buffer) in self.buffers.iter().enumerate() {
                if i != self.current_buffer && !buffer.is_terminal() {
                    self.current_buffer = i;
                    return;
                }
            }
        }
    }

    pub fn current_buffer(&self) -> &Buffer {
        &self.buffers[self.current_buffer]
    }

    pub fn current_buffer_mut(&mut self) -> &mut Buffer {
        &mut self.buffers[self.current_buffer]
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    pub fn update_cursor_blink(&mut self) {
        if self.config.ui.cursor_blink {
            let now = Instant::now();
            if now.duration_since(self.last_cursor_blink).as_millis() > 500 {
                self.cursor_blink_state = !self.cursor_blink_state;
                self.last_cursor_blink = now;
            }
        } else {
            self.cursor_blink_state = true;
        }
    }

    pub fn should_show_cursor(&self) -> bool {
        if self.config.ui.cursor_blink {
            self.cursor_blink_state
        } else {
            true
        }
    }

    pub fn set_status_message(&mut self, message: String) {
        self.status_message = Some(message);
    }

    pub fn set_error_message(&mut self, message: String) {
        self.error_message = Some(message);
    }

    pub fn clear_error_message(&mut self) {
        self.error_message = None;
    }

    pub fn save_undo_state(&mut self) {
        let cursor = self.cursor.clone();
        self.current_buffer_mut().save_state(&cursor);
    }

    pub fn undo(&mut self) {
        if let Some(cursor) = self.current_buffer_mut().undo() {
            self.cursor = cursor;
            let buffer = self.current_buffer().clone();
            self.cursor.clamp_to_buffer(&buffer);
            self.set_status_message("Undo".to_string());
        } else {
            self.set_status_message("Nothing to undo".to_string());
        }
    }

    pub fn redo(&mut self) {
        if let Some(cursor) = self.current_buffer_mut().redo() {
            self.cursor = cursor;
            let buffer = self.current_buffer().clone();
            self.cursor.clamp_to_buffer(&buffer);
            self.set_status_message("Redo".to_string());
        } else {
            self.set_status_message("Nothing to redo".to_string());
        }
    }

    pub fn search(&mut self, query: &str) {
        self.search_state.search(query, &self.buffers[self.current_buffer]);
        if !self.search_state.results.is_empty() {
            self.search_state.goto_current_result(&mut self.cursor);
            self.set_status_message(format!("Found {} matches", self.search_state.results.len()));
        } else {
            self.set_error_message(format!("Pattern not found: {}", query));
        }
    }

    pub fn search_next(&mut self) {
        if self.search_state.next() {
            self.search_state.goto_current_result(&mut self.cursor);
            self.set_status_message(format!("Match {} of {}", 
                self.search_state.current_result + 1, 
                self.search_state.results.len()));
        }
    }

    pub fn search_previous(&mut self) {
        if self.search_state.previous() {
            self.search_state.goto_current_result(&mut self.cursor);
            self.set_status_message(format!("Match {} of {}", 
                self.search_state.current_result + 1, 
                self.search_state.results.len()));
        }
    }

    pub fn show_help(&mut self) {
        self.help_window.show();
    }

    pub fn hide_help(&mut self) {
        self.help_window.hide();
    }

    pub fn update_horizontal_scroll(&mut self, viewport_width: usize) {
        if self.config.editor.wrap_lines {
            self.horizontal_scroll_offset = 0;
            return;
        }

        let line_number_width = if self.config.editor.line_numbers || self.config.editor.relative_line_numbers {
            if self.config.editor.relative_line_numbers { 5 } else { 4 }
        } else {
            0
        };

        let content_width = viewport_width.saturating_sub(line_number_width);
        let scroll_margin = 5;

        if self.cursor.col < self.horizontal_scroll_offset {
            self.horizontal_scroll_offset = self.cursor.col;
        } else if self.cursor.col >= self.horizontal_scroll_offset + content_width {
            self.horizontal_scroll_offset = self.cursor.col - content_width + 1;
        } else if self.cursor.col < self.horizontal_scroll_offset + scroll_margin && self.horizontal_scroll_offset > 0 {
            self.horizontal_scroll_offset = self.cursor.col.saturating_sub(scroll_margin);
        } else if self.cursor.col >= self.horizontal_scroll_offset + content_width - scroll_margin {
            self.horizontal_scroll_offset = self.cursor.col + scroll_margin - content_width + 1;
        }
    }

    pub fn get_horizontal_scroll_offset(&self) -> usize {
        if self.config.editor.wrap_lines {
            0
        } else {
            self.horizontal_scroll_offset
        }
    }
}

impl SearchState {
    pub fn new() -> Self {
        Self {
            query: String::new(),
            results: Vec::new(),
            current_result: 0,
            is_active: false,
        }
    }

    pub fn search(&mut self, query: &str, buffer: &Buffer) {
        self.query = query.to_string();
        self.results.clear();
        self.current_result = 0;
        self.is_active = !query.is_empty();

        if query.is_empty() {
            return;
        }

        for line_idx in 0..buffer.line_count() {
            if let Some(line_content) = buffer.line(line_idx) {
                let mut start_pos = 0;
                while let Some(pos) = line_content[start_pos..].find(query) {
                    let actual_pos = start_pos + pos;
                    self.results.push(SearchResult {
                        line: line_idx,
                        col: actual_pos,
                        match_length: query.len(),
                    });
                    start_pos = actual_pos + 1;
                }
            }
        }
    }

    pub fn next(&mut self) -> bool {
        if self.results.is_empty() {
            return false;
        }
        self.current_result = (self.current_result + 1) % self.results.len();
        true
    }

    pub fn previous(&mut self) -> bool {
        if self.results.is_empty() {
            return false;
        }
        self.current_result = if self.current_result == 0 {
            self.results.len() - 1
        } else {
            self.current_result - 1
        };
        true
    }

    pub fn goto_current_result(&self, cursor: &mut Cursor) {
        if let Some(result) = self.results.get(self.current_result) {
            cursor.line = result.line;
            cursor.col = result.col;
            cursor.desired_col = result.col;
        }
    }

    pub fn clear(&mut self) {
        self.query.clear();
        self.results.clear();
        self.current_result = 0;
        self.is_active = false;
    }
}

impl HelpWindow {
    pub fn new() -> Self {
        let content = vec![
            "NeoCrim Editor - Help".to_string(),
            "".to_string(),
            "Available Commands:".to_string(),
            "".to_string(),
            "File Operations:".to_string(),
            "  :e <file>          - Edit/open file (creates if not exists)".to_string(),
            "  :w                 - Save current file".to_string(),
            "  :wq                - Save and quit".to_string(),
            "  :q                 - Quit editor".to_string(),
            "  :pwd               - Show current directory".to_string(),
            "  :cd <dir>          - Change current directory".to_string(),
            "  :explorer          - Toggle file explorer".to_string(),
            "  :refresh           - Refresh file explorer".to_string(),
            "".to_string(),
            "Search & Navigation:".to_string(),
            "  :find <pattern>    - Search for pattern".to_string(),
            "  :findnext          - Go to next search result".to_string(),
            "  :findprev          - Go to previous search result".to_string(),
            "  :goto <line>       - Jump to line number".to_string(),
            "  :clear             - Clear search results".to_string(),
            "".to_string(),
            "Configuration & Settings:".to_string(),
            "  :set               - Show all current settings".to_string(),
            "  :set all           - Show all settings with descriptions".to_string(),
            "  :set <option>?     - Show value of specific setting".to_string(),
            "".to_string(),
            "Line Numbers:".to_string(),
            "  :set nu            - Enable line numbers".to_string(),
            "  :set nonu          - Disable line numbers".to_string(),
            "  :set rnu           - Enable relative line numbers".to_string(),
            "  :set nornu         - Disable relative line numbers".to_string(),
            "  :set nu=true/false - Set line numbers directly".to_string(),
            "".to_string(),
            "Editor Options:".to_string(),
            "  :set ts=4          - Set tab size (1-16)".to_string(),
            "  :set et            - Use spaces instead of tabs".to_string(),
            "  :set noet          - Use tabs instead of spaces".to_string(),
            "  :set syntax        - Enable syntax highlighting".to_string(),
            "  :set nosyntax      - Disable syntax highlighting".to_string(),
            "  :set autosave      - Enable auto-save".to_string(),
            "  :set noautosave    - Disable auto-save".to_string(),
            "  :set wrap          - Enable line wrapping".to_string(),
            "  :set nowrap        - Disable line wrapping".to_string(),
            "  :set so=5          - Set scroll offset (0-20)".to_string(),
            "".to_string(),
            "UI Options:".to_string(),
            "  :set cursorblink=true/false    - Enable/disable cursor blinking".to_string(),
            "  :set statusline=true/false     - Show/hide status line".to_string(),
            "  :set commandline=true/false    - Show/hide command line".to_string(),
            "".to_string(),
            "Setting Shortcuts (Vim-style):".to_string(),
            "  nu/number          - Line numbers".to_string(),
            "  rnu/relativenumber - Relative line numbers".to_string(),
            "  ts/tabsize         - Tab size".to_string(),
            "  et/expandtab       - Insert tabs as spaces".to_string(),
            "  so/scrolloffset    - Scroll offset".to_string(),
            "".to_string(),
            "Themes:".to_string(),
            "  :theme list        - List all available themes".to_string(),
            "  :theme <name>      - Switch to built-in theme".to_string(),
            "  :theme <index>     - Switch to theme by index".to_string(),
            "  :theme <file.nctheme> - Load custom theme file".to_string(),
            "  :theme default <index> - Load default theme by index".to_string(),
            "".to_string(),
            "Help:".to_string(),
            "  :help              - Show this help window".to_string(),
            "".to_string(),
            "Movement (Normal Mode):".to_string(),
            "  h, j, k, l         - Move cursor left, down, up, right".to_string(),
            "  Arrow keys         - Move cursor".to_string(),
            "  Shift+Arrow keys   - Select text".to_string(),
            "  w                  - Jump to next word".to_string(),
            "  b                  - Jump to previous word".to_string(),
            "  g                  - Go to beginning of file".to_string(),
            "  G                  - Go to end of file".to_string(),
            "  0                  - Go to beginning of line".to_string(),
            "  $                  - Go to end of line".to_string(),
            "".to_string(),
            "Editing:".to_string(),
            "  i                  - Enter insert mode".to_string(),
            "  a                  - Enter insert mode after cursor".to_string(),
            "  o                  - Insert new line below and enter insert mode".to_string(),
            "  x                  - Delete character under cursor".to_string(),
            "".to_string(),
            "Clipboard Operations:".to_string(),
            "  Ctrl+C             - Copy selection".to_string(),
            "  Ctrl+X             - Cut selection".to_string(),
            "  Ctrl+V             - Paste from clipboard".to_string(),
            "  Ctrl+A             - Select all text".to_string(),
            "".to_string(),
            "Special Keys:".to_string(),
            "  Esc                - Return to normal mode / Close this help".to_string(),
            "  :                  - Enter command mode".to_string(),
            "  Enter              - Next search result (when searching)".to_string(),
            "  Shift+Enter        - Previous search result (when searching)".to_string(),
            "  F2                 - Toggle file explorer".to_string(),
            "  Ctrl+Q             - Quit editor".to_string(),
            "".to_string(),
            "Examples:".to_string(),
            "  :set rnu           - Enable relative line numbers".to_string(),
            "  :set ts=2          - Set tab size to 2 spaces".to_string(),
            "  :set rnu?          - Check if relative numbers are enabled".to_string(),
            "  :goto 42           - Jump to line 42 (works with relative numbers)".to_string(),
            "  :set et autosave   - Enable both expand tabs and auto-save".to_string(),
            "".to_string(),
            "Press ESC to close this help window".to_string(),
            "Use Up/Down arrow keys to scroll".to_string(),
        ];

        Self {
            visible: false,
            scroll_offset: 0,
            content,
        }
    }

    pub fn show(&mut self) {
        self.visible = true;
        self.scroll_offset = 0;
    }

    pub fn hide(&mut self) {
        self.visible = false;
    }

    pub fn scroll_up(&mut self) {
        if self.scroll_offset > 0 {
            self.scroll_offset -= 1;
        }
    }

    pub fn scroll_down(&mut self, viewport_height: usize) {
        let max_scroll = self.content.len().saturating_sub(viewport_height);
        if self.scroll_offset < max_scroll {
            self.scroll_offset += 1;
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new().unwrap()
    }
}
