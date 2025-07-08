use crate::editor::Buffer;

#[derive(Debug, Clone, Copy)]
pub struct Cursor {
    pub line: usize,
    pub col: usize,
    pub desired_col: usize,
    pub visual_line_offset: usize,
}

impl Cursor {
    pub fn new() -> Self {
        Self {
            line: 0,
            col: 0,
            desired_col: 0,
            visual_line_offset: 0,
        }
    }

    pub fn calculate_visual_lines(&self, buffer: &Buffer, viewport_width: usize) -> (usize, usize) {
        if let Some(line_content) = buffer.line(self.line) {
            let line_width = line_content.len();
            if line_width <= viewport_width {
                (0, 1)
            } else {
                let visual_lines = line_width.div_ceil(viewport_width);
                let current_visual_line = self.col / viewport_width;
                (current_visual_line, visual_lines)
            }
        } else {
            (0, 1)
        }
    }

    pub fn move_down_visual(&mut self, buffer: &Buffer, viewport_width: usize) {
        if viewport_width == 0 {
            self.move_down(buffer);
            return;
        }

        let current_line_content = match buffer.line(self.line) {
            Some(content) => content,
            None => {
                self.move_down(buffer);
                return;
            }
        };

        let line_len = current_line_content.len();

        if line_len <= viewport_width {
            self.move_down(buffer);
            return;
        }

        let current_visual_line = self.col / viewport_width;
        let max_visual_lines = line_len.div_ceil(viewport_width);

        if current_visual_line + 1 < max_visual_lines {
            let next_line_start = (current_visual_line + 1) * viewport_width;
            let column_offset = self.col % viewport_width;
            self.col = (next_line_start + column_offset).min(line_len);
        } else if self.line + 1 < buffer.line_count() {
            self.line += 1;
            let column_offset = self.col % viewport_width;
            let new_line_len = buffer.line_len(self.line);
            self.col = column_offset.min(new_line_len);
        }

        self.desired_col = self.col % viewport_width;
    }

    pub fn move_up_visual(&mut self, buffer: &Buffer, viewport_width: usize) {
        if viewport_width == 0 {
            self.move_up(buffer);
            return;
        }

        let current_line_content = match buffer.line(self.line) {
            Some(content) => content,
            None => {
                self.move_up(buffer);
                return;
            }
        };

        let line_len = current_line_content.len();

        if line_len <= viewport_width {
            self.move_up(buffer);
            return;
        }

        let current_visual_line = self.col / viewport_width;

        if current_visual_line > 0 {
            let prev_line_start = (current_visual_line - 1) * viewport_width;
            let column_offset = self.col % viewport_width;
            self.col = prev_line_start + column_offset;
        } else if self.line > 0 {
            self.line -= 1;
            let prev_line_len = buffer.line_len(self.line);
            let column_offset = self.col % viewport_width;

            if prev_line_len > viewport_width {
                let prev_max_visual_lines =
                    prev_line_len.div_ceil(viewport_width);
                let target_line_start = (prev_max_visual_lines - 1) * viewport_width;
                self.col = (target_line_start + column_offset).min(prev_line_len);
            } else {
                self.col = column_offset.min(prev_line_len);
            }
        }

        self.desired_col = self.col % viewport_width;
    }

    pub fn move_left(&mut self, buffer: &Buffer) {
        if self.col > 0 {
            self.col -= 1;
            self.desired_col = self.col;
        } else if self.line > 0 {
            self.line -= 1;
            self.col = buffer.line_len(self.line);
            self.desired_col = self.col;
        }
    }

    pub fn move_right(&mut self, buffer: &Buffer) {
        let line_len = buffer.line_len(self.line);
        if self.col < line_len {
            self.col += 1;
            self.desired_col = self.col;
        } else if self.line + 1 < buffer.line_count() {
            self.line += 1;
            self.col = 0;
            self.desired_col = 0;
        }
    }

    pub fn move_up(&mut self, buffer: &Buffer) {
        if self.line > 0 {
            self.line -= 1;
            let line_len = buffer.line_len(self.line);
            self.col = self.desired_col.min(line_len);
        }
    }

