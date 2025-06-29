pub mod app;
pub mod editor;
pub mod ui;
pub mod input;
pub mod file;
pub mod config;
pub mod utils;
pub mod syntax;

pub use app::App;
pub use editor::*;
pub use ui::*;
pub use input::*;
pub use file::*;
pub use config::*;

pub type Result<T> = anyhow::Result<T>;
