use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::terminal;
use crate::app::App;
use crate::editor::Mode;
use crate::Result;
use std::time::Duration;

pub struct EventHandler {
    pub should_quit: bool,
}

impl EventHandler {
    pub fn new() -> Self {
        Self {
            should_quit: false,
        }
    }

    pub fn handle_events(&mut self, app: &mut App) -> Result<()> {
        if event::poll(Duration::from_millis(16))? {
            match event::read()? {
                Event::Key(key_event) => {
                    if key_event.kind == KeyEventKind::Press {
                        self.handle_key_event(app, key_event)?;
                    }
                }
                Event::Resize(_, _) => {

                }
                _ => {}
            }
        }
        Ok(())
    }

    fn handle_key_event(&mut self, app: &mut App, key_event: KeyEvent) -> Result<()> {
       if app.help_window.visible && key_event.code == KeyCode::Esc {
            app.hide_help();
            return Ok(());
        }

        if app.help_window.visible {
            match key_event.code {
                KeyCode::Up => {
                    app.help_window.scroll_up();
                    return Ok(());
                }
                KeyCode::Down => {
                    let viewport_height = 20;
                    app.help_window.scroll_down(viewport_height);
                    return Ok(());
                }
                _ => {}
            }
        }

        match app.mode {
            Mode::Normal => self.handle_normal_mode(app, key_event),
            Mode::Insert => self.handle_insert_mode(app, key_event),
            Mode::Visual => self.handle_visual_mode(app, key_event),
            Mode::Command => self.handle_command_mode(app, key_event),
        }
    }

    fn get_viewport_width(&self, app: &App) -> Result<usize> {
        let (width, _) = terminal::size()?;
        let mut viewport_width = width as usize;

        if app.config.editor.line_numbers {
            viewport_width = viewport_width.saturating_sub(5);
        }

        if app.file_explorer.visible {
            viewport_width = viewport_width.saturating_sub(30);
        }

        Ok(viewport_width.max(20))
    }

    fn handle_normal_mode(&mut self, app: &mut App, key_event: KeyEvent) -> Result<()> {
        if app.file_explorer.visible {
            return self.handle_file_explorer_mode(app, key_event);
        }

        // Handle terminal buffer input
        if app.current_buffer().is_terminal() {
            match key_event.code {
                KeyCode::Esc => {
                    app.switch_to_previous_buffer();
                    return Ok(());
                }
                KeyCode::Char(ch) => {
                    app.current_buffer_mut().handle_terminal_input_char(ch);
                    return Ok(());
                }
                KeyCode::Backspace => {
                    app.current_buffer_mut().handle_terminal_backspace();
                    return Ok(());
                }
                KeyCode::Enter => {
                    if let Err(e) = app.current_buffer_mut().handle_terminal_enter() {
                        app.set_error_message(format!("Terminal error: {}", e));
                    }
                    return Ok(());
                }
                KeyCode::Up => {
                    app.current_buffer_mut().handle_terminal_history_up();
                    return Ok(());
                }
                KeyCode::Down => {
                    app.current_buffer_mut().handle_terminal_history_down();
                    return Ok(());
                }
                _ => {}
            }
        }

        let viewport_width = self.get_viewport_width(app)?;

        match key_event.code {
            KeyCode::Char('h') | KeyCode::Left => {
                let buffer = app.current_buffer().clone();
                app.cursor.move_left(&buffer);
            }
            KeyCode::Char('j') | KeyCode::Down => {
                let buffer = app.current_buffer().clone();
                app.cursor.move_down_visual(&buffer, viewport_width);
            }
            KeyCode::Char('k') | KeyCode::Up => {
                let buffer = app.current_buffer().clone();
                app.cursor.move_up_visual(&buffer, viewport_width);
            }
            KeyCode::Char('l') | KeyCode::Right => {
                let buffer = app.current_buffer().clone();
                app.cursor.move_right(&buffer);
            }
            KeyCode::Char('w') => {
                let buffer = app.current_buffer().clone();
                app.cursor.move_word_forward(&buffer);
            }
            KeyCode::Char('b') => {
                let buffer = app.current_buffer().clone();
                app.cursor.move_word_backward(&buffer);
            }
            KeyCode::Char('0') => {
                app.cursor.move_line_start();
            }
            KeyCode::Char('$') => {
                let buffer = app.current_buffer().clone();
                app.cursor.move_line_end(&buffer);
            }
            KeyCode::Char('g') => {
                app.cursor.move_file_start();
            }
            KeyCode::Char('G') => {
                let buffer = app.current_buffer().clone();
                app.cursor.move_file_end(&buffer);
            }

            KeyCode::F(2) => {
                app.file_explorer.toggle_visibility();
            }

            KeyCode::Char('i') => {
                app.mode = Mode::Insert;
            }
            KeyCode::Char('a') => {
                let buffer = app.current_buffer().clone();
                app.cursor.move_right(&buffer);
                app.mode = Mode::Insert;
            }
            KeyCode::Char('o') => {
                let buffer = app.current_buffer().clone();
                app.cursor.move_line_end(&buffer);

                let cursor_line = app.cursor.line;
                let cursor_col = app.cursor.col;

                let buffer = app.current_buffer_mut();
                buffer.insert_char(cursor_line, cursor_col, '\n');

                app.cursor.line += 1;
                app.cursor.col = 0;
                app.cursor.desired_col = 0;
                app.mode = Mode::Insert;
            }
            KeyCode::Char('v') => {
                app.mode = Mode::Visual;
            }
            KeyCode::Char(':') => {
                app.mode = Mode::Command;
                app.command_line.clear();
            }

            KeyCode::Char('x') => {
                let cursor_line = app.cursor.line;
                let cursor_col = app.cursor.col;
                let buffer = app.current_buffer_mut();
                buffer.delete_char(cursor_line, cursor_col);
            }
            KeyCode::Char('d') => {
                if key_event.modifiers.contains(KeyModifiers::SHIFT) {
                    let cursor_line = app.cursor.line;
                    let buffer = app.current_buffer_mut();
                    if buffer.line_count() > 1 {
                        buffer.delete_range(cursor_line, 0, cursor_line + 1, 0);
                    }
                }
            }

            KeyCode::Char('q') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                app.quit();
            }

            _ => {}
        }

