use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap},
};
use std::process::{Command, Stdio};
use std::io::{BufRead, BufReader};

#[derive(Debug, Clone)]
pub struct TerminalOutput {
    pub lines: Vec<String>,
    pub scroll_offset: usize,
    pub is_running: bool,
    pub current_input: String,
    pub command_history: Vec<String>,
    pub history_index: Option<usize>,
}

impl TerminalOutput {
    pub fn new() -> Self {
        Self {
            lines: vec!["Terminal ready. Type commands directly or use :cmd <command>".to_string()],
            scroll_offset: 0,
            is_running: false,
            current_input: String::new(),
            command_history: Vec::new(),
            history_index: None,
        }
    }

    pub fn add_line(&mut self, line: String) {
        self.lines.push(line);
        if self.lines.len() > 1000 {
            self.lines.remove(0);
        }
    }

    pub fn clear(&mut self) {
        self.lines.clear();
        self.lines.push("Terminal cleared.".to_string());
        self.scroll_offset = 0;
        self.current_input.clear();
        self.history_index = None;
    }

    pub fn scroll_up(&mut self) {
        if self.scroll_offset > 0 {
            self.scroll_offset -= 1;
        }
    }

    pub fn scroll_down(&mut self, viewport_height: usize) {
        let max_scroll = self.lines.len().saturating_sub(viewport_height);
        if self.scroll_offset < max_scroll {
            self.scroll_offset += 1;
        }
    }

    pub fn execute_command(&mut self, command: &str) -> Result<(), Box<dyn std::error::Error>> {
        if command.trim().is_empty() {
            return Ok(());
        }

        self.add_line(format!("$ {}", command));
        self.is_running = true;

        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            return Ok(());
        }

        let mut cmd = Command::new(parts[0]);
        if parts.len() > 1 {
            cmd.args(&parts[1..]);
        }

        match cmd.stdout(Stdio::piped()).stderr(Stdio::piped()).spawn() {
            Ok(mut child) => {
                if let Some(stdout) = child.stdout.take() {
                    let reader = BufReader::new(stdout);
                    for line in reader.lines() {
                        match line {
                            Ok(line) => self.add_line(line),
                            Err(_) => break,
                        }
                    }
                }

                if let Some(stderr) = child.stderr.take() {
                    let reader = BufReader::new(stderr);
                    for line in reader.lines() {
                        match line {
                            Ok(line) => self.add_line(format!("ERROR: {}", line)),
                            Err(_) => break,
                        }
                    }
                }

                match child.wait() {
                    Ok(status) => {
                        if !status.success() {
                            self.add_line(format!("Command exited with code: {}", 
                                status.code().unwrap_or(-1)));
                        }
                    }
                    Err(e) => {
                        self.add_line(format!("Failed to wait for command: {}", e));
                    }
                }
            }
            Err(e) => {
                self.add_line(format!("Failed to execute command '{}': {}", command, e));
            }
        }

        self.is_running = false;
        Ok(())
    }

    pub fn handle_input_char(&mut self, ch: char) {
        self.current_input.push(ch);
        self.history_index = None;
    }

    pub fn handle_backspace(&mut self) {
        self.current_input.pop();
        self.history_index = None;
    }

    pub fn handle_enter(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.current_input.trim().is_empty() {
            let command = self.current_input.clone();
            self.command_history.push(command.clone());
            self.current_input.clear();
            self.history_index = None;
            self.execute_command(&command)?;
        } else {
            self.add_line("".to_string());
        }
        Ok(())
    }

    pub fn history_up(&mut self) {
        if !self.command_history.is_empty() {
            match self.history_index {
                None => {
                    self.history_index = Some(self.command_history.len() - 1);
                    self.current_input = self.command_history[self.command_history.len() - 1].clone();
                }
                Some(index) if index > 0 => {
                    self.history_index = Some(index - 1);
                    self.current_input = self.command_history[index - 1].clone();
                }
                _ => {}
            }
        }
    }

    pub fn history_down(&mut self) {
        if let Some(index) = self.history_index {
            if index + 1 < self.command_history.len() {
                self.history_index = Some(index + 1);
                self.current_input = self.command_history[index + 1].clone();
            } else {
                self.history_index = None;
                self.current_input.clear();
            }
        }
    }

    pub fn get_prompt_line(&self) -> String {
        format!("$ {}", self.current_input)
    }
}

pub struct Terminal {
    pub output: TerminalOutput,
    pub visible: bool,
    pub title: String,
}

impl Terminal {
    pub fn new() -> Self {
        Self {
            output: TerminalOutput::new(),
            visible: false,
            title: "Terminal".to_string(),
        }
    }

    pub fn show(&mut self) {
        self.visible = true;
    }

    pub fn hide(&mut self) {
        self.visible = false;
    }

    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }

    pub fn render(&mut self, area: Rect, buf: &mut Buffer, theme: &crate::ui::theme::Theme) {
        if !self.visible {
            return;
        }

        let block = Block::default()
            .title(self.title.as_str())
            .borders(Borders::ALL)
            .border_style(theme.terminal_border)
            .title_style(theme.terminal_title);

        let inner_area = block.inner(area);
        block.render(area, buf);

        let viewport_height = inner_area.height as usize;
        let total_lines = self.output.lines.len();
        let scroll_offset = if total_lines <= viewport_height {
            0
        } else {
            if self.output.scroll_offset == 0 {
                total_lines.saturating_sub(viewport_height)
            } else {
                self.output.scroll_offset.min(total_lines.saturating_sub(viewport_height))
            }
        };

        let visible_lines: Vec<Line> = self.output.lines
            .iter()
            .skip(scroll_offset)
            .take(viewport_height)
            .map(|line| {
                let style = if line.starts_with("$ ") {
                    theme.terminal_command
                } else if line.starts_with("ERROR:") {
                    theme.terminal_error
                } else {
                    theme.terminal_output
                };
                Line::from(Span::styled(line.clone(), style))
            })
            .collect();

        let paragraph = Paragraph::new(visible_lines)
            .style(theme.terminal_background)
            .wrap(Wrap { trim: false });

        paragraph.render(inner_area, buf);

        if total_lines > viewport_height {
            let scrollbar_area = Rect {
                x: area.x + area.width - 1,
                y: area.y + 1,
                width: 1,
                height: area.height - 2,
            };

            let mut scrollbar_state = ScrollbarState::new(total_lines)
                .position(scroll_offset);

            let scrollbar = Scrollbar::default()
                .orientation(ScrollbarOrientation::VerticalRight)
                .style(theme.scrollbar);

            scrollbar.render(scrollbar_area, buf, &mut scrollbar_state);
        }

        if self.output.is_running {
            let status_area = Rect {
                x: area.x + 2,
                y: area.y,
                width: 12,
                height: 1,
            };
            let status = Paragraph::new("[Running...]")
                .style(theme.terminal_running);
            status.render(status_area, buf);
        }
    }
}

impl Default for Terminal {
    fn default() -> Self {
        Self::new()
    }
}