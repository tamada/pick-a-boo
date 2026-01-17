//! # pick-a-boo
//! A very simple option picker for CLI tools!
//!
//! ## Installation
//!
//! Add `pick-a-boo` to your project's dependencies:
//! 
//! ```toml
//! [dependencies]
//! pick-a-boo = "0.1"
//! ```
//!
//! ## Usage
//! 
//! Here is just a simplest example of how to use `pick-a-boo` in your Rust project:
//! 
//! ```rust,no_run
//! fn main() -> std::io::Result<()> {
//!     let options = pick_a_boo::Options::from(&["Yes", "So so", "Maybe", "No"])
//!         .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
//!     match pick_a_boo::choose("🦀 Do you like Rust? 🦀", options) {
//!         Ok(Some(choice)) if choice == "Yes"   => println!("You like Rust! 🤩"),
//!         Ok(Some(choice)) if choice == "So so" => println!("You feel so so about Rust."),
//!         Ok(Some(choice)) if choice == "Maybe" => println!("You are unsure about Rust."),
//!         Ok(Some(choice)) if choice == "No"    => println!("You don't like Rust... 😭"),
//!         Ok(Some(_)) => panic!("Unknown choice. never reach here!"),
//!         Ok(None) => println!("Cancelled."),
//!         Err(e) => return Err(std::io::Error::new(std::io::ErrorKind::Other, e)),
//!     }
//!     Ok(())
//! }
//! ```
//! 
//! When you run the program, you'll be presented with a prompt in your terminal:
//! 
//! ```text
//! Do you like Rust? [Yes /s/m/n]
//! Do you like Rust? [y/ So So /m/n]
//! Do you like Rust? [y/s/ Maybe /n]
//! Do you like Rust? [y/s/m/ No]
//! ```
//! 
//! - Navigate between "Yes" and "No" using the left and right arrow keys.
//! - Press Enter to select your choice.
//! - Press the corresponding key (y/s/m/n) to select an option directly.
//! - Press Ctrl+C or Escape to cancel (returns `None`).

use derive_builder::Builder;

mod screen;
mod routine;

/// Item struct represents a selectable option with a name, key, and optional description.
#[derive(Debug, Clone)]
pub struct Item {
    name: String,
    key: char,
    description: Option<String>,
}

/// Macro to create an [Item] instance with flexible arguments.
/// 
/// Usage examples:
/// ```rust
/// use pick_a_boo::item;
/// let a = item!("Alpha");                          // key is 'a'
/// let b = item!("Beta", 'B');                      // key is 'B' (specified takes precedence)
/// let c = item!("Gamma", 'G', "description");      // three positional arguments
/// let d = item!("Delta", description = "desc");    // key is 'd'
/// let e = item!("Epsilon", key = 'x');             // key is 'x'
/// let f = item!("Zeta", key = 'z', description = "desc");
/// let g = item!("Eta", description = "first", key = 'E'); // order doesn't matter
/// let h = item!("", description = "empty");        // empty name → key is '\0'
/// ```
#[macro_export]
macro_rules! item {
    // ---- named arguments first ----

    // named: name + key[required] + description[optional]
    ($name:expr, key = $key:expr $(, description = $desc:expr )? ) => {{
        let __name: ::std::string::String = ($name).into();
        let __key: ::core::primitive::char = $key;
        $crate::Item::new(__name, __key, $crate::item!(@desc_opt $( $desc )? ))
    }};

    // named: name + description[required] + key[optional]
    ($name:expr, description = $desc:expr $(, key = $key:expr )? ) => {{
        let __name: ::std::string::String = ($name).into();
        let __key: ::core::primitive::char = $crate::item!(@key_or_derive __name $( $key )? );
        $crate::Item::new(__name, __key, ::core::option::Option::Some(($desc).into()))
    }};

    // ---- positional arguments after named ----

    // positional: name only
    ($name:expr) => {{
        $crate::Item::parse($name)
    }};

    // positional: name and key
    ($name:expr, $key:expr) => {{
        let __name: ::std::string::String = ($name).into();
        let __key: ::core::primitive::char = $key;
        $crate::Item::new(__name, __key, ::core::option::Option::None)
    }};

    // positional: name, key, description
    ($name:expr, $key:expr, $desc:expr) => {{
        let __name: ::std::string::String = ($name).into();
        let __key: ::core::primitive::char = $key;
        $crate::Item::new(__name, __key, ::core::option::Option::Some(($desc).into()))
    }};

    // helper: optional description
    (@desc_opt) => { ::core::option::Option::<::std::string::String>::None };
    (@desc_opt $desc:expr) => { ::core::option::Option::Some(($desc).into()) };

    // helper: if key is specified, use it; otherwise derive from name
    (@key_or_derive $name_ident:ident) => {{
        match $name_ident.chars().next() {
            Some(ch) => ch.to_ascii_lowercase(),
            None => '\0',
        }
    }};
    (@key_or_derive $name_ident:ident $key:expr) => { $key };
}

