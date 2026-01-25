//! Screen management for the picker application.
//! This module handles switching to alternate screens and managing
//! raw mode for terminal interactions.
use crossterm::{cursor, execute, queue, terminal};
use crate::{DescriptionShowMode, Options, Picker};

pub(crate) fn new(picker: &Picker, opts: &Options, stdout: &mut std::io::Stdout) -> std::io::Result<Screen> {
    log::info!("Initializing screen mode: alternate_screen={}", picker.alternate_screen);
    if picker.alternate_screen {
        Ok(Screen::A(Alternate::new(stdout)?))
    } else {
        Ok(Screen::K(Keeper::new(picker, opts.items.len(), stdout)?))
    }
}

pub(super) enum Screen {
    /// Alternate screen mode.
    /// use crossterm's `EnterAlternateScreen` and `LeaveAlternateScreen`
    A(Alternate),
    /// Keeper mode.
    /// keeps the original screen content, and the outputs of this library below them.
    K(Keeper),
}

impl Screen {
    pub(crate) fn prepare_write(&mut self, stdout: &mut std::io::Stdout) -> std::io::Result<()> {
        match self {
            Screen::A(g) => g.prepare_write(stdout),
            Screen::K(g) => g.prepare_write(stdout),
        }
    }
}

pub(super) struct Keeper{
}

impl Keeper {
    fn new(picker: &Picker, opts_len: usize, stdout: &mut std::io::Stdout) -> std::io::Result<Self> {
        log::info!("Entering not-alternate screen mode");
        let mode = picker.description_show_mode.clone();
        let up = match mode {
            DescriptionShowMode::All => opts_len + 1,
            DescriptionShowMode::CurrentOnly => 1,
            DescriptionShowMode::Never => 0,
        };
        for _ in 0..up { // obtain the draw space in advance
            println!();
        }
        terminal::enable_raw_mode()?;
        queue!(stdout, cursor::Hide, cursor::MoveUp(up as u16),cursor::SavePosition)?;
        Ok(Self{})
    }

    fn prepare_write(&mut self, stdout: &mut std::io::Stdout) -> std::io::Result<()> {
        queue!(
            stdout,
            cursor::RestorePosition,
            terminal::Clear(terminal::ClearType::CurrentLine)
        )
    }
}

impl Drop for Keeper {
    fn drop(&mut self) {
        log::info!("Leaving not-alternate screen mode");
        let _ = execute!(std::io::stdout(), cursor::Show);
        terminal::disable_raw_mode().ok();
        println!();
    }
}

pub(super) struct Alternate;

impl Alternate {
    fn new(stdout: &mut std::io::Stdout) -> std::io::Result<Self> {
        log::info!("Entering alternate screen mode");
        terminal::enable_raw_mode()?;
        queue!(stdout, cursor::Hide, terminal::EnterAlternateScreen, cursor::MoveTo(0, 0), cursor::SavePosition)?;
        Ok(Self)
    }

    fn prepare_write(&mut self, stdout: &mut std::io::Stdout) -> std::io::Result<()> {
        queue!(
            stdout,
            cursor::RestorePosition,
            terminal::Clear(terminal::ClearType::CurrentLine)
        )
    }
}

impl Drop for Alternate {
    fn drop(&mut self) {
        log::info!("Leaving alternate screen mode");
        let _ = execute!(std::io::stdout(), cursor::Show, terminal::LeaveAlternateScreen);
        let _ = terminal::disable_raw_mode();
    }
}
