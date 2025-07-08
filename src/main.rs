use clap::{Arg, Command};
use log::LevelFilter;
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Root};
use log4rs::encode::pattern::PatternEncoder;
use log4rs::Config;
use neocrim::input::EventHandler;
use neocrim::ui::{restore_terminal, setup_terminal, Renderer};
use neocrim::{App, Result};
use std::path::PathBuf;

fn main() -> Result<()> {
    let matches = Command::new("neocrim")
        .version("0.1.0")
        .author("Philipp01105")
        .about("A Neovim clone written in Rust")
        .arg(
            Arg::new("files")
                .help("Files to open")
                .value_name("FILE")
                .num_args(0..)
                .value_parser(clap::value_parser!(PathBuf)),
        )
        .arg(
            Arg::new("config")
                .long("config")
                .short('c')
                .help("Use custom config file")
                .value_name("CONFIG")
                .value_parser(clap::value_parser!(PathBuf)),
        )
        .get_matches();

    let logfile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{l} - {m}\n")))
        .build("log.log")?;

    let config = Config::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .build(
            Root::builder()
                .appender("logfile")
                .build(if cfg!(debug_assertions) {
                    LevelFilter::Debug
                } else {
                    LevelFilter::Off
                }),
        )?;

    log4rs::init_config(config).expect("TODO: panic message");

    log::info!("Starting neocrim...");

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
    let mut renderer =
        Renderer::new_with_glass_effects(app.config.theme.clone(), &app.config.current_theme);

    loop {
        app.update_cursor_blink();
        app.check_file_changes();
        renderer.update_theme_with_effects(app.config.theme.clone(), &app.config.current_theme);

        let (width, _) = crossterm::terminal::size()?;
        app.update_horizontal_scroll(width as usize);

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