    pub fn move_down(&mut self, buffer: &Buffer) {
        if self.line + 1 < buffer.line_count() {
            self.line += 1;
            let line_len = buffer.line_len(self.line);
            self.col = self.desired_col.min(line_len);
        }
    }

    pub fn move_line_start(&mut self) {
        self.col = 0;
        self.desired_col = 0;
    }

    pub fn move_line_end(&mut self, buffer: &Buffer) {
        self.col = buffer.line_len(self.line);
        self.desired_col = self.col;
    }

    pub fn move_file_start(&mut self) {
        self.line = 0;
        self.col = 0;
        self.desired_col = 0;
    }

    pub fn move_file_end(&mut self, buffer: &Buffer) {
        if buffer.line_count() > 0 {
            self.line = buffer.line_count() - 1;
            self.col = buffer.line_len(self.line);
            self.desired_col = self.col;
        }
    }

    pub fn move_word_forward(&mut self, buffer: &Buffer) {
        let current_line = buffer.line(self.line);
        if let Some(line_content) = current_line {
            let chars: Vec<char> = line_content.chars().collect();

            while self.col < chars.len() && !chars[self.col].is_whitespace() {
                self.col += 1;
            }

            while self.col < chars.len() && chars[self.col].is_whitespace() {
                self.col += 1;
            }

            if self.col >= chars.len() && self.line + 1 < buffer.line_count() {
                self.line += 1;
                self.col = 0;
            }

            self.desired_col = self.col;
        }
    }

    pub fn move_word_backward(&mut self, buffer: &Buffer) {
        if self.col > 0 {
            let current_line = buffer.line(self.line);
            if let Some(line_content) = current_line {
                let chars: Vec<char> = line_content.chars().collect();
                self.col -= 1;

                while self.col > 0 && chars[self.col].is_whitespace() {
                    self.col -= 1;
                }

                while self.col > 0 && !chars[self.col - 1].is_whitespace() {
                    self.col -= 1;
                }

                self.desired_col = self.col;
            }
        } else if self.line > 0 {
            self.line -= 1;
            self.col = buffer.line_len(self.line);
            self.desired_col = self.col;
        }
    }

    pub fn clamp_to_buffer(&mut self, buffer: &Buffer) {
        if self.line >= buffer.line_count() {
            self.line = buffer.line_count().saturating_sub(1);
        }

        let line_len = buffer.line_len(self.line);
        if self.col > line_len {
            self.col = line_len;
        }

        self.desired_col = self.col;
    }

    pub fn clamp_to_buffer_insert_mode(&mut self, buffer: &Buffer) {
        if self.line >= buffer.line_count() {
            self.line = buffer.line_count().saturating_sub(1);
        }

        let line_len = buffer.line_len(self.line);
        if self.col > line_len {
            self.col = line_len;
        }

        self.desired_col = self.col;
    }

    pub fn move_right_insert_mode(&mut self, buffer: &Buffer) {
        let line_len = buffer.line_len(self.line);
        log::debug!(
            "cursor position: {} {}, line len: {}",
            self.line,
            self.col,
            line_len
        );
        if self.col < line_len {
            self.col += 1;
            self.desired_col = self.col;
        } else if self.line + 1 < buffer.line_count() {
            self.line += 1;
            self.col = 0;
            self.desired_col = 0;
        }
    }

    pub fn move_left_insert_mode(&mut self, buffer: &Buffer) {
        if self.col > 0 {
            self.col -= 1;
            self.desired_col = self.col;
        } else if self.line > 0 {
            self.line -= 1;
            self.col = buffer.line_len(self.line);
            self.desired_col = self.col;
        }
    }

    pub fn move_up_insert_mode(&mut self, buffer: &Buffer) {
        if self.line > 0 {
            self.line -= 1;
            let line_len = buffer.line_len(self.line);
            if self.col > line_len {
                self.col = line_len;
            }
            self.desired_col = self.col;
        }
    }

    pub fn move_down_insert_mode(&mut self, buffer: &Buffer) {
        if self.line + 1 < buffer.line_count() {
            self.line += 1;
            let line_len = buffer.line_len(self.line);
            if self.col > line_len {
                self.col = line_len;
            }
            self.desired_col = self.col;
        }
    }
}

impl Default for Cursor {
    fn default() -> Self {
        Self::new()
    }
}
