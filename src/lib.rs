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
//! Here is just a simple example of how to use `pick-a-boo` in your Rust project:
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

#[cfg(test)]
extern crate self as pick_a_boo;

/// Macro to create an [Item] instance with flexible arguments.
/// The first positional argument is the long name (label) of the item.
/// Following arguments can be provided as either positional or named arguments:
/// 
/// ## Positional arguments:
/// 
/// The number of positional arguments can be from 1 to 4.
/// 
/// ```rust
/// use pick_a_boo::item;/// 
/// item!("LongName");                                  // only long name
/// item!("LongName", "ShortName");                     // long name and short name
/// item!("LongName", "ShortName", "Description");      // long name, short name, and description
/// item!("LongName", "ShortName", 'k', "Description"); // four positional arguments
/// ```
/// 
/// ## Named arguments:
/// 
/// The named arguments can be provided in any order after the first positional argument (long name).
/// 
/// ```rust
/// use pick_a_boo::item;
/// item!("LongName");                              // only long name
/// item!("LongName", key = 'k');                   // long name and key
/// item!("LongName", short = "ShortName");         // long name and short name
/// item!("LongName", description = "Description"); // long name and description
/// ```
///
/// ## Usage examples:
/// 
/// Exhaustive examples of various argument combinations:
///
/// ```rust
/// use pick_a_boo::item;
/// let a = item!("Alpha");                                  // Item::parse("Alpha")
/// let b = item!("Beta", "2");                              // Item::new_full("Beta", "2", "2", None)
/// let c = item!("Gamma", "3", "The third letter");         // Item::new_full("Gamma", "3", '3', Some("The third letter"))
/// let d = item!("Delta", "4", 'F', "The fourth letter");   // Item::new_full("Delta", "4", 'F', Some("The fourth letter"))
/// let e = item!("Epsilon", key = 'G');                     // Item::new_full("Epsilon", "G", 'G', None)
/// let f = item!("Zeta", description = "The sixth letter"); // Item::new_full("Zeta", "z", 'z', Some("The sixth letter"))
/// let g = item!("Eta", short = "7");                       // Item::new_full("Eta", "7", '7', None)
/// let h = item!("Theta", short = "A", key = '8');          // Item::new_full("Theta", "A", '8', None)
/// let i = item!("Iota", short = "A", description = "The nineth letter");
/// // Item::new_full("Iota", "A", 'A', Some("The nineth letter"))
/// 
/// let j = item!("Kappa", short = "10", description = "The tenth letter", key = 'u');
/// // Item::new_full("Kappa", "10", 'u', Some("The tenth letter"))
/// 
/// let k = item!("Lambda", key = 'u', short = "11");        // Item::new_full("Lambda", "11", 'u', None)
/// 
/// let l = item!("Mu", key = 'i', short = "12", description = "The twelveth letter");
/// // Item::new_full("Mu", "12", 'i', Some("The twelveth letter"))
/// 
/// let m = item!("Nu", key = 'i', description = "The thirteenth letter", short = "13");
/// // Item::new_full("Nu", "13", 'i', Some("The thirteenth letter"))
/// 
/// let n = item!("Xi", key = 'o', description = "The fourteenth letter");
/// // Item::new_full("Xi", "o", 'o', Some("The fourteenth letter"))
/// 
/// let o = item!("Omicron", short = "15", key = 'u', description = "the fifteenth letter");
/// // Item::new_full("Omicron", "15", 'u', Some("The fifteenth letter"))
/// 
/// let p = item!("Pi", description = "The sixteenth letter", short = "16");
/// // Item::new_full("Pi", "16", '1', Some("The sixteenth letter"))
/// 
/// let q = item!("Rho", description = "The seventeenth letter", key = 'K');
/// // Item::new_full("Rho", "K", 'K', Some("The seventeenth letter"))
/// 
/// let r = item!("Sigma", description = "The eighteenth letter", key = 'K', short = "k");
/// // Item::new_full("Sigma", "k", 'K', Some("The eighteenth letter"))
/// 
/// let s = item!("Tau", description = "The nineteenth letter", short = "k", key = 'K');
/// // Item::new_full("Tau", "k", 'K', Some("The nineteenth letter"))
/// 
/// let i = item!("", description = "empty");        // empty name then key and short are '\0'
/// ```
pub use pick_a_boo_macros::item;

