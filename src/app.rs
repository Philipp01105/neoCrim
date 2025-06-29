use crate::editor::{Buffer, Cursor, Mode};
use crate::config::Config;
use crate::ui::components::FileExplorer;
use crate::syntax::SyntaxHighlighter;
use crate::Result;
use std::path::PathBuf;

pub struct App {
    pub should_quit: bool,
    pub buffers: Vec<Buffer>,
    pub current_buffer: usize,
    pub cursor: Cursor,
    pub mode: Mode,
    pub config: Config,
    pub status_message: Option<String>,
    pub command_line: String,
    pub file_explorer: FileExplorer,
    pub syntax_highlighter: SyntaxHighlighter,
    pub search_state: SearchState,
    pub error_message: Option<String>,
    pub help_window: HelpWindow,
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
            buffers: vec![Buffer::empty()],
            current_buffer: 0,
            cursor: Cursor::new(),
            mode: Mode::Normal,
            config,
            status_message: None,
            command_line: String::new(),
            file_explorer,
            syntax_highlighter,
            search_state: SearchState::new(),
            error_message: None,
            help_window: HelpWindow::new(),
        })
    }

    pub fn open_file(&mut self, path: PathBuf) -> Result<()> {
        let buffer = Buffer::from_file(&path)?;
        self.buffers.push(buffer);
        self.current_buffer = self.buffers.len() - 1;
        Ok(())
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

    pub fn set_status_message(&mut self, message: String) {
        self.status_message = Some(message);
    }

    pub fn set_error_message(&mut self, message: String) {
        self.error_message = Some(message);
    }

    pub fn clear_error_message(&mut self) {
        self.error_message = None;
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
            "  :e <file>          - Edit/open file".to_string(),
            "  :w                 - Save current file".to_string(),
            "  :wq                - Save and quit".to_string(),
            "  :q                 - Quit editor".to_string(),
            "".to_string(),
            "Search & Navigation:".to_string(),
            "  :find <pattern>    - Search for pattern".to_string(),
            "  :findnext          - Go to next search result".to_string(),
            "  :findprev          - Go to previous search result".to_string(),
            "  :goto <line>       - Jump to line number".to_string(),
            "  :clear             - Clear search results".to_string(),
            "".to_string(),
            "Settings:".to_string(),
            "  :set numbers       - Show line numbers".to_string(),
            "  :set nonumbers     - Hide line numbers".to_string(),
            "  :set syntax        - Enable syntax highlighting".to_string(),
            "  :set nosyntax      - Disable syntax highlighting".to_string(),
            "".to_string(),
            "Themes:".to_string(),
            "  :theme <file.nctheme> - Load theme file".to_string(),
            "".to_string(),
            "Help:".to_string(),
            "  :help              - Show this help window".to_string(),
            "".to_string(),
            "Movement (Normal Mode):".to_string(),
            "  h, j, k, l         - Move cursor left, down, up, right".to_string(),
            "  Arrow keys         - Move cursor".to_string(),
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
            "  d                  - Delete line (with Shift)".to_string(),
            "  v                  - Enter visual mode".to_string(),
            "".to_string(),
            "Special Keys:".to_string(),
            "  Esc                - Return to normal mode / Close this help".to_string(),
            "  :                  - Enter command mode".to_string(),
            "  Enter              - Next search result (when searching)".to_string(),
            "  Shift+Enter        - Previous search result (when searching)".to_string(),
            "  F2                 - Toggle file explorer".to_string(),
            "  Ctrl+Q             - Quit editor".to_string(),
            "".to_string(),
            "Press ESC to close this help window".to_string(),
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
