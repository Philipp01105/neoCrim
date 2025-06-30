use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};
use crate::app::App;
use crate::ui::theme::Theme;
use crate::ui::themes::NeoTheme;

pub struct Renderer {
    theme: Theme,
    glass_effects_enabled: bool,
}

impl Renderer {
    pub fn new(theme: Theme) -> Self {
        Self { 
            theme,
            glass_effects_enabled: false,
        }
    }

    pub fn new_with_glass_effects(theme: Theme, neo_theme: &NeoTheme) -> Self {
        Self {
            theme,
            glass_effects_enabled: neo_theme.colors.enable_glass,
        }
    }

    pub fn update_theme(&mut self, theme: Theme) {
        self.theme = theme;
    }

    pub fn update_theme_with_effects(&mut self, theme: Theme, neo_theme: &NeoTheme) {
        self.theme = theme;
        self.glass_effects_enabled = neo_theme.colors.enable_glass;
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

            let command_line_height = if app.config.ui.show_command_line {
                if app.error_message.is_some() { 2 } else { 1 }
            } else { 0 };
            
            let status_line_height = if app.config.ui.show_status_line { 1 } else { 0 };
            
            let mut constraints = vec![Constraint::Min(3)];
            if app.config.ui.show_status_line {
                constraints.push(Constraint::Length(status_line_height));
            }
            if app.config.ui.show_command_line {
                constraints.push(Constraint::Length(command_line_height));
            }
            
            let editor_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(constraints)
                .split(main_chunks[1]);

            self.render_editor(frame, app, editor_chunks[0]);
            
            let mut chunk_idx = 1;
            if app.config.ui.show_status_line {
                self.render_status_line(frame, app, editor_chunks[chunk_idx]);
                chunk_idx += 1;
            }
            if app.config.ui.show_command_line {
                self.render_command_line(frame, app, editor_chunks[chunk_idx]);
            }
        } else {
            let command_line_height = if app.config.ui.show_command_line {
                if app.error_message.is_some() { 2 } else { 1 }
            } else { 0 };
            
            let status_line_height = if app.config.ui.show_status_line { 1 } else { 0 };
            
            let mut constraints = vec![Constraint::Min(3)];
            if app.config.ui.show_status_line {
                constraints.push(Constraint::Length(status_line_height));
            }
            if app.config.ui.show_command_line {
                constraints.push(Constraint::Length(command_line_height));
            }
            
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(constraints)
                .split(main_chunks[0]);

            self.render_editor(frame, app, chunks[0]);
            
            let mut chunk_idx = 1;
            if app.config.ui.show_status_line {
                self.render_status_line(frame, app, chunks[chunk_idx]);
                chunk_idx += 1;
            }
            if app.config.ui.show_command_line {
                self.render_command_line(frame, app, chunks[chunk_idx]);
            }
        }