impl Item {
    /// Create a new Item instance.
    pub fn new<S: AsRef<str>>(name: S, key: char, description: Option<S>) -> Self {
        Item {
            name: name.as_ref().to_string(),
            key,
            description: description.map(|d| d.as_ref().to_string()),
        }
    }

    /// Parse an item from a string.
    /// The key is derived from the first character of the name, converted to lowercase.
    /// If uppercase key is desired, use the [`Item::new`] method or the [`item!`] macro.
    /// 
    /// The given string should formatted with an optional description separated by a colon (`:`).
    /// 
    /// ### Example
    /// 
    /// ```rust
    /// use pick_a_boo::Item;
    /// let item1 = Item::parse("Example");                     //  Item::new("Example", 'e', None)
    /// let item2 = Item::parse("Test: This is just test");     //  Item::new("Test", 't', Some("This is just test"))
    /// let item3 = Item::parse("Colon: Its:too:many:colons!"); //  Item::new("Colon", 'c', Some("Its:too:many:colons!"))
    /// ```
    pub fn parse<S: AsRef<str>>(input: S) -> Self {
        let (name, desc) = match input.as_ref().split_once(":") {
            Some((n, d)) => (n.trim(), Some(d.trim().to_string())),
            None => (input.as_ref().trim(), None),
        };
        let key = name.chars().next().unwrap_or('\0').to_ascii_lowercase();
        Item::new(name, key, desc.as_deref())
    }
}

impl From<&str> for Item {
    fn from(s: &str) -> Self {
        Item::parse(s)
    }
}

impl From<&String> for Item {
    fn from(s: &String) -> Self {
        Item::parse(s)
    }
}

type ErrBox = Box<dyn std::error::Error + Send + Sync>;

/// Options struct holds a list of items and the current selection index.
/// To create an instance, use the `OptionBuilder` or the [`Options::from`] helper method.
/// 
/// ### Example: Create an instance with the builder
/// 
/// Each item can be created with the [`item!`] macro or the [`Item::new`] method.
/// The [`item!`] macro receives the item name as the first positional argument,
/// and optional key and description as either positional or named arguments.
/// 
/// ```rust
/// use pick_a_boo::{item, Item, OptionsBuilder};
/// let options = OptionsBuilder::default()
///     .item(Item::new("Yes", 'y', Some("I love it")))
///     .item(item!("So so", description = "I like it, but sometimes it's hard")) 
///     .item(item!("Maybe", key = 'm', description = "I haven't tried it yet"))
///     .item(item!("No", 'n', "I don't like it"));
/// ```
/// 
/// ### Example: Create an instance from a slice of strings
/// 
/// In this case, each string is converted to an `Item` instance with the [`Item::parse`] method.
/// 
/// ```rust
/// use pick_a_boo::Options;
/// let options = Options::from(&["Yes", "So so", "Maybe", "No"]);
/// ```
/// 
/// ### Errors
/// 
/// The builder will return an error if:
/// - No items are provided.
/// - The current index is out of bounds.
/// - There are duplicate keys among the items.
/// 
#[derive(Debug, Builder)]
#[builder(build_fn(validate = "validate_options", error = "ErrBox"))]
pub struct Options {
    #[builder(setter(each(name="item", into)))]
    items: Vec<Item>,
    #[builder(default = 0)]
    current: usize,
}

fn validate_options(options: &OptionsBuilder) -> Result<(), ErrBox> {
    let items = options.items.as_ref().ok_or("items must be set")?;
    let current = if let Some(current) = options.current {
        current
    } else {
        0
    };
    validate_option_items(items, current)
}

fn validate_option_items(items: &[Item], current: usize) -> Result<(), ErrBox> {
    if items.is_empty() {
        return Err("items cannot be empty".into());
    }
    if current >= items.len() {
        return Err(format!("{current}: current index is out of bounds (len: {})", items.len()).into());
    }
    if let Some(key) = find_duplicate_keys(items) {
        return Err(format!("{key}: duplicate key found").into());
    }
    Ok(())
}

