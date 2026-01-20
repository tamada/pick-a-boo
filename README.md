# pick-a-boo

A simple terminal picker library for Rust.

This library is inspired by [`AntoineRenaud91/yes-or-no`](https://github.com/AntoineRenaud91/yes-or-no), and
extends its functionality to support multiple option selection.
The base design concept is originally from `yes-or-no`.

## Features

`pick-a-boo` has the following features:

- Navigate options using the arrow keys or assigned keys,
- Customizable prompt and separator,
- Showing descriptions, and
- Optional cancellation support.

## Example

The following examples are contained in `examples` folder.
To build them, run `cargo build --examples`, then the `cargo` puts the executables at `target/debug/examples` folder.

### A Simplest example (`simple_chooser.rs`)

```rust
fn main() -> std::io::Result<()> {
    let options = pick_a_boo::Options::from(
        &vec!["Yes", "Maybe", "So so", "No"]).unwrap();
    let answer = pick_a_boo::choose(
        "Do you like Rust?", options);
    match answer {
        Ok(Some(choice)) if &choice == "Yes"   => println!("I love Rust!"),
        Ok(Some(choice)) if &choice == "Maybe" => println!("I like Rust, but sometimes it's hard"),
        Ok(Some(choice)) if &choice == "So so" => println!("I haven't tried it yet"),
        Ok(Some(choice)) if &choice == "No"    => println!("I don't like it"),
        Ok(Some(_))   => panic!("never reach here!"),
        Ok(None)      => println!("You cancelled"),
        Err(e) => return Err(e),
    }
    Ok(())
}
```

#### Output

```text
Do you like Rust? Yes /s/m/n
```

Type `y`, `m`, `s`, or `n` to choose an option, or `Esc`/`Ctrl+C` to cancel.
Also, the arrow keys can be used to navigate the options.
The not selected option is shown only the first letter of an item.
Then, press `Enter` to confirm your choice.

### Display settings (`alternate_screen.rs`)

```rust
fn main() -> std::io::Result<()> {
    let options = pick_a_boo::OptionsBuilder::default()
        .item(pick_a_boo::Item::new("Yes", '😍', Some("I love it")))
        .item(pick_a_boo::item!("So so", key = '😄', description = "I like it, but sometimes it's hard"))
        .item(pick_a_boo::item!("Maybe", key = '😅', description = "I haven't tried it yet"))
        .item(pick_a_boo::item!("No", '😔', "I don't like it"))
        .build().unwrap();
    let mut picker = pick_a_boo::PickerBuilder::default()
        .alternate_screen(true)
        .allow_wrap(true)
        .description_show_mode(pick_a_boo::DescriptionShowMode::All)
        .description_name_width(pick_a_boo::DescriptionNameWidth::Auto)
        .build().expect("");

    let answer = picker.choose("Do you like Rust?", options)?;
    match answer {
        Some(choice) => println!("You chose: {choice}"),
        None => println!("Cancelled."),
    }
    Ok(())
}```
