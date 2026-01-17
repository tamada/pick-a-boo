//! Routine for handling user choice interactions.
use crate::{Options, Picker, screen};
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use crossterm::{cursor, queue, terminal};
use std::io::{IsTerminal, Write};

enum Action {
    Cancel,
    Confirm,
    Continue(usize),
    Next,
    Previous,
}

pub(crate) fn choose(picker: &mut Picker, prompt: &str, options: Options) -> std::io::Result<Option<String>> {
    let mut stdout = std::io::stdout();
    if !stdout.is_terminal() || std::io::stdin().is_terminal() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "not running on a TTY (interactive input is unavailable)",
        ));
    }
    let mut guard = screen::new(picker, &options, &mut stdout)?;
    let mut opts = options;
    let (paren_left, paren_right) = paren_strings(picker);

    loop {
        guard.prepare_write(&mut stdout)?;
        print!(
            "{prompt} {paren_left}{}{paren_right}",
            &opts.display(picker)
        );
        print_description(picker, &mut stdout, &opts);
        stdout.flush()?;

        if let Event::Key(key_event) = event::read()? {
            opts = match process_key(key_event.code, key_event.modifiers, &opts) {
                Action::Confirm => return Ok(Some(opts.current_name())),
                Action::Cancel => return Ok(None),
                Action::Continue(new_current) => opts.update_current(new_current),
                Action::Next => {
                    let new_index = opts.next(picker);
                    opts.update_current(new_index)
                }
                Action::Previous => {
                    let new_index = opts.previous(picker);
                    opts.update_current(new_index)
                }
            }
        }
    }
}

fn paren_strings(picker: &Picker) -> (String, String) {
    match &picker.paren {
        Some((left, right)) => (left.clone(), right.clone()),
        None => ("".to_string(), "".to_string()),
    }
}

fn print_description(picker: &Picker, stdout: &mut std::io::Stdout, opts: &Options) {
    use super::DescriptionShowMode;

    let name_width = calculate_name_width(picker, opts);
    match picker.description_show_mode {
        DescriptionShowMode::All => 
            write_all_descriptions(stdout, opts, name_width),
        DescriptionShowMode::CurrentOnly => 
            write_current_description(stdout, opts, name_width),
        DescriptionShowMode::Never => {}
    }
}

fn calculate_name_width(picker: &Picker, opts: &Options) -> usize {
    use super::DescriptionNameWidth::*;
    match picker.description_name_width {
        Fixed(w) => w,
        Never => 0,
        Auto => 
            opts.iter().map(|item| item.name.len()).max().unwrap_or(0),
    }  
}

fn write_current_description(stdout: &mut std::io::Stdout, opts: &Options, _name_width: usize) {
    let item = opts.current_item();
    queue!(
        stdout,
        cursor::MoveToNextLine(1),
        cursor::MoveToColumn(0),
        terminal::Clear(terminal::ClearType::CurrentLine)
    ).ok();
    print!(
        "    {:6} {}",
        item.name,
        item.description.clone().unwrap_or("".to_string())
    )
}

fn write_all_descriptions(stdout: &mut std::io::Stdout, opts: &Options, name_width: usize) {
    for (index, item) in opts.iter().enumerate() {
        let selected = if opts.current == index { ">" } else { " " };
        queue!(stdout, cursor::MoveToNextLine(1), cursor::MoveToColumn(0)).ok();
        print!(
            "{:1} {:w$} {}",
            selected,
            item.name,
            item.description.clone().unwrap_or("".to_string()),
            w = name_width
        );
    }
}

/// Process a key event and return the resulting action.
/// This is the pure logic extracted for testability.
fn process_key(key_code: KeyCode, modifiers: KeyModifiers, options: &Options) -> Action {
    if let KeyCode::Char(c) = key_code {
        if c == 'c' && modifiers.contains(KeyModifiers::CONTROL) {
            Action::Cancel
        } else {
            for (index, item) in options.iter().enumerate() {
                if item.key == c {
                    return Action::Continue(index);
                }
            }
            Action::Continue(options.current)
        }
    } else {
        match key_code {
            KeyCode::Left | KeyCode::Up => Action::Previous,
            KeyCode::Right | KeyCode::Down => Action::Next,
            KeyCode::Enter => Action::Confirm,
            KeyCode::Esc => Action::Cancel,
            _ => Action::Continue(options.current),
        }
    }
}
