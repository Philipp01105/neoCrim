use clap::{Arg, Command};
use neocrim::{App, Result};
use neocrim::input::EventHandler;
use neocrim::ui::{setup_terminal, restore_terminal, Renderer};
use std::path::PathBuf;

fn main() -> Result<()> {
    let matches = Command::new("neocrim")
        .version("0.1.0")
        .author("NeoCrim Team")
        .about("A modern Neovim clone written in Rust")
        .arg(
            Arg::new("files")
                .help("Files to open")
                .value_name("FILE")
                .num_args(0..)
                .value_parser(clap::value_parser!(PathBuf))
        )
        .arg(
            Arg::new("config")
                .long("config")
                .short('c')
                .help("Use custom config file")
                .value_name("CONFIG")
                .value_parser(clap::value_parser!(PathBuf))
        )
        .get_matches();

    let mut app = App::new()?;
    let mut event_handler = EventHandler::new();

    if let Some(files) = matches.get_many::<PathBuf>("files") {
        for file_path in files {
            if let Err(e) = app.open_file(file_path.clone()) {
                eprintln!("Error opening file {}: {}", file_path.display(), e);
            }
        }
    }

    let mut terminal = setup_terminal()?;
    let mut renderer = Renderer::new(app.config.theme.clone());

    loop {
        renderer.update_theme(app.config.theme.clone());
        
        terminal.draw(|frame| {
            renderer.render(frame, &mut app);
        })?;

        event_handler.handle_events(&mut app)?;

        if app.should_quit {
            break;
        }
    }

    restore_terminal()?;

    Ok(())
}