        let buffer = app.current_buffer().clone();
        app.cursor.clamp_to_buffer(&buffer);
        Ok(())
    }

    fn handle_file_explorer_mode(&mut self, app: &mut App, key_event: KeyEvent) -> Result<()> {
        match key_event.code {
            KeyCode::Char('j') | KeyCode::Down => {
                app.file_explorer.move_down();
            }
            KeyCode::Char('k') | KeyCode::Up => {
                app.file_explorer.move_up();
            }
            KeyCode::Enter | KeyCode::Char('l') | KeyCode::Right => {
                if let Some(file_path) = app.file_explorer.select_current()? {
                    if let Err(e) = app.open_file(file_path.clone()) {
                        app.set_status_message(format!("Error opening file: {}", e));
                    } else {
                        app.set_status_message(format!("Opened: {}", file_path.display()));
                        app.file_explorer.toggle_visibility();
                    }
                }
            }
            KeyCode::Char('h') | KeyCode::Left => {
                match app.file_explorer.go_to_parent() {
                    Ok(()) => {
                    }
                    Err(e) => {
                        app.set_status_message(format!("{}", e));
                    }
                }
            }
            KeyCode::F(2) | KeyCode::Esc => {
                app.file_explorer.toggle_visibility();
            }
            KeyCode::Char('r') => {
                match app.file_explorer.refresh() {
                    Ok(()) => {
                        app.set_status_message("Directory refreshed".to_string());
                    }
                    Err(e) => {
                        app.set_status_message(format!("Error refreshing: {}", e));
                    }
                }
            }
            KeyCode::Char('p') => {
                let current_path = app.file_explorer.get_current_path();
                app.set_status_message(format!("Current: {}", current_path.display()));
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_insert_mode(&mut self, app: &mut App, key_event: KeyEvent) -> Result<()> {
        let viewport_width = self.get_viewport_width(app).unwrap_or(80);

        match key_event.code {
            KeyCode::Esc => {
                app.mode = Mode::Normal;
            }

            KeyCode::Left => {
                let buffer = app.current_buffer().clone();
                app.cursor.move_left(&buffer);
            }
            KeyCode::Right => {
                let buffer = app.current_buffer().clone();
                app.cursor.move_right(&buffer);
            }
            KeyCode::Up => {
                let buffer = app.current_buffer().clone();
                app.cursor.move_up_visual(&buffer, viewport_width);
            }
            KeyCode::Down => {
                let buffer = app.current_buffer().clone();
                app.cursor.move_down_visual(&buffer, viewport_width);
            }
            KeyCode::Home => {
                app.cursor.move_line_start();
            }
            KeyCode::End => {
                let buffer = app.current_buffer().clone();
                app.cursor.move_line_end(&buffer);
            }
            KeyCode::Char(c) => {
                let cursor_line = app.cursor.line;
                let cursor_col = app.cursor.col;
                let buffer = app.current_buffer_mut();
                buffer.insert_char(cursor_line, cursor_col, c);

                let buffer = app.current_buffer().clone();
                app.cursor.move_right(&buffer);
            }
            KeyCode::Enter => {
                let cursor_line = app.cursor.line;
                let cursor_col = app.cursor.col;
                let buffer = app.current_buffer_mut();
                buffer.insert_char(cursor_line, cursor_col, '\n');

                let buffer = app.current_buffer().clone();
                app.cursor.move_down(&buffer);
                app.cursor.move_line_start();
            }
            KeyCode::Backspace => {
                if app.cursor.col > 0 {
                    let buffer = app.current_buffer().clone();
                    app.cursor.move_left(&buffer);

                    let cursor_line = app.cursor.line;
                    let cursor_col = app.cursor.col;
                    let buffer = app.current_buffer_mut();
                    buffer.delete_char(cursor_line, cursor_col);
                } else if app.cursor.line > 0 {
                    let prev_line_idx = app.cursor.line - 1;
                    let buffer = app.current_buffer().clone();
                    let prev_line_len = buffer.line_len(prev_line_idx);

                    app.cursor.line -= 1;
                    app.cursor.col = prev_line_len;
                    app.cursor.desired_col = app.cursor.col;

                    let cursor_line = app.cursor.line;
                    let cursor_col = app.cursor.col;
                    let buffer = app.current_buffer_mut();
                    buffer.delete_char(cursor_line, cursor_col);
                }
            }
            KeyCode::Delete => {
                let cursor_line = app.cursor.line;
                let cursor_col = app.cursor.col;
                let buffer = app.current_buffer_mut();
                buffer.delete_char(cursor_line, cursor_col);
            }
            KeyCode::Tab => {
                let tab_size = app.config.editor.tab_size;
                let spaces = " ".repeat(tab_size);
                let cursor_line = app.cursor.line;
                let cursor_col = app.cursor.col;

                let buffer = app.current_buffer_mut();
                buffer.insert_str(cursor_line, cursor_col, &spaces);

                app.cursor.col += tab_size;
                app.cursor.desired_col = app.cursor.col;
            }
            _ => {}
        }

        let buffer = app.current_buffer().clone();
        app.cursor.clamp_to_buffer(&buffer);
        Ok(())
    }

    fn handle_visual_mode(&mut self, app: &mut App, key_event: KeyEvent) -> Result<()> {
        match key_event.code {
            KeyCode::Esc => {
                app.mode = Mode::Normal;
            }
            _ => {
                self.handle_normal_mode(app, key_event)?;
            }
        }
        Ok(())
    }

    fn handle_command_mode(&mut self, app: &mut App, key_event: KeyEvent) -> Result<()> {
        match key_event.code {
            KeyCode::Esc => {
                app.mode = Mode::Normal;
                app.command_line.clear();
                app.clear_error_message();
            }
            KeyCode::Enter => {
                let command = app.command_line.clone();
                app.command_line.clear();
                app.mode = Mode::Normal;
                
                if app.search_state.is_active && command.is_empty() {
                    if key_event.modifiers.contains(KeyModifiers::SHIFT) {
                        app.search_previous();
                    } else {
                        app.search_next();
                    }
                } else {
                    self.execute_command(app, &command)?;
                }
            }
            KeyCode::Char(c) => {
                app.command_line.push(c);
                app.clear_error_message(); 
            }
            KeyCode::Backspace => {
                app.command_line.pop();
                app.clear_error_message(); 
            }
            _ => {}
        }
        Ok(())
    }

    fn execute_command(&mut self, app: &mut App, command: &str) -> Result<()> {
        let parts: Vec<&str> = command.trim().split_whitespace().collect();
        if parts.is_empty() {
            return Ok(());
        }

        app.clear_error_message();

        match parts[0] {
            "q" | "quit" => {
                app.quit();
            }
            "w" | "write" => {
                if let Err(e) = app.current_buffer_mut().save() {
                    app.set_error_message(format!("Error saving: {}", e));
                } else {
                    app.set_status_message("File saved".to_string());
                }
            }
            "wq" => {
                if let Err(e) = app.current_buffer_mut().save() {
                    app.set_error_message(format!("Error saving: {}", e));
                } else {
                    app.quit();
                }
            }
            "e" | "edit" => {
                if parts.len() > 1 {
                    let path = std::path::PathBuf::from(parts[1]);
                    let path_display = path.display().to_string();
                    if let Err(e) = app.open_file(path) {
                        app.set_error_message(format!("Error opening file: {}", e));
                    } else {
                        app.set_status_message(format!("Opened: {}", path_display));
                    }
                } else {
                    app.set_error_message("Usage: :e <filename>".to_string());
                }
            }
            "theme" => {
                if parts.len() > 1 {
                    let theme_path = std::path::PathBuf::from(parts[1]);
                    
                    if let Some(extension) = theme_path.extension() {
                        if extension != "nctheme" {
                            app.set_error_message("Theme files must have .nctheme extension".to_string());
                            return Ok(());
                        }
                    } else {
                        app.set_error_message("Theme files must have .nctheme extension".to_string());
                        return Ok(());
                    }
                    
                    match app.config.set_theme(&theme_path) {
                        Ok(()) => {
                            app.set_status_message(format!("Theme loaded: {}", app.config.current_theme.name));
                        }
                        Err(e) => {
                            app.set_error_message(format!("Error loading theme: {}", e));
                        }
                    }
                } else {
                    app.set_error_message("Usage: :theme <path-to-theme.nctheme>".to_string());
                }
            }
            "find" | "f" => {
                if parts.len() > 1 {
                    let query = parts[1..].join(" ");
                    app.search(&query);
                } else {
                    app.set_error_message("Usage: :find <pattern>".to_string());
                }
            }
            "findnext" | "fn" => {
                if app.search_state.is_active {
                    app.search_next();
                } else {
                    app.set_error_message("No active search. Use :find <pattern> first".to_string());
                }
            }
            "findprev" | "fp" => {
                if app.search_state.is_active {
                    app.search_previous();
                } else {
                    app.set_error_message("No active search. Use :find <pattern> first".to_string());
                }
            }
            "goto" | "g" => {
                if parts.len() > 1 {
                    if let Ok(line_num) = parts[1].parse::<usize>() {
                        let buffer = app.current_buffer();
                        if line_num > 0 && line_num <= buffer.line_count() {
                            app.cursor.line = line_num - 1; 
                            app.cursor.col = 0;
                            app.cursor.desired_col = 0;
                            app.set_status_message(format!("Jumped to line {}", line_num));
                        } else {
                            app.set_error_message(format!("Line {} out of range (1-{})", line_num, buffer.line_count()));
                        }
                    } else {
                        app.set_error_message("Usage: :goto <line_number>".to_string());
                    }
                } else {
                    app.set_error_message("Usage: :goto <line_number>".to_string());
                }
            }
            "help" | "h" => {
                self.show_help(app);
            }
            "set" => {
                if parts.len() > 1 {
                    match parts[1] {
                        "numbers" => {
                            app.config.editor.line_numbers = true;
                            app.set_status_message("Line numbers enabled".to_string());
                        }
                        "nonumbers" => {
                            app.config.editor.line_numbers = false;
                            app.set_status_message("Line numbers disabled".to_string());
                        }
                        "syntax" => {
                            app.config.editor.syntax_highlighting = true;
                            app.set_status_message("Syntax highlighting enabled".to_string());
                        }
                        "nosyntax" => {
                            app.config.editor.syntax_highlighting = false;
                            app.set_status_message("Syntax highlighting disabled".to_string());
                        }
                        _ => {
                            app.set_error_message(format!("Unknown setting: {}", parts[1]));
                        }
                    }
                } else {
                    app.set_error_message("Usage: :set <option> (numbers, nonumbers, syntax, nosyntax)".to_string());
                }
            }
            "clear" => {
                app.search_state.clear();
                app.set_status_message("Search cleared".to_string());
            }
            "cmd" => {
                app.open_terminal();
                if parts.len() > 1 {
                    let command = parts[1..].join(" ");
                    if let Err(e) = app.current_buffer_mut().execute_terminal_command(&command) {
                        app.set_error_message(format!("Command error: {}", e));
                    }
                } else {
                    app.set_status_message("Terminal opened".to_string());
                }
            }
            _ => {
                app.set_error_message(format!("Unknown command: {}. Type :help for available commands", parts[0]));
            }
        }
        Ok(())
    }

    fn show_help(&self, app: &mut App) {
        app.show_help();
    }
}

impl Default for EventHandler {
    fn default() -> Self {
        Self::new()
    }
}
