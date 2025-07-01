use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::terminal;
use crate::app::App;
use crate::editor::{Cursor, Mode};
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

        if key_event.modifiers.contains(KeyModifiers::CONTROL) {
            if key_event.modifiers.contains(KeyModifiers::SHIFT) {
               match key_event.code {
                    KeyCode::Left => {
                        if !app.selection.active {
                            app.start_selection();
                        }
                        let buffer = app.current_buffer().clone();
                        app.cursor.move_word_backward(&buffer);
                        app.update_selection();
                        return Ok(());
                    }
                    KeyCode::Right => {
                        if !app.selection.active {
                            app.start_selection();
                        }
                        let buffer = app.current_buffer().clone();
                        app.cursor.move_word_forward(&buffer);
                        app.update_selection();
                        return Ok(());
                    }
                    _ => {}
                }
            }
            
            match key_event.code {
                KeyCode::Char('c') => {
                    app.copy_selection();
                    if app.selection.active {
                        app.set_status_message("Copied selection".to_string());
                    }
                    return Ok(());
                }
                KeyCode::Char('x') => {
                    if app.selection.active {
                        app.cut_selection();
                        app.set_status_message("Cut selection".to_string());
                    }
                    return Ok(());
                }
                KeyCode::Char('v') => {
                    app.paste();
                    app.set_status_message("Pasted from clipboard".to_string());
                    return Ok(());
                }
                KeyCode::Char('z') => {
                    app.undo();
                    return Ok(());
                }
                KeyCode::Char('y') => {
                    app.redo();
                    return Ok(());
                }
                KeyCode::Char('a') => {
                    let line_count = app.current_buffer().line_count();
                    let last_line_text = if line_count > 0 {
                        app.current_buffer().line(line_count.saturating_sub(1))
                    } else {
                        None
                    };

                    if line_count > 0 {
                        app.selection.start_selection(Cursor::new());
                        let mut end_cursor = Cursor::new();
                        end_cursor.line = line_count.saturating_sub(1);
                        if let Some(last_line) = last_line_text {
                            end_cursor.col = last_line.len();
                        }
                        app.selection.update_selection(end_cursor);
                        app.set_status_message("Selected all text".to_string());
                    }
                    return Ok(());
                }
                _ => {}
            }
        }

        if key_event.modifiers.contains(KeyModifiers::SHIFT) {
            match key_event.code {
                KeyCode::Left => {
                    if !app.selection.active {
                        app.start_selection();
                    }
                    let buffer = app.current_buffer().clone();
                    app.cursor.move_left(&buffer);
                    app.update_selection();
                    app.update_horizontal_scroll(viewport_width);
                    return Ok(());
                }
                KeyCode::Right => {
                    if !app.selection.active {
                        app.start_selection();
                    }
                    let buffer = app.current_buffer().clone();
                    app.cursor.move_right(&buffer);
                    app.update_selection();
                    app.update_horizontal_scroll(viewport_width);
                    return Ok(());
                }
                KeyCode::Up => {
                    if !app.selection.active {
                        app.start_selection();
                    }
                    let buffer = app.current_buffer().clone();
                    app.cursor.move_up(&buffer);
                    app.update_selection();
                    app.update_horizontal_scroll(viewport_width);
                    return Ok(());
                }
                KeyCode::Down => {
                    if !app.selection.active {
                        app.start_selection();
                    }
                    let buffer = app.current_buffer().clone();
                    app.cursor.move_down(&buffer);
                    app.update_selection();
                    app.update_horizontal_scroll(viewport_width);
                    return Ok(());
                }
                KeyCode::Home => {
                    if !app.selection.active {
                        app.start_selection();
                    }
                    app.cursor.move_line_start();
                    app.update_selection();
                    app.update_horizontal_scroll(viewport_width);
                    return Ok(());
                }
                KeyCode::End => {
                    if !app.selection.active {
                        app.start_selection();
                    }
                    let buffer = app.current_buffer().clone();
                    app.cursor.move_line_end(&buffer);
                    app.update_selection();
                    app.update_horizontal_scroll(viewport_width);
                    return Ok(());
                }
                _ => {}
            }
        } else {
            if app.selection.active {
                match key_event.code {
                    KeyCode::Left | KeyCode::Right | KeyCode::Up | KeyCode::Down |
                    KeyCode::Char('h') | KeyCode::Char('j') | KeyCode::Char('k') | KeyCode::Char('l') => {
                        app.clear_selection();
                    }
                    _ => {}
                }
            }
        }

        match key_event.code {
            KeyCode::Char('h') | KeyCode::Left => {
                let buffer = app.current_buffer().clone();
                app.cursor.move_left(&buffer);
                app.update_horizontal_scroll(viewport_width);
            }
            KeyCode::Char('j') | KeyCode::Down => {
                if app.search_state.is_active && !app.search_state.results.is_empty() {
                    app.search_next();
                    return Ok(());
                }
                let buffer = app.current_buffer().clone();
                app.cursor.move_down_visual(&buffer, viewport_width);
                app.update_horizontal_scroll(viewport_width);
            }
            KeyCode::Char('k') | KeyCode::Up => {
                if app.search_state.is_active && !app.search_state.results.is_empty() {
                    app.search_previous();
                    return Ok(());
                }
                let buffer = app.current_buffer().clone();
                app.cursor.move_up_visual(&buffer, viewport_width);
                app.update_horizontal_scroll(viewport_width);
            }
            KeyCode::Char('l') | KeyCode::Right => {
                let buffer = app.current_buffer().clone();
                app.cursor.move_right(&buffer);
                app.update_horizontal_scroll(viewport_width);
            }
            KeyCode::Char('w') => {
                let buffer = app.current_buffer().clone();
                app.cursor.move_word_forward(&buffer);
                app.update_horizontal_scroll(viewport_width);
            }
            KeyCode::Char('b') => {
                let buffer = app.current_buffer().clone();
                app.cursor.move_word_backward(&buffer);
                app.update_horizontal_scroll(viewport_width);
            }
            KeyCode::Char('0') => {
                app.cursor.move_line_start();
                app.update_horizontal_scroll(viewport_width);
            }
            KeyCode::Char('$') => {
                let buffer = app.current_buffer().clone();
                app.cursor.move_line_end(&buffer);
                app.update_horizontal_scroll(viewport_width);
            }
            KeyCode::Char('g') => {
                app.cursor.move_file_start();
                app.update_horizontal_scroll(viewport_width);
            }
            KeyCode::Char('G') => {
                let buffer = app.current_buffer().clone();
                app.cursor.move_file_end(&buffer);
                app.update_horizontal_scroll(viewport_width);
            }

            KeyCode::F(2) => {
                app.file_explorer.toggle_visibility();
            }

            KeyCode::Char('i') => {
                app.save_undo_state();
                app.mode = Mode::Insert;
            }
            KeyCode::Char('a') => {
                app.save_undo_state();
                let buffer = app.current_buffer().clone();
                app.cursor.move_right(&buffer);
                app.mode = Mode::Insert;
            }
            KeyCode::Char('o') => {
                let buffer = app.current_buffer().clone();
                app.cursor.move_line_end(&buffer);

                let cursor_line = app.cursor.line;
                let cursor_col = app.cursor.col;

                app.save_undo_state();
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
                app.save_undo_state();
                let buffer = app.current_buffer_mut();
                buffer.delete_char(cursor_line, cursor_col);
            }
            KeyCode::Char('d') => {
                if key_event.modifiers.contains(KeyModifiers::SHIFT) {
                    let cursor_line = app.cursor.line;
                    app.save_undo_state();
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

        if key_event.modifiers.contains(KeyModifiers::CONTROL) {
            if key_event.modifiers.contains(KeyModifiers::SHIFT) {
                match key_event.code {
                    KeyCode::Left => {
                        if !app.selection.active {
                            app.start_selection();
                        }
                        let buffer = app.current_buffer().clone();
                        app.cursor.move_word_backward(&buffer);
                        app.update_selection();
                        return Ok(());
                    }
                    KeyCode::Right => {
                        if !app.selection.active {
                            app.start_selection();
                        }
                        let buffer = app.current_buffer().clone();
                        app.cursor.move_word_forward(&buffer);
                        app.update_selection();
                        return Ok(());
                    }
                    _ => {}
                }
            }
            
            match key_event.code {
                KeyCode::Char('z') => {
                    app.undo();
                    return Ok(());
                }
                KeyCode::Char('y') => {
                    app.redo();
                    return Ok(());
                }
                KeyCode::Char('c') => {
                    app.copy_selection();
                    if app.selection.active {
                        app.set_status_message("Copied selection".to_string());
                    }
                    return Ok(());
                }
                KeyCode::Char('x') => {
                    if app.selection.active {
                        app.cut_selection();
                        app.set_status_message("Cut selection".to_string());
                    }
                    return Ok(());
                }
                KeyCode::Char('v') => {
                    app.paste();
                    app.set_status_message("Pasted from clipboard".to_string());
                    return Ok(());
                }
                KeyCode::Char('a') => {
                    let line_count = app.current_buffer().line_count();
                    let last_line_text = if line_count > 0 {
                        app.current_buffer().line(line_count.saturating_sub(1))
                    } else {
                        None
                    };

                    if line_count > 0 {
                        app.selection.start_selection(Cursor::new());
                        let mut end_cursor = Cursor::new();
                        end_cursor.line = line_count.saturating_sub(1);
                        if let Some(last_line) = last_line_text {
                            end_cursor.col = last_line.len();
                        }
                        app.selection.update_selection(end_cursor);
                        app.set_status_message("Selected all text".to_string());
                    }
                    return Ok(());
                }
                _ => {}
            }
        }

        if key_event.modifiers.contains(KeyModifiers::SHIFT) {
            match key_event.code {
                KeyCode::Left => {
                    if !app.selection.active {
                        app.start_selection();
                    }
                    let buffer = app.current_buffer().clone();
                    app.cursor.move_left(&buffer);
                    app.update_selection();
                    app.update_horizontal_scroll(viewport_width);
                    return Ok(());
                }
                KeyCode::Right => {
                    if !app.selection.active {
                        app.start_selection();
                    }
                    let buffer = app.current_buffer().clone();
                    app.cursor.move_right(&buffer);
                    app.update_selection();
                    app.update_horizontal_scroll(viewport_width);
                    return Ok(());
                }
                KeyCode::Up => {
                    if !app.selection.active {
                        app.start_selection();
                    }
                    let buffer = app.current_buffer().clone();
                    app.cursor.move_up_visual(&buffer, viewport_width);
                    app.update_selection();
                    return Ok(());
                }
                KeyCode::Down => {
                    if !app.selection.active {
                        app.start_selection();
                    }
                    let buffer = app.current_buffer().clone();
                    app.cursor.move_down_visual(&buffer, viewport_width);
                    app.update_selection();
                    return Ok(());
                }
                KeyCode::Home => {
                    if !app.selection.active {
                        app.start_selection();
                    }
                    app.cursor.move_line_start();
                    app.update_selection();
                    app.update_horizontal_scroll(viewport_width);
                    return Ok(());
                }
                KeyCode::End => {
                    if !app.selection.active {
                        app.start_selection();
                    }
                    let buffer = app.current_buffer().clone();
                    app.cursor.move_line_end(&buffer);
                    app.update_selection();
                    app.update_horizontal_scroll(viewport_width);
                    return Ok(());
                }
                _ => {}
            }
        } else {
            if app.selection.active {
                match key_event.code {
                    KeyCode::Left | KeyCode::Right | KeyCode::Up | KeyCode::Down => {
                        app.clear_selection();
                    }
                    _ => {}
                }
            }
        }

        match key_event.code {
            KeyCode::Esc => {
                app.mode = Mode::Normal;
                app.clear_selection();
            }

            KeyCode::Left => {
                let buffer = app.current_buffer().clone();
                app.cursor.move_left(&buffer);
                app.update_horizontal_scroll(viewport_width);
            }
            KeyCode::Right => {
                let buffer = app.current_buffer().clone();
                app.cursor.move_right(&buffer);
                app.update_horizontal_scroll(viewport_width);
            }
            KeyCode::Up => {
                let buffer = app.current_buffer().clone();
                app.cursor.move_up_visual(&buffer, viewport_width);
                app.update_horizontal_scroll(viewport_width);
            }
            KeyCode::Down => {
                let buffer = app.current_buffer().clone();
                app.cursor.move_down_visual(&buffer, viewport_width);
                app.update_horizontal_scroll(viewport_width);
            }
            KeyCode::Home => {
                app.cursor.move_line_start();
                app.update_horizontal_scroll(viewport_width);
            }
            KeyCode::End => {
                let buffer = app.current_buffer().clone();
                app.cursor.move_line_end(&buffer);
                app.update_horizontal_scroll(viewport_width);
            }
            KeyCode::Char(c) => {
                app.delete_selection();
                
                app.save_undo_state();
                let cursor_line = app.cursor.line;
                let cursor_col = app.cursor.col;
                let buffer = app.current_buffer_mut();
                buffer.insert_char(cursor_line, cursor_col, c);

                let buffer = app.current_buffer().clone();
                app.cursor.move_right(&buffer);
                app.update_horizontal_scroll(viewport_width);
            }
            KeyCode::Enter => {
                app.delete_selection();
                
                let cursor_line = app.cursor.line;
                let cursor_col = app.cursor.col;
                app.save_undo_state();
                let buffer = app.current_buffer_mut();
                buffer.insert_char(cursor_line, cursor_col, '\n');

                let buffer = app.current_buffer().clone();
                app.cursor.move_down(&buffer);
                app.cursor.move_line_start();
                app.update_horizontal_scroll(viewport_width);
            }
            KeyCode::Backspace => {
                if app.delete_selection() {
                    app.update_horizontal_scroll(viewport_width);
                    return Ok(());
                }
                
                app.save_undo_state();
                if app.cursor.col > 0 {
                    let buffer = app.current_buffer().clone();
                    app.cursor.move_left(&buffer);

                    let cursor_line = app.cursor.line;
                    let cursor_col = app.cursor.col;
                    let buffer = app.current_buffer_mut();
                    buffer.delete_char(cursor_line, cursor_col);
                    app.update_horizontal_scroll(viewport_width);
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
                    app.update_horizontal_scroll(viewport_width);
                }
            }
            KeyCode::Delete => {
                if app.delete_selection() {
                    app.update_horizontal_scroll(viewport_width);
                    return Ok(());
                }
                
                app.save_undo_state(); // Save state before delete operation
                let cursor_line = app.cursor.line;
                let cursor_col = app.cursor.col;
                let buffer = app.current_buffer_mut();
                buffer.delete_char(cursor_line, cursor_col);
                app.update_horizontal_scroll(viewport_width);
            }
            KeyCode::Tab => {
                app.delete_selection();
                
                app.save_undo_state();
                let tab_size = app.config.editor.tab_size;
                let spaces = " ".repeat(tab_size);
                let cursor_line = app.cursor.line;
                let cursor_col = app.cursor.col;

                let buffer = app.current_buffer_mut();
                buffer.insert_str(cursor_line, cursor_col, &spaces);

                app.cursor.col += tab_size;
                app.cursor.desired_col = app.cursor.col;
                app.update_horizontal_scroll(viewport_width);
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
                    let filename = parts[1];
                    match app.open_or_create_file(filename) {
                        Ok(()) => {
                            let current_buffer = app.current_buffer();
                            if current_buffer.is_modified {
                                app.set_status_message(format!("Created new file: {}", filename));
                            } else {
                                app.set_status_message(format!("Opened: {}", filename));
                            }
                        }
                        Err(e) => {
                            app.set_error_message(format!("Error opening/creating file: {}", e));
                        }
                    }
                } else {
                    app.set_error_message("Usage: :e <filename>".to_string());
                }
            }
            "pwd" => {
                let current_dir = app.get_current_directory();
                app.set_status_message(format!("Current directory: {}", current_dir.display()));
            }
            "cd" => {
                if parts.len() > 1 {
                    let target_path = std::path::PathBuf::from(parts[1]);
                    match app.file_explorer.navigate_to(&target_path) {
                        Ok(()) => {
                            app.set_status_message(format!("Changed directory to: {}", target_path.display()));
                        }
                        Err(e) => {
                            app.set_error_message(format!("Error changing directory: {}", e));
                        }
                    }
                } else {
                    app.set_error_message("Usage: :cd <directory>".to_string());
                }
            }
            "explorer" => {
                app.file_explorer.toggle_visibility();
                let status = if app.file_explorer.visible { "shown" } else { "hidden" };
                app.set_status_message(format!("File explorer {}", status));
            }
            "refresh" => {
                match app.file_explorer.refresh() {
                    Ok(()) => {
                        app.set_status_message("File explorer refreshed".to_string());
                    }
                    Err(e) => {
                        app.set_error_message(format!("Error refreshing explorer: {}", e));
                    }
                }
            }
            "theme" => {
                if parts.len() > 1 {
                    if let Ok(index) = parts[1].parse::<usize>() {
                        match app.config.set_theme_by_index(index) {
                            Ok(()) => {
                                app.set_status_message(format!("Theme changed to: {}", app.config.current_theme.name));
                            }
                            Err(e) => {
                                app.set_error_message(format!("Error setting theme: {}", e));
                            }
                        }
                    } else if parts[1] == "list" {
                        let themes = app.config.list_available_themes();
                        let mut message = "Available themes:\n".to_string();
                        for (index, name, author, description) in themes {
                            message.push_str(&format!("  {}: {} by {} - {}\n", index, name, author, description));
                        }
                        app.set_status_message(message);
                    } else if parts[1] == "default" {
                        if parts.len() > 2 {
                            if let Ok(index) = parts[2].parse::<usize>() {
                                match app.config.set_default_theme_by_index(index) {
                                    Ok(()) => {
                                        app.set_status_message(format!("Default theme {} loaded: {}", index, app.config.current_theme.name));
                                    }
                                    Err(e) => {
                                        app.set_error_message(format!("Error setting default theme: {}", e));
                                    }
                                }
                            } else {
                                app.set_error_message("Usage: :theme default <index>".to_string());
                            }
                        } else {
                            let default_themes = app.config.get_default_themes();
                            let mut message = "Default themes (from /themes directory):\n".to_string();
                            for (index, theme_name) in default_themes.iter().enumerate() {
                                message.push_str(&format!("  {}: {}\n", index, theme_name));
                            }
                            message.push_str("Usage: :theme default <index>");
                            app.set_status_message(message);
                        }
                    } else {
                        match app.config.set_theme_by_name(parts[1]) {
                            Ok(()) => {
                                app.set_status_message(format!("Theme changed to: {}", app.config.current_theme.name));
                            }
                            Err(_) => {
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
                            }
                        }
                    }
                } else {
                    let current_theme = &app.config.current_theme.name;
                    let themes_count = app.config.theme_manager.theme_count();
                    app.set_status_message(format!(
                        "Current theme: {}\nUsage: :theme <name|index|list> or :theme default [index] or :theme <path.nctheme>\nAvailable themes: {} (use ':theme list' to see all)",
                        current_theme, themes_count
                    ));
                }
            }
            "goto" | "g" => {
                if parts.len() > 1 {
                    let arg = parts[1];
                    
                    if arg.ends_with('j') || arg.ends_with('k') {
                        let direction = arg.chars().last().unwrap();
                        let number_part = &arg[..arg.len()-1];
                        
                        if let Ok(steps) = number_part.parse::<usize>() {
                            let buffer = app.current_buffer();
                            match direction {
                                'j' => {
                                    app.cursor.line = (app.cursor.line + steps).min(buffer.line_count().saturating_sub(1));
                                }
                                'k' => {
                                    app.cursor.line = app.cursor.line.saturating_sub(steps);
                                }
                                _ => unreachable!()
                            }
                            app.cursor.col = 0;
                            app.cursor.desired_col = 0;
                            app.set_status_message(format!("Moved {} lines {}", steps, 
                                if direction == 'j' { "down" } else { "up" }));
                        } else {
                            app.set_error_message("Invalid number in goto command".to_string());
                        }
                    }
                    else if let Ok(line_num) = arg.parse::<usize>() {
                        let buffer = app.current_buffer();
                        if line_num > 0 && line_num <= buffer.line_count() {
                            if app.config.editor.relative_line_numbers {
                                if line_num <= app.cursor.line {
                                    app.cursor.line = app.cursor.line.saturating_sub(line_num);
                                } else {
                                    app.cursor.line = (app.cursor.line + line_num).min(buffer.line_count() - 1);
                                }
                            } else {
                                // Absolute line number
                                app.cursor.line = line_num - 1;
                            }
                            app.cursor.col = 0;
                            app.cursor.desired_col = 0;
                            app.set_status_message(format!("Jumped to line {}", line_num));
                        } else {
                            app.set_error_message(format!("Line {} out of range (1-{})", line_num, buffer.line_count()));
                        }
                    } else {
                        // Not a number, treat as search pattern
                        let query = parts[1..].join(" ");
                        app.search(&query);
                        if !app.search_state.results.is_empty() {
                            app.set_status_message(format!("Found {} matches - Use Up/Down arrows to navigate", app.search_state.results.len()));
                        }
                    }
                } else {
                    app.set_error_message("Usage: :goto <line_number> or :goto <number>j/k or :goto <search_pattern>".to_string());
                }
            }
            "help" | "h" => {
                self.show_help(app);
            }
            "set" => {
                if parts.len() == 1 {
                    let settings = app.config.get_all_settings_display();
                    app.set_status_message(settings.join("\n"));
                } else if parts.len() == 2 && parts[1] == "all" {
                    let mut settings = app.config.get_all_settings_display();
                    settings.insert(0, "All Settings with Descriptions:".to_string());
                    settings.push("".to_string());
                    settings.push("Setting Shortcuts (Vim-style):".to_string());
                    settings.push("  nu/number          - Line numbers".to_string());
                    settings.push("  rnu/relativenumber - Relative line numbers".to_string());
                    settings.push("  ts/tabsize         - Tab size".to_string());
                    settings.push("  et/expandtab       - Insert tabs as spaces".to_string());
                    settings.push("  so/scrolloffset    - Scroll offset".to_string());
                    app.set_status_message(settings.join("\n"));
                } else {
                    for i in 1..parts.len() {
                        let setting = parts[i];
                        
                        if setting.ends_with('?') {
                            let setting_name = &setting[..setting.len()-1];
                            let display = app.config.get_setting_display(setting_name);
                            app.set_status_message(display);
                            continue;
                        }
                        
                        if let Some(eq_pos) = setting.find('=') {
                            let (key, value) = setting.split_at(eq_pos);
                            let value = &value[1..]; 
                            if let Err(e) = self.handle_set_assignment(app, key, value) {
                                app.set_error_message(e.to_string());
                                return Ok(());
                            }
                            continue;
                        }
                        
                        if let Err(e) = self.handle_set_flag(app, setting) {
                            app.set_error_message(e.to_string());
                            return Ok(());
                        }
                    }
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

    fn handle_set_assignment(&self, app: &mut App, key: &str, value: &str) -> Result<()> {
        match key.to_lowercase().as_str() {
            "ts" | "tabsize" | "tab_size" => {
                if let Ok(size) = value.parse::<usize>() {
                    app.config.set_tab_size(size)?;
                    app.set_status_message(format!("Tab size set to {}", size));
                } else {
                    return Err(anyhow::anyhow!("Invalid tab size: {}", value));
                }
            }
            "so" | "scrolloffset" | "scroll_offset" => {
                if let Ok(offset) = value.parse::<usize>() {
                    app.config.set_scroll_offset(offset)?;
                    app.set_status_message(format!("Scroll offset set to {}", offset));
                } else {
                    return Err(anyhow::anyhow!("Invalid scroll offset: {}", value));
                }
            }
            "nu" | "number" | "line_numbers" => {
                match value.to_lowercase().as_str() {
                    "true" | "1" => {
                        app.config.set_line_numbers(true)?;
                        app.set_status_message("Line numbers enabled".to_string());
                    }
                    "false" | "0" => {
                        app.config.set_line_numbers(false)?;
                        app.set_status_message("Line numbers disabled".to_string());
                    }
                    _ => return Err(anyhow::anyhow!("Invalid value for line numbers: {} (use true/false)", value)),
                }
            }
            "rnu" | "relativenumber" | "relative_line_numbers" => {
                match value.to_lowercase().as_str() {
                    "true" | "1" => {
                        app.config.set_relative_line_numbers(true)?;
                        app.set_status_message("Relative line numbers enabled".to_string());
                    }
                    "false" | "0" => {
                        app.config.set_relative_line_numbers(false)?;
                        app.set_status_message("Relative line numbers disabled".to_string());
                    }
                    _ => return Err(anyhow::anyhow!("Invalid value for relative line numbers: {} (use true/false)", value)),
                }
            }
            "et" | "expandtab" | "insert_tabs" => {
                match value.to_lowercase().as_str() {
                    "true" | "1" => {
                        app.config.set_insert_tabs(true)?;
                        app.set_status_message("Expand tabs enabled (tabs as spaces)".to_string());
                    }
                    "false" | "0" => {
                        app.config.set_insert_tabs(false)?;
                        app.set_status_message("Expand tabs disabled (use actual tabs)".to_string());
                    }
                    _ => return Err(anyhow::anyhow!("Invalid value for expand tabs: {} (use true/false)", value)),
                }
            }
            "autosave" | "auto_save" => {
                match value.to_lowercase().as_str() {
                    "true" | "1" => {
                        app.config.set_auto_save(true)?;
                        app.set_status_message("Auto-save enabled".to_string());
                    }
                    "false" | "0" => {
                        app.config.set_auto_save(false)?;
                        app.set_status_message("Auto-save disabled".to_string());
                    }
                    _ => return Err(anyhow::anyhow!("Invalid value for auto-save: {} (use true/false)", value)),
                }
            }
            "wrap" | "wrap_lines" => {
                match value.to_lowercase().as_str() {
                    "true" | "1" => {
                        app.config.set_wrap_lines(true)?;
                        app.set_status_message("Line wrapping enabled".to_string());
                    }
                    "false" | "0" => {
                        app.config.set_wrap_lines(false)?;
                        app.set_status_message("Line wrapping disabled".to_string());
                    }
                    _ => return Err(anyhow::anyhow!("Invalid value for line wrapping: {} (use true/false)", value)),
                }
            }
            "syntax" | "syntax_highlighting" => {
                match value.to_lowercase().as_str() {
                    "true" | "1" => {
                        app.config.set_syntax_highlighting(true)?;
                        app.set_status_message("Syntax highlighting enabled".to_string());
                    }
                    "false" | "0" => {
                        app.config.set_syntax_highlighting(false)?;
                        app.set_status_message("Syntax highlighting disabled".to_string());
                    }
                    _ => return Err(anyhow::anyhow!("Invalid value for syntax highlighting: {} (use true/false)", value)),
                }
            }
            "cursorblink" | "cursor_blink" => {
                match value.to_lowercase().as_str() {
                    "true" | "1" => {
                        app.config.set_cursor_blink(true)?;
                        app.set_status_message("Cursor blinking enabled".to_string());
                    }
                    "false" | "0" => {
                        app.config.set_cursor_blink(false)?;
                        app.set_status_message("Cursor blinking disabled".to_string());
                    }
                    _ => return Err(anyhow::anyhow!("Invalid value for cursor blink: {} (use true/false)", value)),
                }
            }
            "statusline" | "show_status_line" => {
                match value.to_lowercase().as_str() {
                    "true" | "1" => {
                        app.config.set_show_status_line(true)?;
                        app.set_status_message("Status line enabled".to_string());
                    }
                    "false" | "0" => {
                        app.config.set_show_status_line(false)?;
                        app.set_status_message("Status line disabled".to_string());
                    }
                    _ => return Err(anyhow::anyhow!("Invalid value for status line: {} (use true/false)", value)),
                }
            }
            "commandline" | "show_command_line" => {
                match value.to_lowercase().as_str() {
                    "true" | "1" => {
                        app.config.set_show_command_line(true)?;
                        app.set_status_message("Command line enabled".to_string());
                    }
                    "false" | "0" => {
                        app.config.set_show_command_line(false)?;
                        app.set_status_message("Command line disabled".to_string());
                    }
                    _ => return Err(anyhow::anyhow!("Invalid value for command line: {} (use true/false)", value)),
                }
            }
            _ => {
                return Err(anyhow::anyhow!("Unknown setting: {}", key));
            }
        }
        Ok(())
    }

    fn handle_set_flag(&self, app: &mut App, flag: &str) -> Result<()> {
        match flag.to_lowercase().as_str() {
            "nu" | "number" => {
                app.config.set_line_numbers(true)?;
                app.set_status_message("Line numbers enabled".to_string());
            }
            "nonu" | "nonumber" => {
                app.config.set_line_numbers(false)?;
                app.set_status_message("Line numbers disabled".to_string());
            }
            "rnu" | "relativenumber" => {
                app.config.set_relative_line_numbers(true)?;
                app.set_status_message("Relative line numbers enabled".to_string());
            }
            "nornu" | "norelativenumber" => {
                app.config.set_relative_line_numbers(false)?;
                app.set_status_message("Relative line numbers disabled".to_string());
            }
            "et" | "expandtab" => {
                app.config.set_insert_tabs(true)?;
                app.set_status_message("Expand tabs enabled (tabs as spaces)".to_string());
            }
            "noet" | "noexpandtab" => {
                app.config.set_insert_tabs(false)?;
                app.set_status_message("Expand tabs disabled (use actual tabs)".to_string());
            }
            "autosave" => {
                app.config.set_auto_save(true)?;
                app.set_status_message("Auto-save enabled".to_string());
            }
            "noautosave" => {
                app.config.set_auto_save(false)?;
                app.set_status_message("Auto-save disabled".to_string());
            }
            "wrap" => {
                app.config.set_wrap_lines(true)?;
                app.set_status_message("Line wrapping enabled".to_string());
            }
            "nowrap" => {
                app.config.set_wrap_lines(false)?;
                app.set_status_message("Line wrapping disabled".to_string());
            }
            "syntax" => {
                app.config.set_syntax_highlighting(true)?;
                app.set_status_message("Syntax highlighting enabled".to_string());
            }
            "nosyntax" => {
                app.config.set_syntax_highlighting(false)?;
                app.set_status_message("Syntax highlighting disabled".to_string());
            }
            _ => {
                return Err(anyhow::anyhow!("Unknown setting: {}. Use ':set' to see available options", flag));
            }
        }
        Ok(())
    }
}

impl Default for EventHandler {
    fn default() -> Self {
        Self::new()
    }
}