        if app.help_window.visible {
            self.render_help_window(frame, app, size);
        }
    }

    fn render_editor(&self, frame: &mut Frame, app: &App, area: Rect) {
        let buffer = app.current_buffer();
        let cursor = &app.cursor;

        if buffer.is_terminal() {
            self.render_terminal(frame, app, area);
            return;
        }

        let line_number_width = if app.config.editor.line_numbers || app.config.editor.relative_line_numbers { 5 } else { 0 };
        let content_width = area.width as usize - line_number_width;
        let viewport_height = area.height as usize;
        let scroll_offset = app.config.editor.scroll_offset;

        let mut visual_lines = Vec::new();
        let mut current_visual_line = 0;
        let mut cursor_visual_line = 0;

        for line_idx in 0..buffer.line_count() {
            let line_content = buffer.line(line_idx).unwrap_or_default();
            let line_len = line_content.len();
            
            if !app.config.editor.wrap_lines || line_len <= content_width {
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
                let line_number = if app.config.editor.relative_line_numbers {
                    if *wrap_idx == 0 {
                        if *line_idx == cursor.line {
                            if app.config.editor.line_numbers {
                                format!("{:4} ", line_idx + 1)
                            } else {
                                format!("{:4} ", 0)
                            }
                        } else {
                            let relative_distance = if *line_idx > cursor.line {
                                *line_idx - cursor.line
                            } else {
                                cursor.line - *line_idx
                            };
                            format!("{:4} ", relative_distance)
                        }
                    } else {
                        "     ".to_string()
                    }
                } else if app.config.editor.line_numbers {
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

                let content_with_highlights = self.apply_highlighting(content, *line_idx, app);
                
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

        let background_style = self.get_background_style(app);
        let paragraph = Paragraph::new(lines)
            .style(Style::default().fg(self.theme.foreground).bg(background_style))
            .block(Block::default().borders(Borders::NONE));

        frame.render_widget(paragraph, area);

        self.render_cursor_visual(frame, app, area, start_visual_line, &visual_lines);
    }

    fn render_cursor_visual(&self, frame: &mut Frame, app: &App, area: Rect, start_visual_line: usize, visual_lines: &[(usize, usize, String)]) {
        let cursor = &app.cursor;
        let line_number_width = if app.config.editor.line_numbers || app.config.editor.relative_line_numbers { 5 } else { 0 };
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

                        if cursor_x < area.x + area.width && cursor_y < area.y + area.height && app.should_show_cursor() {
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

        let background_style = self.get_glass_style(app, self.theme.background);
        let status_paragraph = Paragraph::new(Line::from(spans))
            .style(Style::default().bg(background_style));

        frame.render_widget(status_paragraph, area);

        let cursor_area = Rect {
            x: area.x + area.width.saturating_sub(cursor_info.len() as u16),
            y: area.y,
            width: cursor_info.len() as u16,
            height: 1,
        };

        let cursor_paragraph = Paragraph::new(cursor_info)
            .style(Style::default().fg(self.theme.foreground).bg(background_style));

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

        let background_style = self.get_glass_style(app, self.theme.background);
        let paragraph = Paragraph::new(content)
            .style(Style::default().fg(self.theme.foreground).bg(background_style));

        frame.render_widget(paragraph, chunks[0]);

        if let Some(ref error_msg) = app.error_message {
            if chunks.len() > 1 {
                let error_paragraph = Paragraph::new(error_msg.clone())
                    .style(Style::default().fg(Color::Red).bg(background_style));
                frame.render_widget(error_paragraph, chunks[1]);
            }
        }
    }

    fn apply_highlighting<'a>(&self, content: &'a str, line_idx: usize, app: &App) -> Vec<Span<'a>> {
        let mut segments = Vec::new();
        
        if app.selection.active {
            if let Some((start, end)) = app.selection.get_range() {
                if line_idx >= start.line && line_idx <= end.line {
                    let (sel_start, sel_end) = if line_idx == start.line && line_idx == end.line {
                        (start.col.min(content.len()), end.col.min(content.len()))
                    } else if line_idx == start.line {
                        (start.col.min(content.len()), content.len())
                    } else if line_idx == end.line {
                        (0, end.col.min(content.len()))
                    } else {
                        (0, content.len())
                    };
                    
                    let sel_start = sel_start.min(content.len());
                    let sel_end = sel_end.min(content.len()).max(sel_start);
                    
                    if sel_start > 0 {
                        segments.push((0, sel_start, false, false));
                    }
                    if sel_end > sel_start {
                        segments.push((sel_start, sel_end, true, false));
                    }
                    if sel_end < content.len() {
                        segments.push((sel_end, content.len(), false, false));
                    }
                } else {
                    segments.push((0, content.len(), false, false));
                }
            } else {
                segments.push((0, content.len(), false, false));
            }
        } else {
            segments.push((0, content.len(), false, false));
        }
        
        let mut spans = Vec::new();
        let query = if app.search_state.is_active && !app.search_state.query.is_empty() {
            Some(&app.search_state.query)
        } else {
            None
        };
        
        for (start, end, is_selected, _) in segments {
            let text = &content[start..end];
            if text.is_empty() {
                continue;
            }
            
            if let Some(search_query) = query {
                let mut last_pos = 0;
                while let Some(match_pos) = text[last_pos..].find(search_query) {
                    let actual_pos = last_pos + match_pos;
                    
                    if actual_pos > last_pos {
                        let before_text = &text[last_pos..actual_pos];
                        let style = self.get_text_style(line_idx, is_selected, false, app);
                        spans.push(Span::styled(before_text, style));
                    }
                    
                    let match_end = actual_pos + search_query.len();
                    let match_text = &text[actual_pos..match_end];
                    let style = self.get_text_style(line_idx, is_selected, true, app);
                    spans.push(Span::styled(match_text, style));
                    
                    last_pos = match_end;
                }
                
                if last_pos < text.len() {
                    let remaining_text = &text[last_pos..];
                    let style = self.get_text_style(line_idx, is_selected, false, app);
                    spans.push(Span::styled(remaining_text, style));
                }
            } else {
                let style = self.get_text_style(line_idx, is_selected, false, app);
                spans.push(Span::styled(text, style));
            }
        }
        
        spans
    }
    
    fn get_text_style(&self, line_idx: usize, is_selected: bool, is_search_match: bool, app: &App) -> Style {
        let mut style = Style::default();
        
        if line_idx == app.cursor.line {
            style = style.bg(self.theme.current_line);
        }
        
        if is_selected {
            style = style.bg(self.theme.selection).fg(Color::White);
        }
        
        if is_search_match {
            if is_selected {
                style = style.bg(Color::LightYellow).fg(Color::Black);
            } else {
                style = style.bg(Color::Yellow).fg(Color::Black);
            }
        }
        
        style
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

        let visible_height = help_area.height.saturating_sub(2) as usize;
        let start_line = app.help_window.scroll_offset;
        let end_line = (start_line + visible_height).min(app.help_window.content.len());
        
        let visible_content: Vec<Line> = app.help_window.content[start_line..end_line]
            .iter()
            .map(|line| Line::from(line.as_str()))
            .collect();

        let can_scroll_up = app.help_window.scroll_offset > 0;
        let can_scroll_down = end_line < app.help_window.content.len();
        
        let title = if can_scroll_up || can_scroll_down {
            format!(" Help ({}/{}) ", start_line + 1, app.help_window.content.len())
        } else {
            " Help ".to_string()
        };

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

    fn get_background_style(&self, app: &App) -> Color {
        if self.glass_effects_enabled {
            let neo_theme = &app.config.current_theme;
            if neo_theme.colors.enable_glass && neo_theme.colors.background_opacity < 1.0 {
                return neo_theme.colors.background.to_transparent_color();
            }
        }
        self.theme.background
    }

    fn get_glass_style(&self, app: &App, base_color: Color) -> Color {
        if self.glass_effects_enabled {
            let neo_theme = &app.config.current_theme;
            if neo_theme.colors.enable_glass && neo_theme.colors.background_opacity < 1.0 {
                let status_bg_alpha = neo_theme.colors.status_bg.get_alpha();
                if status_bg_alpha < 1.0 {
                    return neo_theme.colors.status_bg.to_transparent_color();
                }
                
                if let Color::Rgb(r, g, b) = base_color {
                    let alpha = neo_theme.colors.background_opacity;
                    if alpha < 0.5 {
                        return Color::Reset; 
                    } else {
                        let blended_r = ((r as f32 * alpha) + (30.0 * (1.0 - alpha))) as u8;
                        let blended_g = ((g as f32 * alpha) + (30.0 * (1.0 - alpha))) as u8;
                        let blended_b = ((b as f32 * alpha) + (30.0 * (1.0 - alpha))) as u8;
                        return Color::Rgb(blended_r, blended_g, blended_b);
                    }
                }
            }
        }
        base_color
    }

    fn render_terminal(&self, frame: &mut Frame, app: &App, area: Rect) {
        let buffer = app.current_buffer();
        
        if let Some(ref terminal_output) = buffer.terminal_output {
            let block = ratatui::widgets::Block::default()
                .title("[Terminal]")
                .borders(ratatui::widgets::Borders::ALL)
                .border_style(self.theme.terminal_border)
                .title_style(self.theme.terminal_title);

            let inner_area = block.inner(area);
            frame.render_widget(block, area);

            let viewport_height = inner_area.height as usize;
            let total_lines = terminal_output.lines.len();
            let _start_line = if total_lines <= viewport_height {
                0
            } else {
                total_lines - viewport_height
            };

            let content_height = viewport_height.saturating_sub(1);
            let display_lines = if terminal_output.lines.len() > content_height {
                terminal_output.lines.len() - content_height
            } else {
                0
            };

            let mut visible_lines: Vec<Line> = terminal_output.lines
                .iter()
                .skip(display_lines)
                .take(content_height)
                .map(|line| {
                    let style = if line.starts_with("$ ") {
                        self.theme.terminal_command
                    } else if line.starts_with("ERROR:") {
                        self.theme.terminal_error
                    } else {
                        self.theme.terminal_output
                    };
                    Line::from(Span::styled(line.clone(), style))
                })
                .collect();

            let prompt_line = terminal_output.get_prompt_line();
            visible_lines.push(Line::from(Span::styled(prompt_line, self.theme.terminal_command)));

            let paragraph = Paragraph::new(visible_lines)
                .style(self.theme.terminal_background);

            frame.render_widget(paragraph, inner_area);

            let prompt_line = terminal_output.get_prompt_line();
            let cursor_x = inner_area.x + prompt_line.len() as u16;
            let cursor_y = inner_area.y + inner_area.height - 1;
            
            if cursor_x < inner_area.x + inner_area.width && cursor_y < inner_area.y + inner_area.height && app.should_show_cursor() {
                let cursor_area = Rect {
                    x: cursor_x,
                    y: cursor_y,
                    width: 1,
                    height: 1,
                };

                let cursor_widget = Paragraph::new("█")
                    .style(Style::default().fg(self.theme.cursor).add_modifier(ratatui::style::Modifier::BOLD));

                frame.render_widget(ratatui::widgets::Clear, cursor_area);
                frame.render_widget(cursor_widget, cursor_area);
            }

            if terminal_output.is_running {
                let status_area = Rect {
                    x: area.x + 2,
                    y: area.y,
                    width: 12,
                    height: 1,
                };
                let status = Paragraph::new("[Running...]")
                    .style(self.theme.terminal_running);
                frame.render_widget(status, status_area);
            }
        }
    }

}
