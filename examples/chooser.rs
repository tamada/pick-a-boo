fn main() -> std::io::Result<()> {
    let options = pick_a_boo::OptionsBuilder::default()
        .item(pick_a_boo::Item::new("Yes", 'y', Some("I love it")))
        .item(pick_a_boo::item!("So so", description = "I like it, but sometimes it's hard"))
        .item(pick_a_boo::item!("Maybe", key = 'm', description = "I haven't tried it yet"))
        .item(pick_a_boo::item!("No", 'n', "I don't like it"))
        .build().unwrap();
    let mut picker = pick_a_boo::Picker::default();

    let answer = picker.choose("Do you like Rust?", options)?;
    match answer {
        Some(choice) => println!("You chose: {choice}"),
        None => println!("Cancelled."),
    }
    Ok(())
}