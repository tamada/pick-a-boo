fn main() -> std::io::Result<()> {
    let answer = pick_a_boo::yes_or_no("Do you like Rust?", true)?;
    match answer {
        Some(true) => println!("You like Rust!"),
        Some(false) => println!("You don't like Rust..."),
        None => println!("Cancelled."),
    }
    Ok(())
}
