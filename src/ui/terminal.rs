use crate::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal as RatatuiTerminal};
use std::io::{self, Stdout, Write};

pub type Terminal = RatatuiTerminal<CrosstermBackend<Stdout>>;

pub fn setup_terminal() -> Result<Terminal> {
    enable_raw_mode()
        .map_err(|e| anyhow::anyhow!("Failed to enable raw mode: {}", e))?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    enable_transparency(&mut stdout)?;

    let backend = CrosstermBackend::new(stdout);
    let terminal = RatatuiTerminal::new(backend)?;
    Ok(terminal)
}

pub fn enable_transparency(stdout: &mut Stdout) -> Result<()> {
    write!(stdout, "\x1b]11;;\x07")?;
    write!(stdout, "\x1b[?1049h")?;
    stdout.flush()?;
    Ok(())
}

pub fn restore_terminal() -> Result<()> {
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;
    Ok(())
}
