fn main() -> std::io::Result<()> {
    let options = pick_a_boo::Options::from(
        &vec!["Yes", "Maybe", "So so", "No"])
        .expect("Failed to create Options");
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
