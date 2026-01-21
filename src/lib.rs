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

impl From<String> for Item {
    fn from(s: String) -> Self {
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
///     .item(item!("No", 'n', "I don't like it"))
///     .current(1) // set the default selected index to 1 ("So so")
///     .build().expect("Failed to build Options");
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
    let current = options.current.unwrap_or(0);
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
        let item_vec = items.iter().map(Item::parse).collect::<Vec<_>>();
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
/// 
/// ### Example
/// 
/// Based following definition of options, it illustrates the differences in display modes.
/// 
/// ```rust
/// use pick_a_boo::{item, OptionsBuilder};
/// let opts = OptionsBuilder::default()
///     .item(item!("Yes", 'y', "I love it"))
///     .item(item!("So so", description = "I like it, but sometimes it's hard"))
///     .item(item!("Maybe", key = 'm', description = "I haven't tried it yet"))
///     .item(item!("No", 'n', "I don't like it"))
///     .build().expect("Failed to build Options");
/// ```
#[derive(Debug, Clone)]
pub enum DescriptionShowMode {
    /// Descriptions are never shown.
    /// Default mode. 
    /// 
    /// ```text
    /// Do you like Rust?  Yes /s/m/n
    /// ```
    Never,
    /// Only the current item's description is shown.
    /// 
    /// ```text
    /// Do you like Rust?  Yes /s/m/n
    ///   Yes    I love it
    /// ```
    /// 
    /// Other item is selected:
    /// ```text
    /// Do you like Rust?  y/s/ Maybe /n
    ///   Maybe  I haven't tried it yet 
    /// ```
    CurrentOnly,
    /// All item descriptions are shown.
    /// 
    /// ```text
    /// Do you like Rust?  Yes /s/m/n
    /// > Yes    I love it
    ///   So so  I like it, but sometimes it's hard
    ///   Maybe  I haven't tried it yet
    ///   No     I don't like it
    /// ```
    /// 
    /// Other item is selected:
    /// ```text
    /// Do you like Rust?  y/s/ Maybe /n
    ///   Yes    I love it
    ///   So so  I like it, but sometimes it's hard
    /// > Maybe  I haven't tried it yet
    ///   No     I don't like it
    /// ```
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
/// It holds the following configuration for the picker behavior.
/// 
/// ### Example
/// 
/// ```text
/// Do you like Rust? [Yes /s/m/n]
/// > Yes    I love it
///   So so  I like it, but sometimes it's hard
///   Maybe  I haven't tried it yet
///   No     I don't like it
/// ```
#[derive(Debug, Builder)]
#[builder(build_fn(error = "ErrBox"))]
pub struct Picker {
    /// Delimiter string used to separate options in the display.
    /// Defaults to "/".
    /// 
    /// The following example is using " | " as the delimiter:
    /// 
    /// ```text
    /// Do you like Rust? [Yes |s|m|n]
    /// ```
    #[builder(default = "/".to_string(), setter(into))]
    pub delimiter: String,
    #[builder(default = false)]
    /// Whether to use the alternate screen for the picker.
    /// Default is `false`.
    /// If `true`, the picker will switch to the alternate screen.
    /// 
    /// For more details, see [crossterm::terminal::EnterAlternateScreen].
    pub alternate_screen: bool,
    #[builder(default = false)]
    /// Whether to allow wrapping around when navigating options.
    /// Default is `false`.
    /// If `true`, navigating past the last option will wrap around to the first option,
    /// and vice versa.
    pub allow_wrap: bool,
    #[builder(default = None, setter(strip_option, into, custom))]
    /// Parentheses to enclose the options display.
    /// If `None`, no parentheses are used.
    /// If `Some((left, right))`, the options will be enclosed with the specified left and right strings.
    /// 
    /// In the [`PickerBuilder`], use the `paren(AsRef<str>)` method to set this field.
    /// see [PickerBuilder::paren] for details.
    pub paren: Option<(String, String)>,
    /// Mode for showing item descriptions. Default is [`DescriptionShowMode::Never`].
    /// see [`DescriptionShowMode`] for details.
    #[builder(default = DescriptionShowMode::Never)]
    pub description_show_mode: DescriptionShowMode,
    /// Width setting for item names when displaying descriptions.
    /// Default is [`DescriptionNameWidth::Auto`].
    /// see [`DescriptionNameWidth`] for details.
    #[builder(default = DescriptionNameWidth::Auto, setter(into))]
    pub description_name_width: DescriptionNameWidth,
}

impl PickerBuilder {
    /// If the given string has an even length, it will be split into two equal halves for left and right parentheses.
    /// Otherwise, the entire string will be used as the left parenthesis, and the right parenthesis will be an empty string.
    /// 
    /// For example:
    /// - `paren("()")` sets parentheses to `Some(("(", ")"))`.
    /// - `paren("[]")` sets parentheses to `Some(("[", "]"))`.
    /// - `paren("[[]]")` sets parentheses to `Some(("[[", "]]"))`.
    /// - `paren(":")` sets parentheses to `Some((":", ""))`.
    /// - `paren(":::")` sets parentheses to `Some((":::", ""))`.
    /// - `paren("")` sets parentheses to `Some(("", ""))`.
    /// - Not calling `paren` leaves it as `None`.
    pub fn paren<T: AsRef<str>>(&mut self, paren: T) -> &mut Self {
        let paren = paren.as_ref().to_string();
        log::info!("Setting paren: {paren}");
        if paren.is_empty() {
            self.paren = Some(None);
            self
        } else if paren.len() % 2 != 0 {
            self.paren = Some(Some((paren, "".to_string())));
            self
        } else {
            let len = paren.len() / 2;
            let l = paren.chars().take(len).collect::<String>();
            let r = paren.chars().skip(len).collect::<String>();
            self.paren = Some(Some((l, r)));
            self
        }
    }
}

impl Default for Picker {
    /// Create a default Picker instance, it equivalent the following code.
    /// 
    /// ```rust
    /// pick_a_boo::PickerBuilder::default()
    ///     .delimiter("/")
    ///     .alternate_screen(false)
    ///     .allow_wrap(false)
    ///     // .paren("")  // None by default
    ///     .description_show_mode(pick_a_boo::DescriptionShowMode::Never)
    ///     .description_name_width(pick_a_boo::DescriptionNameWidth::Auto)
    ///     .build().expect("Failed to build Picker");
    /// ```
    fn default() -> Self {
        log::info!("Building default Picker");
        PickerBuilder::default()
            .build().expect("Failed to build Picker")
    }
}

impl Picker {
    /// Choose an option from the provided [Options] with the given prompt.
    /// Returns `Ok(Some(String))` for the selected option name, and `Ok(None)` if cancelled.
    pub fn choose(&mut self, prompt: &str, options: Options) -> std::io::Result<Option<String>> {
        log::info!("Picker choosing with prompt: {prompt}");
        routine::choose(self, prompt, options)
    }

    /// Ask a yes-or-no question with the given prompt.
    /// The `default_yes` parameter determines the default selection.
    /// Returns `Ok(Some(true))` for "Yes", `Ok(Some(false))` for "No", and `Ok(None)` if cancelled.
    pub fn yes_or_no(&mut self, prompt: &str, default_yes: bool) -> std::io::Result<Option<bool>> {
        log::info!("Picker yes_or_no with prompt: {prompt}");
        let yes_item = Item::new("Yes", 'y', None);
        let no_item = Item::new("No", 'n', None);
        let options = OptionsBuilder::default()
            .item(yes_item)
            .item(no_item)
            .current(if default_yes { 0 } else { 1 })
            .build().map_err(std::io::Error::other)?;
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

/// Helper function to ask a yes-or-no question with the given prompt.
/// This routine is a shortcut for creating a default [Picker] instance and
/// calling its [Picker::yes_or_no] method.
/// 
/// ```rust
/// fn run_yes_or_no(prompt: &str, default_yes: bool) -> std::io::Result<Option<bool>> {
///     pick_a_boo::Picker::default()
///         .yes_or_no(prompt, default_yes)
/// }
/// ```
pub fn yes_or_no(prompt: &str, default_yes: bool) -> std::io::Result<Option<bool>> {
    Picker::default()
        .yes_or_no(prompt, default_yes)
}

/// Hellper function to choose an option from the provided [Options] with the given prompt.
/// This routine is a shortcut for creating a default [Picker] instance and
/// calling its [Picker::choose] method.
/// 
/// ```rust
/// fn run_pick_a_boo(prompt: &str, options: pick_a_boo::Options) -> std::io::Result<Option<String>> {
///     pick_a_boo::Picker::default()
///         .choose(prompt, options)
/// }
/// ```
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
    fn test_optionsbuilder_empty_items() {
        let result = crate::OptionsBuilder::default()
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
    fn test_from_str() {
        let it: crate::Item = "Sample".into();
        assert_eq!(it.name, "Sample");
        assert_eq!(it.key, 's');
        assert!(it.description.is_none());
    }

    #[test]
    fn test_from_string() {
        let it: crate::Item = String::from("Example: This is example").into();
        assert_eq!(it.name, "Example");
        assert_eq!(it.key, 'e');
        assert_eq!(it.description.as_deref(), Some("This is example"));
    }

    #[test]
    fn test_item_parse_without_description() {
        let it = crate::Item::parse("Example");
        assert_eq!(it.name, "Example");
        assert_eq!(it.key, 'e');
        assert!(it.description.is_none());
    }

    #[test]
    fn test_item_parse_with_description() {
        let it = crate::Item::parse("Test: This is just test");
        assert_eq!(it.name, "Test");
        assert_eq!(it.key, 't');
        assert_eq!(it.description.as_deref(), Some("This is just test"));
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