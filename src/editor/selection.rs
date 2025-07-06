use crate::editor::Cursor;

#[derive(Debug, Clone)]
pub struct Selection {
    pub start: Cursor,
    pub end: Cursor,
    pub active: bool,
}

impl Selection {
    pub fn new() -> Self {
        Self {
            start: Cursor::new(),
            end: Cursor::new(),
            active: false,
        }
    }

    pub fn start_selection(&mut self, cursor: Cursor) {
        self.start = cursor;
        self.end = cursor;
        self.active = true;
    }

    pub fn update_selection(&mut self, cursor: Cursor) {
        if self.active {
            self.end = cursor;
        }
    }

    pub fn clear(&mut self) {
        self.active = false;
    }

    pub fn get_range(&self) -> Option<(Cursor, Cursor)> {
        if !self.active {
            return None;
        }

        let start = self.start;
        let end = self.end;

        let (ordered_start, ordered_end) =
            if start.line < end.line || (start.line == end.line && start.col <= end.col) {
                (start, end)
            } else {
                (end, start)
            };

        Some((ordered_start, ordered_end))
    }

    pub fn contains_position(&self, line: usize, col: usize) -> bool {
        if let Some((start, end)) = self.get_range() {
            if line < start.line || line > end.line {
                return false;
            }

            if line == start.line && line == end.line {
                col >= start.col && col <= end.col
            } else if line == start.line {
                col >= start.col
            } else if line == end.line {
                col <= end.col
            } else {
                true
            }
        } else {
            false
        }
    }
}

impl Default for Selection {
    fn default() -> Self {
        Self::new()
    }
}