fn find_duplicate_keys(items: &[Item]) -> Option<char> {
    use std::collections::HashSet;
    let mut keys = HashSet::new();
    for item in items {
        if !keys.insert(item.key) {
            return Some(item.key);
        }
    }
    None
}

impl Options {
    /// Helper method to create Options instance from a slice of strings.
    /// Each item of the slice is converted with [`Item::parse`] method.
    pub fn from<S: AsRef<str>>(items: &[S]) -> Result<Self, ErrBox> {
        let item_vec = items.iter().map(|s| Item::parse(s)).collect::<Vec<_>>();
        validate_option_items(&item_vec, 0)?;
        Ok(Options {
            items: item_vec,
            current: 0,
        })
    }

    fn next(&self, picker: &Picker) -> usize {
        let new_index = self.current + 1;
        if picker.allow_wrap {
            new_index % self.items.len()
        } else {
            std::cmp::min(new_index, self.items.len() - 1)
        }
    }

    fn previous(&self, picker: &Picker) -> usize {
        if self.current == 0 {
            if picker.allow_wrap {
                self.items.len() - 1
            } else {
                0
            }
        } else {
            self.current - 1
        }
    }

    /// Returns an iterator over the items.
    pub fn iter(&self) -> std::slice::Iter<'_, Item> {
        self.items.iter()
    }

    /// Returns the currently selected item.
    pub fn current_item(&self) -> &Item {
        &self.items[self.current]
    }

    /// Returns a Display struct for formatting the options for display with [Picker].
    pub fn display<'b>(&self, picker: &'b Picker) -> Display<'_, 'b> {
        Display(self, picker)
    }

    fn current_name(&self) -> String {
        self.items[self.current].name.clone()
    }

    fn update_current(self, index: usize) -> Self {
        Self {
            current: index,
            ..self
        }
    }
}

/// Display struct for formatting the options for display with [Picker].
pub struct Display<'a, 'b>(&'a Options, &'b Picker);
impl std::fmt::Display for Display<'_, '_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let picker = self.1;
        let display = self.0.iter().enumerate()
            .map(|(size, item)| {
                if size == self.0.current {
                    format!(" {} ", item.name)
                } else {
                    item.key.to_string()
                }
            }).collect::<Vec<_>>().join(&picker.delimiter);
        write!(f, "{display}")
    }
}

/// DescriptionShowMode enum defines how item descriptions are displayed.
#[derive(Debug, Clone)]
pub enum DescriptionShowMode {
    /// descriptions are never shown
    Never,
    /// only the current item's description is shown
    CurrentOnly,
    /// all item descriptions are shown
    All,
}

/// DescriptionNameWidth enum defines how the width of item names is determined.
/// This enum is used when displaying item descriptions ([`DescriptionShowMode::CurrentOnly`], and [`DescriptionShowMode::All`]).
#[derive(Debug, Clone)]
pub enum DescriptionNameWidth {
    /// no width adjustment is made.
    Never,
    /// fixed width adjustment is applied.
    Fixed(usize),
    /// auto width adjustment based on the maximum name length.
    Auto,
}

/// Picker struct is the main interface for choosing options.
/// It holds configuration for the picker behavior.
/// 
/// ### Example
/// 
/// ```rust
/// use pick_a_boo::{item, PickerBuilder};
/// 
/// ```
#[derive(Debug, Builder)]
#[builder(build_fn(error = "ErrBox"))]
pub struct Picker {
    #[builder(default = "/".to_string(), setter(into))]
    delimiter: String,
    #[builder(default = false)]
    alternate_screen: bool,
    #[builder(default = false)]
    allow_wrap: bool,
    #[builder(default = None, setter(strip_option, into, custom))]
    paren: Option<(String, String)>,
    #[builder(default = DescriptionShowMode::Never)]
    description_show_mode: DescriptionShowMode,
    #[builder(default = DescriptionNameWidth::Auto, setter(into))]
    description_name_width: DescriptionNameWidth,
}

impl PickerBuilder {
    pub fn paren<T: AsRef<str>>(&mut self, paren: T) -> &mut Self {
        let paren = paren.as_ref().to_string();
        if paren.is_empty() {
            self.paren = Some(None);
            return self;
        } else if paren.len() != 2 {
            self.paren = Some(Some((paren, "".to_string())));
            self
        } else {
            let mut chars = paren.chars();
            let left = chars.next().unwrap();
            let right = chars.next().unwrap();
            self.paren = Some(Some((left.to_string(), right.to_string())));
            self
        }
    }
}

