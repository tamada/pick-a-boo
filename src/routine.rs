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

/// Ensure that both stdin and stdout are TTYs.
fn ensure_tty(stdout: std::io::Stdout) -> std::io::Result<std::io::Stdout> {
    if !stdout.is_terminal() || !std::io::stdin().is_terminal() {
        Err(std::io::Error::other(
            "not running on a TTY (interactive input is unavailable)",
        ))
    } else {
        Ok(stdout)
    }
}

pub(crate) fn choose(
    picker: &mut Picker,
    prompt: &str,
    options: Options,
) -> std::io::Result<Option<String>> {
    let mut stdout = ensure_tty(std::io::stdout())?;
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
        DescriptionShowMode::All => write_all_descriptions(stdout, opts, name_width),
        DescriptionShowMode::CurrentOnly => write_current_description(stdout, opts, name_width),
        DescriptionShowMode::Never => {}
    }
}

fn calculate_name_width(picker: &Picker, opts: &Options) -> usize {
    use super::DescriptionNameWidth::*;
    match picker.description_name_width {
        Fixed(w) => w,
        Never => 0,
        Auto => opts.iter().map(|item| item.name.len()).max().unwrap_or(0),
    }
}

fn write_current_description(stdout: &mut std::io::Stdout, opts: &Options, _name_width: usize) {
    let item = opts.current_item();
    queue!(
        stdout,
        cursor::MoveToNextLine(1),
        cursor::MoveToColumn(0),
        terminal::Clear(terminal::ClearType::CurrentLine)
    )
    .ok();
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

#[cfg(test)]
mod tests {
    mod paren_strings {
        use super::super::*;

        #[test]
        fn test_paren_strings() {
            let picker = crate::PickerBuilder::default().paren("()").build().unwrap();
            let (left, right) = paren_strings(&picker);
            assert_eq!(left, "(".to_string());
            assert_eq!(right, ")".to_string());
        }

        #[test]
        fn test_empty_paren() {
            let picker = crate::PickerBuilder::default().paren("").build().unwrap();
            let (left, right) = paren_strings(&picker);
            assert_eq!(left, "".to_string());
            assert_eq!(right, "".to_string());
        }


        #[test]
        fn test_only_left() {
            let picker = crate::PickerBuilder::default().paren(":").build().unwrap();
            let (left, right) = paren_strings(&picker);
            assert_eq!(left, ":".to_string());
            assert_eq!(right, "".to_string());
        }

        #[test]
        fn test_() {
            let picker = crate::PickerBuilder::default().paren("(<>)").build().unwrap();
            let (left, right) = paren_strings(&picker);
            assert_eq!(left, "(<".to_string());
            assert_eq!(right, ">)".to_string());
        }
    }

    mod process_key {
        use super::super::*;
        use crossterm::event::{KeyCode, KeyModifiers};
        #[test]
        fn cancel_with_ctrl_c() {
            let options = crate::OptionsBuilder::default()
                .item(crate::Item::new("Yes", 'y', None))
                .item(crate::Item::new("No", 'n', None))
                .build()
                .unwrap();
            let action = process_key(KeyCode::Char('c'), KeyModifiers::CONTROL, &options);
            match action {
                Action::Cancel => {}
                _ => panic!("Expected Cancel action"),
            }
        }

        #[test]
        fn cancel_with_esc() {
            let options = crate::OptionsBuilder::default()
                .item(crate::Item::new("Yes", 'y', None))
                .item(crate::Item::new("No", 'n', None))
                .build()
                .unwrap();
            let action = process_key(KeyCode::Esc, KeyModifiers::NONE, &options);
            match action {
                Action::Cancel => {}
                _ => panic!("Expected Cancel action"),
            }
        }

        #[test]
        fn continue_0() {
            let options = crate::OptionsBuilder::default()
                .item(crate::Item::new("Yes", 'y', None))
                .item(crate::Item::new("No", 'n', None))
                .build()
                .unwrap();
            let action = process_key(KeyCode::Char('y'), KeyModifiers::NONE, &options);
            match action {
                Action::Continue(item) => assert_eq!(item, 0),
                _ => panic!("Expected Cancel action"),
            }
        }

        #[test]
        fn continue_1() {
            let options = crate::OptionsBuilder::default()
                .item(crate::Item::new("Yes", 'y', None))
                .item(crate::Item::new("No", 'n', None))
                .build()
                .unwrap();
            let action = process_key(KeyCode::Char('n'), KeyModifiers::NONE, &options);
            match action {
                Action::Continue(item) => assert_eq!(item, 1),
                _ => panic!("Expected Cancel action"),
            }
        }

        #[test]
        fn continue_unrelated_key() {
            let options = crate::OptionsBuilder::default()
                .item(crate::Item::new("Yes", 'y', None))
                .item(crate::Item::new("No", 'n', None))
                .current(1)
                .build()
                .unwrap();
            let action = process_key(KeyCode::Char('x'), KeyModifiers::NONE, &options);
            match action {
                Action::Continue(item) => assert_eq!(item, 1),
                _ => panic!("Expected Cancel action"),
            }
        }

        #[test]
        fn confirm() {
            let options = crate::OptionsBuilder::default()
                .item(crate::Item::new("Yes", 'y', None))
                .item(crate::Item::new("No", 'n', None))
                .current(1)
                .build()
                .unwrap();
            let action = process_key(KeyCode::Enter, KeyModifiers::NONE, &options);
            match action {
                Action::Confirm => {}
                _ => panic!("Expected Confirm action"),
            }
        }

        #[test]
        fn with_arrow_up() {
            let options = crate::OptionsBuilder::default()
                .item(crate::Item::new("Yes", 'y', None))
                .item(crate::Item::new("No", 'n', None))
                .current(1)
                .build()
                .unwrap();
            let action = process_key(KeyCode::Up, KeyModifiers::NONE, &options);
            match action {
                Action::Previous => {}
                _ => panic!("Expected Confirm action"),
            }
        }

        #[test]
        fn with_arrow_right() {
            let options = crate::OptionsBuilder::default()
                .item(crate::Item::new("Yes", 'y', None))
                .item(crate::Item::new("No", 'n', None))
                .current(1)
                .build()
                .unwrap();
            let action = process_key(KeyCode::Right, KeyModifiers::NONE, &options);
            match action {
                Action::Next => {}
                _ => panic!("Expected Confirm action"),
            }
        }

        #[test]
        fn with_arrow_down() {
            let options = crate::OptionsBuilder::default()
                .item(crate::Item::new("Yes", 'y', None))
                .item(crate::Item::new("No", 'n', None))
                .current(1)
                .build()
                .unwrap();
            let action = process_key(KeyCode::Down, KeyModifiers::NONE, &options);
            match action {
                Action::Next => {}
                _ => panic!("Expected Confirm action"),
            }
        }

        #[test]
        fn with_arrow_left() {
            let options = crate::OptionsBuilder::default()
                .item(crate::Item::new("Yes", 'y', None))
                .item(crate::Item::new("No", 'n', None))
                .current(1)
                .build()
                .unwrap();
            let action = process_key(KeyCode::Left, KeyModifiers::NONE, &options);
            match action {
                Action::Previous => {}
                _ => panic!("Expected Confirm action"),
            }
        }
    }

    mod calculate_name_width {
        use crate::{OptionsBuilder, PickerBuilder};

        #[test]
        fn test_fixed_width() {
            let picker = PickerBuilder::default()
                .description_name_width(crate::DescriptionNameWidth::Fixed(7))
                .build()
                .unwrap();
            let options = OptionsBuilder::default()
                .item(crate::Item::new("Short", 's', None))
                .item(crate::Item::new("LongerName", 'l', None))
                .build()
                .unwrap();
            let width = crate::routine::calculate_name_width(&picker, &options);
            assert_eq!(width, 7);
        }

        #[test]
        fn test_auto_width() {
            let picker = PickerBuilder::default()
                .description_name_width(crate::DescriptionNameWidth::Auto)
                .build()
                .unwrap();
            let options = OptionsBuilder::default()
                .item(crate::Item::new("Short", 's', None))
                .item(crate::Item::new("LongerName", 'l', None))
                .build()
                .unwrap();
            let width = crate::routine::calculate_name_width(&picker, &options);
            assert_eq!(width, "LongerName".len());
        }

        #[test]
        fn test_never_width() {
            let picker = PickerBuilder::default()
                .description_name_width(crate::DescriptionNameWidth::Never)
                .build()
                .unwrap();
            let options = OptionsBuilder::default()
                .item(crate::Item::new("Short", 's', None))
                .item(crate::Item::new("LongerName", 'l', None))
                .build()
                .unwrap();
            let width = crate::routine::calculate_name_width(&picker, &options);
            assert_eq!(width, 0);
        }
    }
}
