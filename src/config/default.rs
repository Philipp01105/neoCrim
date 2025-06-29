use crate::config::Config;

pub const DEFAULT_CONFIG_TOML: &str = r#"
[editor]
line_numbers = true
relative_line_numbers = false
tab_size = 4
insert_tabs = false
auto_save = false
wrap_lines = false
scroll_offset = 5
syntax_highlighting = true

[ui]
theme = "default"
show_status_line = true
show_command_line = true
cursor_blink = true

[keybindings]
leader = " "  # Space as leader key

[theme]
background = { Rgb = [30, 30, 30] }      # #1e1e1e
foreground = { Rgb = [212, 212, 212] }   # #d4d4d4
cursor = { Rgb = [255, 255, 0] }         # #ffff00
selection = { Rgb = [38, 79, 120] }      # #264f78
line_number = { Rgb = [133, 133, 133] }  # #858585
current_line = { Rgb = [45, 45, 45] }    # #2d2d2d
status_bg = { Rgb = [0, 120, 215] }      # #0078d7
status_fg = { Rgb = [255, 255, 255] }    # #ffffff
command_bg = { Rgb = [30, 30, 30] }      # #1e1e1e
command_fg = { Rgb = [212, 212, 212] }   # #d4d4d4
"#;

pub fn generate_default_config() -> Config {
    Config::default()
}