impl Default for Picker {
    fn default() -> Self {
        PickerBuilder::default()
            .build().unwrap()
    }
}

impl Picker {
    pub fn choose(&mut self, prompt: &str, options: Options) -> std::io::Result<Option<String>> {
        routine::choose(self, prompt, options)
    }

    pub fn yes_or_no(&mut self, prompt: &str, default_yes: bool) -> std::io::Result<Option<bool>> {
        let yes_item = Item::new("Yes", 'y', None);
        let no_item = Item::new("No", 'n', None);
        let options = OptionsBuilder::default()
            .item(yes_item)
            .item(no_item)
            .current(if default_yes { 0 } else { 1 })
            .build().map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        let answer = self.choose(prompt, options);
        match answer {
            Ok(Some(choice)) if choice == "Yes" => Ok(Some(true)),
            Ok(Some(choice)) if choice == "No" => Ok(Some(false)),
            Ok(Some(_)) => Ok(None),
            Ok(None) => Ok(None),
            Err(e) => Err(e),
        }
    }
}

pub fn yes_or_no(prompt: &str, default_yes: bool) -> std::io::Result<Option<bool>> {
    Picker::default()
        .yes_or_no(prompt, default_yes)
}

pub fn choose(prompt: &str, options: Options) -> std::io::Result<Option<String>> {
    Picker::default()
        .choose(prompt, options)
}

mod tests {
    #[test]
    fn test_optionsbuilder_duplicate_keys() {
        let result = crate::OptionsBuilder::default()
            .item(item!("Option 1", 'a'))
            .item(item!("Option 2", 'a')) // duplicate key
            .build();
        assert!(result.is_err());
    }

    #[test]
    fn test_optionsbuilder_out_of_bounds_current() {
        let result = crate::OptionsBuilder::default()
            .item(item!("Option 1", 'a'))
            .item(item!("Option 2", 'b'))
            .current(10) // out of bounds
            .build();
        assert!(result.is_err());
    }

    #[test]
    fn test_optionsbuilder_no_items() {
        let result = crate::OptionsBuilder::default()
            .build();
        assert!(result.is_err());
    }

    #[test]
    fn test_macro_item_1() {
        let it = item!("Alpha");
        assert_eq!(it.name, "Alpha");
        assert_eq!(it.key, 'a');
        assert!(it.description.is_none());
    }

    #[test]
    fn test_macro_item_2() {
        let it = item!("Beta", description = "The second letter");
        assert_eq!(it.name, "Beta");
        assert_eq!(it.key, 'b');
        assert_eq!(it.description.as_deref(), Some("The second letter"));
    }

    #[test]
    fn test_macro_item_3() {
        let it = item!("Gamma", key = 'G');
        assert_eq!(it.name, "Gamma");
        assert_eq!(it.key, 'G');
        assert!(it.description.is_none());
    }

    #[test]
    fn test_macro_item_4() {
        let it = item!("Delta", key = 'D', description = "The fourth letter");
        assert_eq!(it.name, "Delta");
        assert_eq!(it.key, 'D');
        assert_eq!(it.description.as_deref(), Some("The fourth letter"));
    }

    #[test]
    fn test_macro_item_5() {
        let it = item!("Epsilon", 'E');
        assert_eq!(it.name, "Epsilon");
        assert_eq!(it.key, 'E');
        assert!(it.description.is_none())
    }

    #[test]
    fn test_macro_item_6() {
        let it = item!("Zeta", 'Z', "The sixth letter");
        assert_eq!(it.name, "Zeta");
        assert_eq!(it.key, 'Z');
        assert_eq!(it.description.as_deref(), Some("The sixth letter"));
    }

    #[test]
    fn test_macro_item_7() {
        let it = item!("Eta", description = "The seventh letter", key = 'e');
        assert_eq!(it.name, "Eta");
        assert_eq!(it.key, 'e');
        assert_eq!(it.description.as_deref(), Some("The seventh letter"));
    }

    #[test]
    fn test_macro_item_with_empty_name() {
        let it = item!("");
        assert_eq!(it.name, "");
        assert_eq!(it.key, '\0');
        assert!(it.description.is_none())
    }
}