/// Item struct represents a selectable option with a name, key, and optional description.
#[derive(Debug, Clone)]
pub struct Item {
    pub long_label: String,
    pub short_label: String,
    pub key: char,
    pub description: Option<String>,
}

impl Item {
    /// Create a new Item instance.
    pub fn new_full<S: AsRef<str>>(long_label: S, short_label: S, key: char, description: Option<S>) -> Self {
        let long_label = long_label.as_ref().to_string();
        let short_label = short_label.as_ref().to_string();
        let description = description.map(|d| d.as_ref().to_string());
        log::info!("create Item instance with new_full({long_label}, {short_label}, {key}, {description:?})");
        Item {
            long_label: long_label,
            short_label: short_label,
            key,
            description: description,
        }
    }

    pub fn new<S: AsRef<str>>(long_label: S, short_label: S, key: char) -> Self {
        Item::new_full(long_label, short_label, key, None)
    }

    /// Parse an item from a string.
    /// The key is derived from the first character of the name, converted to lowercase.
    /// If an uppercase key is desired, use the [`Item::new`] method or the [`item!`] macro.
    /// 
    /// The given string should formatted as "LongLabel[(ShortKey)][: Description]".
    /// If the colon `:` is present, the part after it is treated as the description.
    /// If not, the description is `None`.
    /// Also, `ShortKey` is optional and if it is not provided, the `ShortKey` is derived from the first character of the `LongLabel`.
    /// 
    /// ### Example
    /// 
    /// ```rust
    /// use pick_a_boo::Item;
    /// let item1 = Item::parse("Example");                     //  Item::new_full("Example", "e", 'e', None)
    /// let item2 = Item::parse("Test: This is just test");     //  Item::new_full("Test",    "t", 't', Some("This is just test"))
    /// let item3 = Item::parse("Colon: Its:too:many:colons!"); //  Item::new_full("Colon",   "c", 'c', Some("Its:too:many:colons!"))
    /// let item4 = Item::parse("Label(S): With short key");    //  Item::new_full("Label",   "S", 'S', Some("With short key"))
    /// ```
    pub fn parse(input: impl Into<String>) -> Self {
        let from_string = input.into();
        let (head, description) = match from_string.find(":") {
            Some(index) => {
                let head = from_string[..index].trim_end().to_string();
                let desc = from_string[index + 1..].trim().to_string();
                (head, Some(desc))
            }
            None => (from_string.to_string(), None),
        };
        if head.ends_with(")") {
            if let Some(start) = head.rfind("(") {
                let long_label = head[..start].trim_end().to_string();
                let short_label = head[start + 1..head.len() - 1].trim().to_string();
                let key = short_label.chars().next().unwrap_or('\0').to_ascii_lowercase();
                Item::new_full(long_label, short_label, key, description)
            } else {
                let long_label = head;
                let key = long_label.chars().next().unwrap_or('\0').to_ascii_lowercase();
                Item::new_full(long_label, key.to_string(), key, description)
            }
        } else {
            let long_label = head;
            let key = long_label.chars().next().unwrap_or('\0').to_ascii_lowercase();
            Item::new_full(long_label, key.to_string(), key, description)
        }
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
///     .item(Item::new_full("Yes", "y", 'y', Some("I love it")))
///     .item(item!("So so", description = "I like it, but sometimes it's hard")) 
///     .item(item!("Maybe", key = 'm', description = "I haven't tried it yet"))
///     .item(item!("No", "n", "I don't like it"))
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
        let item_vec = items.iter().map(|s| Item::parse(s.as_ref())).collect::<Vec<_>>();
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
        self.items[self.current].long_label.clone()
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
                    format!(" {} ", item.long_label)
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
///     .item(item!("Yes", "y", "I love it"))
///     .item(item!("So so", description = "I like it, but sometimes it's hard"))
///     .item(item!("Maybe", key = 'm', description = "I haven't tried it yet"))
///     .item(item!("No", "n", "I don't like it"))
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
        let yes_item = Item::new_full("Yes", "y", 'y', None);
        let no_item = Item::new_full("No", "n", 'n', None);
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

#[cfg(test)]
mod tests {
    use crate::item;

    #[test]
    fn test_optionsbuilder_duplicate_keys() {
        let result = crate::OptionsBuilder::default()
            .item(item!("Option 1", "o", "description 1"))
            .item(item!("Option 2", "o", "description 2")) // duplicate key
            .build();
        assert!(result.is_err());
    }

    #[test]
    fn test_optionsbuilder_out_of_bounds_current() {
        let result = crate::OptionsBuilder::default()
            .item(item!("Option 1", "1"))
            .item(item!("Option 2", "2"))
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
        assert_eq!(it.long_label, "Sample");
        assert_eq!(it.key, 's');
        assert!(it.description.is_none());
    }

    #[test]
    fn test_from_string() {
        let it: crate::Item = String::from("Example: This is example").into();
        assert_eq!(it.long_label, "Example");
        assert_eq!(it.key, 'e');
        assert_eq!(it.description.as_deref(), Some("This is example"));
    }

    #[test]
    fn test_macro_item_1() {
        let it = item!("Alpha");
        assert_eq!(it.long_label, "Alpha");
        assert_eq!(it.short_label, "a");
        assert_eq!(it.key, 'a');
        assert!(it.description.is_none());
    }

    #[test]
    fn test_macro_item_2() {
        let it = item!("Beta", "2");
        assert_eq!(it.long_label, "Beta");
        assert_eq!(it.short_label, "2");
        assert_eq!(it.key, '2');
        assert!(it.description.is_none())
    }

    #[test]
    fn test_macro_item_3() {
        let it = item!("Gamma", "3", "The third letter");
        assert_eq!(it.long_label, "Gamma");
        assert_eq!(it.short_label, "3");
        assert_eq!(it.key, '3');
        assert_eq!(it.description.as_deref(), Some("The third letter"));
    }

    #[test]
    fn test_macro_item_4() {
        let it = item!("Delta", "4", 'F', "The fourth letter");
        assert_eq!(it.long_label, "Delta");
        assert_eq!(it.short_label, "4");
        assert_eq!(it.key, 'F');
        assert_eq!(it.description.as_deref(), Some("The fourth letter"));
    }

    #[test]
    fn test_macro_item_5() {
        let it = item!("Epsilon", key = 'G');
        assert_eq!(it.long_label, "Epsilon");
        assert_eq!(it.short_label, "G");
        assert_eq!(it.key, 'G');
        assert!(it.description.is_none());
    }

    #[test]
    fn test_macro_item_6() {
        let it = item!("Zeta", description = "The sixth letter");
        assert_eq!(it.long_label, "Zeta");
        assert_eq!(it.short_label, "z");
        assert_eq!(it.key, 'z');
        assert_eq!(it.description.as_deref(), Some("The sixth letter"));
    }

    #[test]
    fn test_macro_item_7() {
        let it = item!("Eta", short = "7");
        assert_eq!(it.long_label, "Eta");
        assert_eq!(it.short_label, "7");
        assert_eq!(it.key, '7');
        assert!(it.description.is_none())
    }

    #[test]
    fn test_macro_item_8() {
        let it = item!("Theta", short = "A", key = '8');
        assert_eq!(it.long_label, "Theta");
        assert_eq!(it.short_label, "A");
        assert_eq!(it.key, '8');
        assert!(it.description.is_none());
    }

    #[test]
    fn test_macro_item_9() {
        let it = item!("Iota", short = "A", description = "The nineth letter");
        assert_eq!(it.long_label, "Iota");
        assert_eq!(it.short_label, "A");
        assert_eq!(it.key, 'A');
        assert_eq!(it.description.as_deref(), Some("The nineth letter"));
    }

    #[test]
    fn test_macro_item_10() {
        let it = item!("Kappa", short = "10", description = "The tenth letter", key = 'u');
        assert_eq!(it.long_label, "Kappa");
        assert_eq!(it.short_label, "10");
        assert_eq!(it.key, 'u');
        assert_eq!(it.description.as_deref(), Some("The tenth letter"));
    }

    #[test]
    fn test_macro_item_11() {
        let it = item!("Lambda", key = 'u', short = "11");
        assert_eq!(it.long_label, "Lambda");
        assert_eq!(it.short_label, "11");
        assert_eq!(it.key, 'u');
        assert!(it.description.is_none());
    }

    #[test]
    fn test_macro_item_12() {
        let it = item!("Mu", key = 'i', short = "12", description = "The twelveth letter");
        assert_eq!(it.long_label, "Mu");
        assert_eq!(it.short_label, "12");
        assert_eq!(it.key, 'i');
        assert_eq!(it.description.as_deref(), Some("The twelveth letter"));
    }

    #[test]
    fn test_macro_item_13() { // Nu
        let it = item!("Nu", key = 'i', description = "The thirteenth letter", short = "13");
        assert_eq!(it.long_label, "Nu");
        assert_eq!(it.short_label, "13");
        assert_eq!(it.key, 'i');
        assert_eq!(it.description.as_deref(), Some("The thirteenth letter"));
    }

    #[test]
    fn test_macro_item_14() {
        let it = item!("Xi", key = 'o', description = "The fourteenth letter");
        assert_eq!(it.long_label, "Xi");
        assert_eq!(it.short_label, "o");
        assert_eq!(it.key, 'o');
        assert_eq!(it.description.as_deref(), Some("The fourteenth letter"));
    }

    #[test]
    fn test_macro_item_15() {
        let it = item!("Omicron", short = "15", key = 'u', description = "The fifteenth letter");
        assert_eq!(it.long_label, "Omicron");
        assert_eq!(it.short_label, "15");
        assert_eq!(it.key, 'u');
        assert_eq!(it.description.as_deref(), Some("The fifteenth letter"));
    }

    #[test]
    fn test_macro_item_16() {
        let it = item!("Pi", description = "The sixteenth letter", short = "16");
        assert_eq!(it.long_label, "Pi");
        assert_eq!(it.short_label, "16");
        assert_eq!(it.key, '1');
        assert_eq!(it.description.as_deref(), Some("The sixteenth letter"));
    }

    #[test]
    fn test_macro_item_17() {
        let it = item!("Rho", description = "The seventeenth letter", key = 'K');
        assert_eq!(it.long_label, "Rho");
        assert_eq!(it.short_label, "K");
        assert_eq!(it.key, 'K');
        assert_eq!(it.description.as_deref(), Some("The seventeenth letter"));
    }

    #[test]
    fn test_macro_item_18() {
        let it = item!("Sigma", description = "The eighteenth letter", key = 'K', short = "k");
        assert_eq!(it.long_label, "Sigma");
        assert_eq!(it.short_label, "k");
        assert_eq!(it.key, 'K');
        assert_eq!(it.description.as_deref(), Some("The eighteenth letter"));
    }

    #[test]
    fn test_macro_item_19() {
        let it = item!("Tau", description = "The nineteenth letter", short = "k", key = 'K');
        assert_eq!(it.long_label, "Tau");
        assert_eq!(it.short_label, "k");
        assert_eq!(it.key, 'K');
        assert_eq!(it.description.as_deref(), Some("The nineteenth letter"));
    }

    #[test]
    fn test_macro_parse_with_short_and_description() {
        let it = item!("Upsilon(20): The twentieth letter");
        assert_eq!(it.long_label, "Upsilon");
        assert_eq!(it.short_label, "20");
        assert_eq!(it.key, '2');
        assert_eq!(it.description.as_deref(), Some("The twentieth letter"));
    }

    #[test]
    fn test_item_parse_without_description() {
        let it = crate::Item::parse("Phi");
        assert_eq!(it.long_label, "Phi");
        assert_eq!(it.short_label, "p");
        assert_eq!(it.key, 'p');
        assert!(it.description.is_none());
    }

    #[test]
    fn test_item_parse_with_description() {
        let it = crate::Item::parse("Chi: This is just test");
        assert_eq!(it.long_label, "Chi");
        assert_eq!(it.short_label, "c");
        assert_eq!(it.key, 'c');
        assert_eq!(it.description.as_deref(), Some("This is just test"));
    }

    #[test]
    fn test_item_parse_with_short_without_description() {
        let it = crate::Item::parse("Psi(Isp)");
        assert_eq!(it.long_label, "Psi");
        assert_eq!(it.short_label, "Isp");
        assert_eq!(it.key, 'i');
        assert!(it.description.is_none());
    }

    #[test]
    fn test_macro_item_with_empty_name() {
        let it = item!("");
        assert_eq!(it.long_label, "");
        assert_eq!(it.key, '\0');
        assert!(it.description.is_none())
    }
}