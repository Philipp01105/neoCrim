use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};
use crate::app::App;
use crate::ui::theme::Theme;

pub struct Renderer {
    theme: Theme,
}

impl Renderer {
    pub fn new(theme: Theme) -> Self {
        Self { theme }
    }

    pub fn update_theme(&mut self, theme: Theme) {
        self.theme = theme;
    }

    pub fn render(&self, frame: &mut Frame, app: &mut App) {
        let size = frame.size();

        let main_chunks = if app.file_explorer.visible {
            Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Length(30),
                    Constraint::Min(10),
                ].as_ref())
                .split(size)
        } else {
            Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(100),
                ].as_ref())
                .split(size)
        };

        if app.file_explorer.visible {
            app.file_explorer.render(frame, main_chunks[0], &self.theme);

            let command_line_height = if app.error_message.is_some() { 2 } else { 1 };
            let editor_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(3),
                    Constraint::Length(1),
                    Constraint::Length(command_line_height),
                ].as_ref())
                .split(main_chunks[1]);

            self.render_editor(frame, app, editor_chunks[0]);
            self.render_status_line(frame, app, editor_chunks[1]);
            self.render_command_line(frame, app, editor_chunks[2]);
        } else {
            let command_line_height = if app.error_message.is_some() { 2 } else { 1 };
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(3),
                    Constraint::Length(1),
                    Constraint::Length(command_line_height),
                ].as_ref())
                .split(main_chunks[0]);

            self.render_editor(frame, app, chunks[0]);
            self.render_status_line(frame, app, chunks[1]);
            self.render_command_line(frame, app, chunks[2]);
        }

        if app.help_window.visible {
            self.render_help_window(frame, app, size);
        }
    }

    fn render_editor(&self, frame: &mut Frame, app: &App, area: Rect) {
        let buffer = app.current_buffer();
        let cursor = &app.cursor;

        let line_number_width = if app.config.editor.line_numbers { 5 } else { 0 };
        let content_width = area.width as usize - line_number_width;
        let viewport_height = area.height as usize;
        let scroll_offset = app.config.editor.scroll_offset;

        let mut visual_lines = Vec::new();
        let mut current_visual_line = 0;
        let mut cursor_visual_line = 0;

        for line_idx in 0..buffer.line_count() {
            let line_content = buffer.line(line_idx).unwrap_or_default();
            let line_len = line_content.len();
            
            if line_len <= content_width {
                visual_lines.push((line_idx, 0, line_content.clone()));
                if line_idx == cursor.line {
                    cursor_visual_line = current_visual_line;
                }
                current_visual_line += 1;
            } else {
                let wrapped_lines = (line_len + content_width - 1) / content_width;
                for wrap_idx in 0..wrapped_lines {
                    let start = wrap_idx * content_width;
                    let end = (start + content_width).min(line_len);
                    let segment = line_content[start..end].to_string();
                    
                    visual_lines.push((line_idx, wrap_idx, segment));
                    
                    if line_idx == cursor.line && cursor.col >= start && cursor.col < end {
                        cursor_visual_line = current_visual_line;
                    }
                    current_visual_line += 1;
                }
            }
        }

        let start_visual_line = if cursor_visual_line >= scroll_offset {
            (cursor_visual_line - scroll_offset).min(visual_lines.len().saturating_sub(viewport_height))
        } else {
            0
        };

        let end_visual_line = (start_visual_line + viewport_height).min(visual_lines.len());

        let mut lines = Vec::new();
        for visual_idx in start_visual_line..end_visual_line {
            if let Some((line_idx, wrap_idx, content)) = visual_lines.get(visual_idx) {
                let line_number = if app.config.editor.line_numbers {
                    if *wrap_idx == 0 {
                        format!("{:4} ", line_idx + 1)
                    } else {
                        "     ".to_string()
                    }
                } else {
                    String::new()
                };

                let buffer = app.current_buffer();
                let mut spans = vec![
                    Span::styled(line_number, Style::default().fg(self.theme.line_number)),
                ];

                let content_with_highlights = self.apply_search_highlighting(content, *line_idx, app);
                
                if app.config.editor.syntax_highlighting {
                    if let Some(syntax) = buffer.file_path()
                        .and_then(|path| app.syntax_highlighter.detect_language(Some(path))) {
                        let highlighted_spans = app.syntax_highlighter.highlight_line(content, syntax, &app.config.current_theme.colors);

                        for (mut highlight_style, text) in highlighted_spans {
                            if *line_idx == cursor.line {
                                highlight_style = highlight_style.bg(self.theme.current_line);
                            }
                            
                            spans.push(Span::styled(text, highlight_style));
                        }
                    } else {
                        spans.extend(content_with_highlights);
                    }
                } else {
                    spans.extend(content_with_highlights);
                }

                lines.push(Line::from(spans));
            }
        }

        let paragraph = Paragraph::new(lines)
            .style(Style::default().fg(self.theme.foreground).bg(self.theme.background))
            .block(Block::default().borders(Borders::NONE));

        frame.render_widget(paragraph, area);

        self.render_cursor_visual(frame, app, area, start_visual_line, &visual_lines);
    }

    fn render_cursor_visual(&self, frame: &mut Frame, app: &App, area: Rect, start_visual_line: usize, visual_lines: &[(usize, usize, String)]) {
        let cursor = &app.cursor;
        let line_number_width = if app.config.editor.line_numbers { 5 } else { 0 };
        let content_width = area.width as usize - line_number_width;

        for (visual_idx, (line_idx, wrap_idx, _content)) in visual_lines.iter().enumerate() {
            if *line_idx == cursor.line {
                let start_col = wrap_idx * content_width;
                let end_col = start_col + content_width;
                
                if cursor.col >= start_col && cursor.col < end_col {
                    let visual_line_idx = visual_idx;
                    
                    if visual_line_idx >= start_visual_line && visual_line_idx < start_visual_line + area.height as usize {
                        let cursor_y = area.y + (visual_line_idx - start_visual_line) as u16;
                        let cursor_x = area.x + line_number_width as u16 + (cursor.col - start_col) as u16;

                        if cursor_x < area.x + area.width && cursor_y < area.y + area.height {
                            let cursor_area = Rect {
                                x: cursor_x,
                                y: cursor_y,
                                width: 1,
                                height: 1,
                            };

                            let cursor_char = if app.mode.is_insert() { "|" } else { "█" };
                            let cursor_widget = Paragraph::new(cursor_char)
                                .style(Style::default().fg(self.theme.cursor).add_modifier(Modifier::BOLD));

                            frame.render_widget(Clear, cursor_area);
                            frame.render_widget(cursor_widget, cursor_area);
                        }
                    }
                    break;
                }
            }
        }
    }

    fn render_cursor(&self, frame: &mut Frame, app: &App, area: Rect, start_line: usize) {
        let cursor = &app.cursor;
        let buffer = app.current_buffer();

        if cursor.line >= start_line && cursor.line < start_line + area.height as usize {
            let line_number_width = if app.config.editor.line_numbers { 5 } else { 0 };
            let content_width = area.width as usize - line_number_width;
            
           let (visual_line_offset, _) = cursor.calculate_visual_lines(&buffer, content_width);
            let visual_col = cursor.col % content_width;
            
            let cursor_y = area.y + (cursor.line - start_line) as u16 + visual_line_offset as u16;
            let cursor_x = area.x + line_number_width as u16 + visual_col as u16;

            if cursor_x < area.x + area.width && cursor_y < area.y + area.height {
                let cursor_area = Rect {
                    x: cursor_x,
                    y: cursor_y,
                    width: 1,
                    height: 1,
                };

                let cursor_char = if app.mode.is_insert() { "|" } else { "█" };
                let cursor_widget = Paragraph::new(cursor_char)
                    .style(Style::default().fg(self.theme.cursor).add_modifier(Modifier::BOLD));

                frame.render_widget(Clear, cursor_area);
                frame.render_widget(cursor_widget, cursor_area);
            }
        }
    }

    fn render_status_line(&self, frame: &mut Frame, app: &App, area: Rect) {
        let buffer = app.current_buffer();
        let cursor = &app.cursor;

        let mode_text = format!(" {} ", app.mode.name());
        let file_info = if let Some(filename) = buffer.file_name() {
            format!(" {} {}", filename, if buffer.is_modified { "[+]" } else { "" })
        } else {
            " [No Name] ".to_string()
        };

        let cursor_info = format!(" {}:{} ", cursor.line + 1, cursor.col + 1);

        let spans = vec![
            Span::styled(mode_text, Style::default().fg(Color::Black).bg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(file_info, Style::default().fg(self.theme.foreground)),
        ];

        let status_paragraph = Paragraph::new(Line::from(spans))
            .style(Style::default().bg(self.theme.background));

        frame.render_widget(status_paragraph, area);

        let cursor_area = Rect {
            x: area.x + area.width.saturating_sub(cursor_info.len() as u16),
            y: area.y,
            width: cursor_info.len() as u16,
            height: 1,
        };

        let cursor_paragraph = Paragraph::new(cursor_info)
            .style(Style::default().fg(self.theme.foreground).bg(self.theme.background));

        frame.render_widget(cursor_paragraph, cursor_area);
    }

    fn render_command_line(&self, frame: &mut Frame, app: &App, area: Rect) {
        let chunks = if app.error_message.is_some() {
            Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(1),
                    Constraint::Length(1),
                ])
                .split(area)
        } else {
            Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(1),
                ])
                .split(area)
        };

        let content = if app.mode.is_command() {
            format!(":{}", app.command_line)
        } else if let Some(ref message) = app.status_message {
            message.clone()
        } else {
            String::new()
        };

        let paragraph = Paragraph::new(content)
            .style(Style::default().fg(self.theme.foreground).bg(self.theme.background));

        frame.render_widget(paragraph, chunks[0]);

        if let Some(ref error_msg) = app.error_message {
            if chunks.len() > 1 {
                let error_paragraph = Paragraph::new(error_msg.clone())
                    .style(Style::default().fg(Color::Red).bg(self.theme.background));
                frame.render_widget(error_paragraph, chunks[1]);
            }
        }
    }

    fn apply_search_highlighting<'a>(&self, content: &'a str, line_idx: usize, app: &App) -> Vec<Span<'a>> {
        if !app.search_state.is_active || app.search_state.query.is_empty() {
            let style = if line_idx == app.cursor.line {
                Style::default().bg(self.theme.current_line)
            } else {
                Style::default()
            };
            return vec![Span::styled(content, style)];
        }

        let mut spans = Vec::new();
        let mut last_end = 0;
        let query = &app.search_state.query;
        
        let mut matches = Vec::new();
        let mut start = 0;
        while let Some(pos) = content[start..].find(query) {
            let actual_pos = start + pos;
            matches.push((actual_pos, actual_pos + query.len()));
            start = actual_pos + 1;
        }

        for (start, end) in matches {
           if start > last_end {
                let text = &content[last_end..start];
                let style = if line_idx == app.cursor.line {
                    Style::default().bg(self.theme.current_line)
                } else {
                    Style::default()
                };
                spans.push(Span::styled(text, style));
            }
            
            let match_text = &content[start..end];
            let mut style = Style::default().bg(Color::Yellow).fg(Color::Black);
            if line_idx == app.cursor.line {
                style = style.bg(Color::LightYellow);
            }
            spans.push(Span::styled(match_text, style));
            
            last_end = end;
        }

        if last_end < content.len() {
            let text = &content[last_end..];
            let style = if line_idx == app.cursor.line {
                Style::default().bg(self.theme.current_line)
            } else {
                Style::default()
            };
            spans.push(Span::styled(text, style));
        }

        spans
    }

    fn apply_search_highlighting_to_span<'a>(&self, text: &'a str, _line_idx: usize, app: &App, base_style: Style) -> Vec<Span<'a>> {
        if !app.search_state.is_active || app.search_state.query.is_empty() {
            return vec![Span::styled(text, base_style)];
        }

        let mut spans = Vec::new();
        let mut last_end = 0;
        let query = &app.search_state.query;
        
        let mut matches = Vec::new();
        let mut start = 0;
        while let Some(pos) = text[start..].find(query) {
            let actual_pos = start + pos;
            matches.push((actual_pos, actual_pos + query.len()));
            start = actual_pos + 1;
        }

        for (start, end) in matches {
            if start > last_end {
                let segment = &text[last_end..start];
                spans.push(Span::styled(segment, base_style));
            }
            
            let match_text = &text[start..end];
            let highlight_style = base_style.bg(Color::Yellow).fg(Color::Black);
            spans.push(Span::styled(match_text, highlight_style));
            
            last_end = end;
        }

        if last_end < text.len() {
            let segment = &text[last_end..];
            spans.push(Span::styled(segment, base_style));
        }

        if spans.is_empty() {
            spans.push(Span::styled(text, base_style));
        }

        spans
    }

    fn render_help_window(&self, frame: &mut Frame, app: &App, area: Rect) {
        let window_width = (area.width * 4 / 5).max(60).min(80);
        let window_height = (area.height * 4 / 5).max(20);
        
        let x = (area.width.saturating_sub(window_width)) / 2;
        let y = (area.height.saturating_sub(window_height)) / 2;
        
        let help_area = Rect {
            x: area.x + x,
            y: area.y + y,
            width: window_width,
            height: window_height,
        };

        let clear_widget = ratatui::widgets::Clear;
        frame.render_widget(clear_widget, help_area);

        let visible_height = help_area.height.saturating_sub(2) as usize; // Account for borders
        let start_line = app.help_window.scroll_offset;
        let end_line = (start_line + visible_height).min(app.help_window.content.len());
        
        let visible_content: Vec<Line> = app.help_window.content[start_line..end_line]
            .iter()
            .map(|line| Line::from(line.as_str()))
            .collect();

        // Determine if scrolling is needed
        let can_scroll_up = app.help_window.scroll_offset > 0;
        let can_scroll_down = end_line < app.help_window.content.len();
        
        // Create title with scroll indicators
        let title = if can_scroll_up || can_scroll_down {
            format!(" Help ({}/{}) ", start_line + 1, app.help_window.content.len())
        } else {
            " Help ".to_string()
        };

        // Create the help window
        let help_paragraph = Paragraph::new(visible_content)
            .style(Style::default().fg(self.theme.foreground).bg(self.theme.background))
            .block(
                Block::default()
                    .borders(ratatui::widgets::Borders::ALL)
                    .title(title)
                    .title_style(Style::default().fg(self.theme.foreground).add_modifier(ratatui::style::Modifier::BOLD))
                    .border_style(Style::default().fg(self.theme.foreground))
            )
            .wrap(ratatui::widgets::Wrap { trim: false });

        frame.render_widget(help_paragraph, help_area);

        // Add scroll instructions at the bottom
        if can_scroll_up || can_scroll_down {
            let scroll_info = if can_scroll_up && can_scroll_down {
                " ↑↓ to scroll, ESC to close "
            } else if can_scroll_up {
                " ↑ to scroll up, ESC to close "
            } else {
                " ↓ to scroll down, ESC to close "
            };
            
            let info_area = Rect {
                x: help_area.x + 2,
                y: help_area.y + help_area.height - 1,
                width: help_area.width.saturating_sub(4),
                height: 1,
            };
            
            let info_paragraph = Paragraph::new(scroll_info)
                .style(Style::default().fg(Color::Yellow).bg(self.theme.background));
            
            frame.render_widget(info_paragraph, info_area);
        }
    }

}
