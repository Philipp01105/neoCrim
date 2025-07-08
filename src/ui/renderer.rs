use crate::app::App;
use crate::ui::theme::Theme;
use crate::ui::themes::NeoTheme;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

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
    pub fn update_theme_with_effects(&mut self, theme: Theme, neo_theme: &NeoTheme) {
        self.theme = theme;
        self.glass_effects_enabled = neo_theme.colors.enable_glass;
    }

    pub fn render(&self, frame: &mut Frame, app: &mut App) {
        let size = frame.size();

        if app.file_explorer.visible {
            let main_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Length(30), Constraint::Min(10)].as_ref())
                .split(size);

            app.file_explorer.render(frame, main_chunks[0], &self.theme);

            let command_line_height = if app.config.ui.show_command_line {
                if app.error_message.is_some() {
                    2
                } else {
                    1
                }
            } else {
                0
            };

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
                if app.error_message.is_some() {
                    2
                } else {
                    1
                }
            } else {
                0
            };

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
                .split(size);

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

        if app.file_change_dialog.visible {
            self.render_file_change_dialog(frame, app, size);
        }
    }

    fn render_editor(&self, frame: &mut Frame, app: &App, area: Rect) {
        let buffer = app.current_buffer();
        let cursor = &app.cursor;

        if buffer.is_terminal() {
            self.render_terminal(frame, app, area);
            return;
        }

        let line_number_width =
            if app.config.editor.line_numbers || app.config.editor.relative_line_numbers {
                5
            } else {
                0
            };
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
                let displayed_content = if !app.config.editor.wrap_lines {
                    let h_offset = app.get_horizontal_scroll_offset();
                    if h_offset < line_len {
                        let end = (h_offset + content_width).min(line_len);
                        let result = line_content[h_offset..end].to_string();
                        log::debug!(
                            "  Line {} no-wrap: h_offset={}, showing chars {}..{}, content=\"{}\"",
                            line_idx,
                            h_offset,
                            h_offset,
                            end,
                            result
                        );
                        result
                    } else {
                        log::info!(
                            "  Line {} no-wrap: h_offset={} >= line_len={}, showing empty",
                            line_idx,
                            h_offset,
                            line_len
                        );
                        String::new()
                    }
                } else {
                    log::info!("  Line {} fits in viewport: showing full content", line_idx);
                    line_content.clone()
                };

                visual_lines.push((line_idx, 0, displayed_content));
                if line_idx == cursor.line {
                    cursor_visual_line = current_visual_line;
                    log::info!(
                        "  Line {line_idx} is cursor line, cursor_visual_line={cursor_visual_line}",
                    );
                }
                current_visual_line += 1;
            } else {
                let wrapped_lines = line_len.div_ceil(content_width);
                log::info!(
                    "  Line {line_idx} wrapping: {wrapped_lines} visual lines needed",
                );

                for wrap_idx in 0..wrapped_lines {
                    let start = wrap_idx * content_width;
                    let end = (start + content_width).min(line_len);
                    let segment = line_content[start..end].to_string();

                    log::info!(
                        "    Wrap {wrap_idx} of line {line_idx}: chars {start}..{end}, content=\"{segment}\"",
                    );

                    visual_lines.push((line_idx, wrap_idx, segment));

                    if line_idx == cursor.line && cursor.col >= start && cursor.col < end {
                        cursor_visual_line = current_visual_line;
                        log::info!(
                            "    Cursor found in wrap {wrap_idx} of line {line_idx}, cursor_visual_line={cursor_visual_line}",
                        );
                    }
                    current_visual_line += 1;
                }
            }
        }

        log::info!(
            "Visual lines created: total={}, cursor at visual line={}",
            visual_lines.len(),
            cursor_visual_line
        );

        let start_visual_line = if cursor_visual_line >= scroll_offset {
            (cursor_visual_line - scroll_offset)
                .min(visual_lines.len().saturating_sub(viewport_height))
        } else {
            0
        };

        let end_visual_line = (start_visual_line + viewport_height).min(visual_lines.len());

        log::info!(
            "Viewport: showing visual lines {start_visual_line} to {end_visual_line} (total viewport_height={viewport_height})",
        );

        let mut lines = Vec::new();
        for visual_idx in start_visual_line..end_visual_line {
            if let Some((line_idx, wrap_idx, content)) = visual_lines.get(visual_idx) {
                let line_number = if app.config.editor.relative_line_numbers {
                    if *wrap_idx == 0 {
                        if *line_idx == cursor.line {
                            let line_num = if app.config.editor.line_numbers {
                                format!("{:4} ", line_idx + 1)
                            } else {
                                format!("{:4} ", 0)
                            };
                            log::info!("  Line number (cursor line): \"{}\"", line_num.trim());
                            line_num
                        } else {
                            let relative_distance = (*line_idx).abs_diff(cursor.line);
                            let line_num = format!("{relative_distance:4} ");
                            log::info!(
                                "  Line number (relative): distance={}, display=\"{}\"",
                                relative_distance,
                                line_num.trim()
                            );
                            line_num
                        }
                    } else {
                        log::info!("  Line number (wrap continuation): empty");
                        "     ".to_string()
                    }
                } else if app.config.editor.line_numbers {
                    if *wrap_idx == 0 {
                        let line_num = format!("{:4} ", line_idx + 1);
                        log::info!("  Line number (absolute): {}", line_num.trim());
                        line_num
                    } else {
                        log::info!("  Line number (wrap continuation): empty");
                        "     ".to_string()
                    }
                } else {
                    log::info!("  Line numbers disabled");
                    String::new()
                };

                let buffer = app.current_buffer();
                let mut spans = vec![Span::styled(
                    line_number,
                    Style::default().fg(self.theme.line_number),
                )];

                let content_with_highlights = self.apply_highlighting(content, *line_idx, app);

                if app.config.editor.syntax_highlighting {
                    if let Some(syntax) = buffer
                        .file_path()
                        .and_then(|path| app.syntax_highlighter.detect_language(Some(path)))
                    {
                        let highlighted_spans = app.syntax_highlighter.highlight_line(
                            content,
                            syntax,
                            &app.config.current_theme.colors,
                        );

                        let mut processed_spans = Vec::new();
                        for (mut highlight_style, text) in highlighted_spans {
                            if *line_idx == cursor.line {
                                highlight_style = highlight_style.bg(self.theme.current_line);
                            }
                            processed_spans.push(Span::styled(text, highlight_style));
                        }

                        let final_spans = self.apply_cursor_overlay(
                            processed_spans,
                            app,
                            *line_idx,
                            wrap_idx * content_width,
                        );
                        spans.extend(final_spans);
                    } else {
                        let final_spans = self.apply_cursor_overlay(
                            content_with_highlights,
                            app,
                            *line_idx,
                            wrap_idx * content_width,
                        );
                        spans.extend(final_spans);
                    }
                } else {
                    let simple_spans = vec![Span::raw(content)];
                    let final_spans = self.apply_cursor_overlay(
                        simple_spans,
                        app,
                        *line_idx,
                        wrap_idx * content_width,
                    );
                    spans.extend(final_spans);
                }

                log::info!("  Final spans count: {}", spans.len());
                lines.push(Line::from(spans));
            }
        }

        log::info!("Total rendered lines: {}", lines.len());

        let background_style = self.get_background_style(app);
        let paragraph = Paragraph::new(lines)
            .style(
                Style::default()
                    .fg(self.theme.foreground)
                    .bg(background_style),
            )
            .block(Block::default().borders(Borders::NONE));

        frame.render_widget(paragraph, area);
    }

    fn apply_cursor_overlay<'a>(
        &self,
        spans: Vec<Span<'a>>,
        app: &App,
        line_idx: usize,
        line_start_col: usize,
    ) -> Vec<Span<'a>> {
        if line_idx != app.cursor.line || !app.should_show_cursor() {
            return spans;
        }

        let h_offset = app.get_horizontal_scroll_offset();
        let cursor_col = app.cursor.col;

        let cursor_pos_in_segment = if cursor_col >= h_offset + line_start_col {
            cursor_col - h_offset - line_start_col
        } else {
            return spans;
        };

        self.apply_normal_cursor(spans, cursor_pos_in_segment)
    }

    fn apply_normal_cursor<'a>(&self, spans: Vec<Span<'a>>, cursor_pos: usize) -> Vec<Span<'a>> {
        let mut result = Vec::new();
        let mut current_pos = 0;
        let mut cursor_applied = false;

        for span in spans {
            let span_text = span.content.as_ref();
            let span_len = span_text.chars().count();

            if current_pos <= cursor_pos && cursor_pos < current_pos + span_len {
                let chars: Vec<char> = span_text.chars().collect();
                let split_pos = cursor_pos - current_pos;

                if split_pos > 0 {
                    let before_text: String = chars[0..split_pos].iter().collect();
                    result.push(Span::styled(before_text, span.style));
                }

                if split_pos < chars.len() {
                    let cursor_char = chars[split_pos].to_string();
                    result.push(Span::styled(
                        cursor_char,
                        Style::default()
                            .fg(self.theme.background)
                            .bg(self.theme.cursor)
                            .add_modifier(Modifier::BOLD),
                    ));
                } else {
                    result.push(Span::styled(
                        " ",
                        Style::default()
                            .fg(self.theme.background)
                            .bg(self.theme.cursor)
                            .add_modifier(Modifier::BOLD),
                    ));
                }

                if split_pos + 1 < chars.len() {
                    let after_text: String = chars[split_pos + 1..].iter().collect();
                    result.push(Span::styled(after_text, span.style));
                }

                cursor_applied = true;
            } else {
                result.push(span);
            }

            current_pos += span_len;
        }

        if !cursor_applied && current_pos <= cursor_pos {
            result.push(Span::styled(
                " ",
                Style::default()
                    .fg(self.theme.background)
                    .bg(self.theme.cursor)
                    .add_modifier(Modifier::BOLD),
            ));
        }

        result
    }

    fn render_status_line(&self, frame: &mut Frame, app: &App, area: Rect) {
        let buffer = app.current_buffer();
        let cursor = &app.cursor;

        let mode_text = format!(" {} ", app.mode.name());
        let file_info = if let Some(filename) = buffer.file_name() {
            format!(
                " {} {}",
                filename,
                if buffer.is_modified { "[+]" } else { "" }
            )
        } else {
            " [No Name] ".to_string()
        };

        let cursor_info = format!(" {}:{} ", cursor.line + 1, cursor.col + 1);

        let spans = vec![
            Span::styled(
                mode_text,
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(file_info, Style::default().fg(self.theme.foreground)),
        ];

        let background_style = self.get_glass_style(app, self.theme.background);
        let status_paragraph =
            Paragraph::new(Line::from(spans)).style(Style::default().bg(background_style));

        frame.render_widget(status_paragraph, area);

        let cursor_area = Rect {
            x: area.x + area.width.saturating_sub(cursor_info.len() as u16),
            y: area.y,
            width: cursor_info.len() as u16,
            height: 1,
        };

        let cursor_paragraph = Paragraph::new(cursor_info).style(
            Style::default()
                .fg(self.theme.foreground)
                .bg(background_style),
        );

        frame.render_widget(cursor_paragraph, cursor_area);
    }

    fn render_command_line(&self, frame: &mut Frame, app: &App, area: Rect) {
        let chunks = if app.error_message.is_some() {
            Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(1), Constraint::Length(1)])
                .split(area)
        } else {
            Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(1)])
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
        let paragraph = Paragraph::new(content).style(
            Style::default()
                .fg(self.theme.foreground)
                .bg(background_style),
        );

        frame.render_widget(paragraph, chunks[0]);

        if let Some(ref error_msg) = app.error_message {
            if chunks.len() > 1 {
                let error_paragraph = Paragraph::new(error_msg.clone())
                    .style(Style::default().fg(Color::Red).bg(background_style));
                frame.render_widget(error_paragraph, chunks[1]);
            }
        }
    }

    fn apply_highlighting<'a>(
        &self,
        content: &'a str,
        line_idx: usize,
        app: &App,
    ) -> Vec<Span<'a>> {
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

    fn get_text_style(
        &self,
        line_idx: usize,
        is_selected: bool,
        is_search_match: bool,
        app: &App,
    ) -> Style {
        let mut style = Style::default().fg(self.theme.foreground);

        if line_idx == app.cursor.line {
            style = style.bg(self.theme.current_line);
        }

        if is_selected {
            style = style.bg(self.theme.selection);
            match self.theme.selection {
                Color::Rgb(r, g, b) => {
                    let luminance =
                        (0.299 * r as f32 + 0.587 * g as f32 + 0.114 * b as f32) / 255.0;
                    if luminance > 0.5 {
                        style = style.fg(Color::Black);
                    } else {
                        style = style.fg(Color::White);
                    }
                }
                _ => {
                    style = style.fg(Color::White);
                }
            }
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
        let window_width = (area.width * 4 / 5).clamp(60, 80);
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
            format!(
                " Help ({}/{}) ",
                start_line + 1,
                app.help_window.content.len()
            )
        } else {
            " Help ".to_string()
        };

        let help_paragraph = Paragraph::new(visible_content)
            .style(
                Style::default()
                    .fg(self.theme.foreground)
                    .bg(self.theme.background),
            )
            .block(
                Block::default()
                    .borders(ratatui::widgets::Borders::ALL)
                    .title(title)
                    .title_style(
                        Style::default()
                            .fg(self.theme.foreground)
                            .add_modifier(ratatui::style::Modifier::BOLD),
                    )
                    .border_style(Style::default().fg(self.theme.foreground)),
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

    fn render_file_change_dialog(&self, frame: &mut Frame, app: &App, area: Rect) {
        let window_width = 60;
        let window_height = 8;

        let x = (area.width.saturating_sub(window_width)) / 2;
        let y = (area.height.saturating_sub(window_height)) / 2;

        let dialog_area = Rect {
            x: area.x + x,
            y: area.y + y,
            width: window_width,
            height: window_height,
        };

        let clear_widget = ratatui::widgets::Clear;
        frame.render_widget(clear_widget, dialog_area);

        let file_name = app
            .file_change_dialog
            .changed_file
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        let content = vec![
            Line::from(""),
            Line::from(format!(" File has been modified: {file_name}")),
            Line::from(""),
            Line::from(" Choose action:"),
            Line::from(""),
        ];

        let selected = app.file_change_dialog.selected_option;

        let option1_style = if selected == 0 {
            Style::default().fg(Color::Black).bg(Color::Yellow)
        } else {
            Style::default().fg(self.theme.foreground)
        };

        let option2_style = if selected == 1 {
            Style::default().fg(Color::Black).bg(Color::Yellow)
        } else {
            Style::default().fg(self.theme.foreground)
        };

        let dialog_paragraph = Paragraph::new(content)
            .style(
                Style::default()
                    .fg(self.theme.foreground)
                    .bg(self.theme.background),
            )
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" File Changed ")
                    .title_style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
                    .border_style(Style::default().fg(Color::Red)),
            );

        frame.render_widget(dialog_paragraph, dialog_area);

        let buttons_area = Rect {
            x: dialog_area.x + 2,
            y: dialog_area.y + 5,
            width: dialog_area.width.saturating_sub(4),
            height: 1,
        };

        let button1 = Span::styled(" [R]eload from disk ", option1_style);
        let spacer = Span::raw("  ");
        let button2 = Span::styled(" [K]eep current ", option2_style);

        let buttons_line = Line::from(vec![button1, spacer, button2]);
        let buttons_paragraph = Paragraph::new(vec![buttons_line]);

        frame.render_widget(buttons_paragraph, buttons_area);
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
            let _start_line = total_lines.saturating_sub(viewport_height);

            let content_height = viewport_height.saturating_sub(1);
            let display_lines = if terminal_output.lines.len() > content_height {
                terminal_output.lines.len() - content_height
            } else {
                0
            };

            let mut visible_lines: Vec<Line> = terminal_output
                .lines
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
            visible_lines.push(Line::from(Span::styled(
                prompt_line,
                self.theme.terminal_command,
            )));

            let paragraph = Paragraph::new(visible_lines).style(self.theme.terminal_background);

            frame.render_widget(paragraph, inner_area);

            let prompt_line = terminal_output.get_prompt_line();
            let cursor_x = inner_area.x + prompt_line.len() as u16;
            let cursor_y = inner_area.y + inner_area.height - 1;

            if cursor_x < inner_area.x + inner_area.width
                && cursor_y < inner_area.y + inner_area.height
                && app.should_show_cursor()
            {
                let cursor_area = Rect {
                    x: cursor_x,
                    y: cursor_y,
                    width: 1,
                    height: 1,
                };

                let cursor_widget = Paragraph::new("█").style(
                    Style::default()
                        .fg(self.theme.cursor)
                        .add_modifier(ratatui::style::Modifier::BOLD),
                );

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
                let status = Paragraph::new("[Running...]").style(self.theme.terminal_running);
                frame.render_widget(status, status_area);
            }
        }
    }
}
