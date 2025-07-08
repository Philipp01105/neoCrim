use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

pub struct TextUtils;

impl TextUtils {
    pub fn display_width(text: &str) -> usize {
        text.width()
    }

    pub fn char_width(ch: char) -> usize {
        ch.width().unwrap_or(0)
    }

    pub fn split_lines(text: &str) -> Vec<&str> {
        text.lines().collect()
    }

    pub fn is_word_boundary(ch: char) -> bool {
        ch.is_whitespace() || ch.is_ascii_punctuation()
    }

    pub fn word_start(text: &str, pos: usize) -> usize {
        let chars: Vec<char> = text.chars().collect();
        if chars.is_empty() || pos >= chars.len() {
            return pos;
        }

        let mut start = pos;
        while start > 0 && !Self::is_word_boundary(chars[start - 1]) {
            start -= 1;
        }
        start
    }

    pub fn word_end(text: &str, pos: usize) -> usize {
        let chars: Vec<char> = text.chars().collect();
        if chars.is_empty() || pos >= chars.len() {
            return pos;
        }

        let mut end = pos;
        while end < chars.len() && !Self::is_word_boundary(chars[end]) {
            end += 1;
        }
        end
    }

    pub fn trim_line_end(line: &str) -> &str {
        line.trim_end()
    }

    pub fn expand_tabs(text: &str, tab_size: usize) -> String {
        let mut result = String::new();
        let mut col = 0;

        for ch in text.chars() {
            if ch == '\t' {
                let spaces_needed = tab_size - (col % tab_size);
                result.push_str(&" ".repeat(spaces_needed));
                col += spaces_needed;
            } else if ch == '\n' {
                result.push(ch);
                col = 0;
            } else {
                result.push(ch);
                col += Self::char_width(ch);
            }
        }

        result
    }

    pub fn column_position(line: &str, byte_pos: usize, tab_size: usize) -> usize {
        let mut col = 0;
        let mut pos = 0;

        for ch in line.chars() {
            if pos >= byte_pos {
                break;
            }

            if ch == '\t' {
                col += tab_size - (col % tab_size);
            } else {
                col += Self::char_width(ch);
            }

            pos += ch.len_utf8();
        }

        col
    }

    pub fn is_blank_line(line: &str) -> bool {
        line.trim().is_empty()
    }

    pub fn leading_whitespace(line: &str) -> usize {
        line.chars().take_while(|&ch| ch.is_whitespace()).count()
    }

    pub fn indentation_level(line: &str, tab_size: usize) -> usize {
        let mut level = 0;
        for ch in line.chars() {
            match ch {
                ' ' => level += 1,
                '\t' => level += tab_size,
                _ => break,
            }
        }
        level / tab_size
    }

    pub fn indent_string(level: usize, tab_size: usize, use_tabs: bool) -> String {
        if use_tabs {
            "\t".repeat(level)
        } else {
            " ".repeat(level * tab_size)
        }
    }

    pub fn test(){
        let _text = "Hello, world!\nThis is a test.\n";
    }
}
