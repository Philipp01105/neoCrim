# NeoCrim

> A modern, lightning-fast terminal text editor built with Rust 🦀

[![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg?style=for-the-badge)](https://opensource.org/licenses/MIT)
[![Terminal](https://img.shields.io/badge/Terminal-Text%20Editor-brightgreen?style=for-the-badge)](https://github.com/yourusername/neocrim)

## 🚀 Features

### Core Editing
- **Modal Editing** - Vim-inspired keybindings with Normal, Insert, Visual, and Command modes
- **Soft-Wrap Support** - Automatically wrap long lines for better readability
- **Syntax Highlighting** - Built-in support for 20+ programming languages
- **File Explorer** - Integrated file browser with directory navigation

### Search & Navigation
- **Powerful Search** - Pattern matching with result highlighting
- **Quick Navigation** - Jump to line numbers, word navigation
- **Search Results** - Navigate through matches with Enter/Shift+Enter
- **Real-time Highlighting** - Visual feedback for search results

### Customization
- **Custom Themes** - `.nctheme` files with TOML configuration
- **Configurable Settings** - Line numbers, syntax highlighting, tab size
- **Theme System** - Switch themes on-the-fly with `:theme` command
- **Extensible Config** - JSON-based configuration system

### User Experience
- **Help** - Comprehensive help window `:help`
- **Keyboard Shortcuts** - Efficient navigation and file management
- **Responsive UI** - Smooth terminal interface built with Ratatui

## 📦 Installation

### Prerequisites
- Rust 1.70+ (install from [rustup.rs](https://rustup.rs/))

### From Source
```bash
cargo install --git https://github.com/Philipp01105/neocrim.git --release
```
OR
```bash
git clone https//github.com/Philipp01105/neocrim.git
cd neocrim
cargo build --release
```

### Quick Start
```bash
# Open NeoCrim
neocrim

# Open a specific file
neocrim myfile.txt (cant be new files as of yet)

# Open with file explorer
neocrim --explorer
```

## 🎯 Usage

### Basic Commands

| Command | Description |
|---------|-------------|
| `:e <file>` | Edit/open file |
| `:w` | Save current file |
| `:wq` | Save and quit |
| `:q` | Quit editor |
| `:help` | Show help window |

### Search & Navigation

| Command | Description |
|---------|-------------|
| `:find <pattern>` | Search for pattern |
| `:findnext` | Go to next search result |
| `:findprev` | Go to previous search result |
| `:goto <line>` | Jump to line number |
| `:clear` | Clear search results |

### Settings

| Command | Description |
|---------|-------------|
| `:set numbers` | Show line numbers |
| `:set nonumbers` | Hide line numbers |
| `:set syntax` | Enable syntax highlighting |
| `:set nosyntax` | Disable syntax highlighting |

### Keyboard Shortcuts

#### Normal Mode
- `h/j/k/l` or Arrow Keys - Move cursor
- `w/b` - Jump to next/previous word
- `0/$` - Jump to beginning/end of line
- `g/G` - Jump to beginning/end of file
- `i` - Enter insert mode
- `a` - Enter insert mode after cursor
- `o` - Insert new line and enter insert mode
- `v` - Enter visual mode
- `x` - Delete character
- `:` - Enter command mode

#### File Explorer
- `F2` - Toggle file explorer
- `j/k` or Arrow Keys - Navigate files
- `Enter` - Open selected file
- `h` - Go to parent directory
- `r` - Refresh directory

#### Help Window
- `:help` - Open help window
- `↑/↓` - Scroll help content
- `Esc` - Close help window

## 🎨 Themes

NeoCrim supports custom themes with `.nctheme` files:

```toml
name = "My Theme"
author = "Your Name"
description = "A custom theme"

[colors]
background = { r = 40, g = 42, b = 54 }
foreground = { r = 248, g = 248, b = 242 }
cursor = { r = 248, g = 248, b = 242 }
# ... more colors
```

### Loading Themes
```bash
:theme path/to/theme.nctheme
```

### Built-in Themes
- `themes/dark.nctheme` - Dark theme with high contrast
- `themes/light.nctheme` - Light theme with soft colors
- `themes/solarized.nctheme` - Solarized theme for readability
- `themes/nord.nctheme` - Nord theme with icy colors
- `themes/monokai.nctheme` - Monokai theme for vibrant colors
- `themes/dracula.nctheme` - Dracula theme with dark background
- `themes/cyberpunk.nctheme` - Cyberpunk theme with neon colors

## 🛠️ Supported Languages

NeoCrim provides syntax highlighting for:

- **Systems**: Rust, C, C++, Go
- **Web**: JavaScript, TypeScript, HTML, CSS
- **Scripting**: Python, Lua, Shell, PHP, Ruby
- **Enterprise**: Java, C#
- **Data**: JSON, XML, YAML, SQL
- **Documentation**: Markdown

## ⚙️ Configuration

### Config File Location
- Linux/macOS: `~/.config/neocrim/config.json`
- Windows: `%APPDATA%/neocrim/config.json`

### Example Configuration
```json
{
  "editor": {
    "line_numbers": true,
    "syntax_highlighting": true,
    "tab_size": 4,
    "soft_wrap": true,
    "scroll_offset": 3
  },
  "theme": "themes/dark.nctheme"
}
```

## 🏗️ Architecture

NeoCrim is built with modern Rust architecture:

- **Ratatui** - Terminal UI framework
- **Tokio** - Async runtime
- **Ropey** - Efficient text buffer
- **Syntect** - Syntax highlighting engine
- **Crossterm** - Cross-platform terminal manipulation

### Project Structure
```
src/
├── app.rs              # Core application state
├── config/             # Configuration management
├── editor/             # Text editing logic
├── input/              # Input handling and commands
├── syntax/             # Syntax highlighting
├── ui/                 # User interface components
└── utils/              # Utility functions
```

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

### Development Setup
```bash
git clone https://github.com/yourusername/neocrim.git
cd neocrim
cargo build
cargo test
cargo run
```

### Code Style
- Follow Rust conventions with `rustfmt`
- Add tests for new features
- Update documentation for changes

## 📝 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- Inspired by Vim's modal editing paradigm
- Built with the amazing Rust ecosystem
- Thanks to the Ratatui community for the excellent TUI framework

---

<p align="center">
  <strong>Made with ❤️ and Rust</strong>
</p>

<p align="center">
  <a href="#-features">Features</a> •
  <a href="#-installation">Installation</a> •
  <a href="#-usage">Usage</a> •
  <a href="#-themes">Themes</a> •
  <a href="#-contributing">Contributing</a>
</p>
