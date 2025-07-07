pub mod app;
pub mod config;
pub mod editor;
pub mod file;
pub mod input;
pub mod syntax;
pub mod ui;
pub mod utils;

pub use app::App;
pub use config::*;
pub use editor::*;
pub use file::*;
pub use input::*;
pub use ui::*;

pub type Result<T> = anyhow::Result<T>;
