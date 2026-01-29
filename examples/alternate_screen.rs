fn main() -> std::io::Result<()> {
    let options = pick_a_boo::OptionsBuilder::default()
        .item(pick_a_boo::Item::new_full("Yes", "☺️", 'y', Some("I love it")))
        .item(pick_a_boo::item!("So so", key = '😄', description = "I like it, but sometimes it's hard"))
        .item(pick_a_boo::item!("Maybe", key = '😅', description = "I haven't tried it yet"))
        .item(pick_a_boo::item!("No", "😔", "I don't like it"))
        .build().expect("Failed to build Options");
    let mut picker = pick_a_boo::PickerBuilder::default()
        .alternate_screen(true)
        .allow_wrap(true)
        .description_show_mode(pick_a_boo::DescriptionShowMode::All)
        .description_name_width(pick_a_boo::DescriptionNameWidth::Auto)
        .build().expect("Failed to build Picker");

    let answer = picker.choose("Do you like Rust?", options)?;
    match answer {
        Some(choice) => println!("You chose: {choice}"),
        None => println!("Cancelled."),
    }
    Ok(())
}