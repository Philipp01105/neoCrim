use ratatui::style::{Color, Style as RatatuiStyle};
use std::path::Path;
use syntect::highlighting::{Style, ThemeSet};
use syntect::parsing::{SyntaxReference, SyntaxSet};

pub struct SyntaxHighlighter {
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
}

impl SyntaxHighlighter {
    pub fn new() -> Self {
        Self {
            syntax_set: SyntaxSet::load_defaults_newlines(),
            theme_set: ThemeSet::load_defaults(),
        }
    }

    pub fn detect_language(&self, file_path: Option<&Path>) -> Option<&SyntaxReference> {
        if let Some(path) = file_path {
            if let Some(extension) = path.extension().and_then(|ext| ext.to_str()) {
                return self.syntax_set.find_syntax_by_extension(extension);
            }
            
            if let Some(filename) = path.file_name().and_then(|name| name.to_str()) {
                return self.syntax_set.find_syntax_by_name(filename);
            }
        }
        
        None
    }

    pub fn get_language_by_extension(&self, extension: &str) -> Option<&SyntaxReference> {
        match extension.to_lowercase().as_str() {
            "rs" => self.syntax_set.find_syntax_by_extension("rs"),
            "c" => self.syntax_set.find_syntax_by_extension("c"),
            "cpp" | "cc" | "cxx" | "hpp" | "h" => self.syntax_set.find_syntax_by_extension("cpp"),
            "java" => self.syntax_set.find_syntax_by_extension("java"),
            "cs" => self.syntax_set.find_syntax_by_extension("cs"),
            "py" => self.syntax_set.find_syntax_by_extension("py"),
            "js" | "mjs" => self.syntax_set.find_syntax_by_extension("js"),
            "ts" => self.syntax_set.find_syntax_by_extension("ts"),
            "html" | "htm" => self.syntax_set.find_syntax_by_extension("html"),
            "css" => self.syntax_set.find_syntax_by_extension("css"),
            "json" => self.syntax_set.find_syntax_by_extension("json"),
            "xml" => self.syntax_set.find_syntax_by_extension("xml"),
            "yaml" | "yml" => self.syntax_set.find_syntax_by_extension("yaml"),
            "lua" => self.syntax_set.find_syntax_by_extension("lua"),
            "go" => self.syntax_set.find_syntax_by_extension("go"),
            "php" => self.syntax_set.find_syntax_by_extension("php"),
            "rb" => self.syntax_set.find_syntax_by_extension("rb"),
            "sh" | "bash" => self.syntax_set.find_syntax_by_extension("sh"),
            "sql" => self.syntax_set.find_syntax_by_extension("sql"),
            "md" => self.syntax_set.find_syntax_by_extension("md"),
            _ => None,
        }
    }

    pub fn highlight_line(&self, line: &str, syntax: &SyntaxReference, theme_colors: &crate::ui::themes::ThemeColors) -> Vec<(RatatuiStyle, String)> {
        use syntect::parsing::ParseState;
        let mut parse_state = ParseState::new(syntax);
        let ops = parse_state.parse_line(line, &self.syntax_set)
            .unwrap_or_default();
        
        let mut result = Vec::new();
        let mut current_pos = 0;
        let mut scope_stack = syntect::parsing::ScopeStack::new();
        
        for (pos, op) in ops {
            if pos > current_pos {
                let text = &line[current_pos..pos];
                if !text.is_empty() {
                    let color = scope_to_theme_color(&scope_stack, theme_colors);
                    let style = RatatuiStyle::default().fg(color);
                    result.push((style, text.to_string()));
                }
            }
            
            scope_stack.apply(&op).unwrap_or(());
            current_pos = pos;
        }
        
       
        if current_pos < line.len() {
            let text = &line[current_pos..];
            if !text.is_empty() {
                let color = scope_to_theme_color(&scope_stack, theme_colors);
                let style = RatatuiStyle::default().fg(color);
                result.push((style, text.to_string()));
            }
        }
        
        if result.is_empty() && !line.is_empty() {
            result.push((
                RatatuiStyle::default().fg(theme_colors.foreground.to_ratatui_color()),
                line.to_string()
            ));
        }
        
        result
    }

    pub fn get_supported_languages(&self) -> Vec<(&str, Vec<&str>)> {
        vec![
            ("Rust", vec!["rs"]),
            ("C", vec!["c", "h"]),
            ("C++", vec!["cpp", "cc", "cxx", "hpp"]),
            ("Java", vec!["java"]),
            ("C#", vec!["cs"]),
            ("Python", vec!["py", "pyw"]),
            ("JavaScript", vec!["js", "mjs"]),
            ("TypeScript", vec!["ts"]),
            ("HTML", vec!["html", "htm"]),
            ("CSS", vec!["css"]),
            ("JSON", vec!["json"]),
            ("XML", vec!["xml"]),
            ("YAML", vec!["yaml", "yml"]),
            ("Lua", vec!["lua"]),
            ("Go", vec!["go"]),
            ("PHP", vec!["php"]),
            ("Ruby", vec!["rb"]),
            ("Shell", vec!["sh", "bash"]),
            ("SQL", vec!["sql"]),
            ("Markdown", vec!["md"]),
        ]
    }
}

impl Default for SyntaxHighlighter {
    fn default() -> Self {
        Self::new()
    }
}

fn scope_to_theme_color(scope_stack: &syntect::parsing::ScopeStack, theme_colors: &crate::ui::themes::ThemeColors) -> Color {
    let scope_str = scope_stack.to_string();
    
    if scope_str.contains("keyword") {
        theme_colors.syntax_keyword.to_ratatui_color()
    } else if scope_str.contains("string") {
        theme_colors.syntax_string.to_ratatui_color()
    } else if scope_str.contains("comment") {
        theme_colors.syntax_comment.to_ratatui_color()
    } else if scope_str.contains("constant.numeric") || scope_str.contains("constant.character") {
        theme_colors.syntax_number.to_ratatui_color()
    } else if scope_str.contains("entity.name.function") || scope_str.contains("support.function") {
        theme_colors.syntax_function.to_ratatui_color()
    } else if scope_str.contains("entity.name.type") || scope_str.contains("storage.type") {
        theme_colors.syntax_type.to_ratatui_color()
    } else if scope_str.contains("constant") {
        theme_colors.syntax_constant.to_ratatui_color()
    } else if scope_str.contains("variable") {
        theme_colors.syntax_variable.to_ratatui_color()
    } else if scope_str.contains("keyword.operator") || scope_str.contains("punctuation.operator") {
        theme_colors.syntax_operator.to_ratatui_color()
    } else if scope_str.contains("punctuation") {
        theme_colors.syntax_punctuation.to_ratatui_color()
    } else {
        theme_colors.foreground.to_ratatui_color()
    }
}