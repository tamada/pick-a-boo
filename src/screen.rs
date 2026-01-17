//! Screen management for the picker application.
//! This module handles switching to alternate screens and managing
//! raw mode for terminal interactions.
use crossterm::{cursor, execute, queue, terminal};
use crate::{DescriptionShowMode, Options, Picker};

pub(crate) fn new(picker: &Picker, opts: &Options, stdout: &mut std::io::Stdout) -> std::io::Result<Screen> {
    if picker.alternate_screen {
        Ok(Screen::Alternae(Alternae::new(stdout)?))
    } else {
        Ok(Screen::Keep(NullGuard::new(picker, opts.items.len(), stdout)?))
    }
}

pub(super) enum Screen {
    Alternae(Alternae),
    Keep(NullGuard),
}

impl Screen {
    pub(crate) fn prepare_write(&mut self, stdout: &mut std::io::Stdout) -> std::io::Result<()> {
        match self {
            Screen::Alternae(g) => g.prepare_write(stdout),
            Screen::Keep(g) => g.prepare_write(stdout),
        }
    }
}

pub(super) struct NullGuard{
}

impl NullGuard {
    fn new(picker: &Picker, opts_len: usize, stdout: &mut std::io::Stdout) -> std::io::Result<Self> {
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

impl Drop for NullGuard {
    fn drop(&mut self) {
        let _ = execute!(std::io::stdout(), cursor::Show);
        terminal::disable_raw_mode().ok();
        println!();
    }
}

pub(super) struct Alternae;

impl Alternae {
    fn new(stdout: &mut std::io::Stdout) -> std::io::Result<Self> {
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

impl Drop for Alternae {
    fn drop(&mut self) {
        let _ = execute!(std::io::stdout(), cursor::Show, terminal::LeaveAlternateScreen);
        let _ = terminal::disable_raw_mode();
    }
}
