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

### A Simplest example

```rust
fn main() -> std::io::Result<()> {
  let answer = pick_a_boo::choose("Do you like Rust?",
      pick_a_boo::Options::from(vec!["Yes", "Maybe", "So so", "No"]));
  match answer {
    Ok(Some(choice)) if  =>   println!("I love Rust!"),
    Ok(Some(choice)) => println!("I like Rust, but sometimes it's hard"),
    OkSome(choice)) => println!("I haven't tried it yet"),
    Ok(Some(choice)) =>    println!("I don't like it"),
    Ok(Some(_)) => panic!("never reach here!")
    Ok(None)            => println!("You cancelled")
    Err(e) => 
  }
}
```

#### Output

```
Do you like Rust? Yes /s/m/n
```

Type `y`, `m`, `s`, or `n` to choose an option, or `Esc`/`Ctrl+C` to cancel.
Also, the arrow keys can be used to navigate the options.
The not selected option is shown only the first letter of an item.
Then, press `Enter` to confirm your choice.

### Display settings

```rust
use pick_a_boo::{Options, PickerBuilder};
fn main() -> std::io::Result<()> {
    let options = Options::from(vec!["Yes", "Maybe", "So so", "No"]))
    let picker = PickerBuilder::default()
        .alternate_screen(true)
        .allow_wrap(true)
        .paren("<>")
        .build()?;

    let answer = picker.choose("Do you like Rust?", options)?;
    match answer {
        Some(choice) => println!("You chose: {}", choice),
        None => println!("Cancelled."),
    }
    Ok(())
}
```

### Showing Descriptions

