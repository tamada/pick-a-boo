//! Macro to create an [pick_a_boo::Item] instance with flexible arguments.
//! 
//! Usage examples:
//! 
//! ```rust
//! use pick_a_boo::item;
//! let a = item!("Alpha");                          // key and short are 'a'
//! let b = item!("Beta", "B", "Description");       // key is 'B' (derived from short)
//! let c = item!("Gamma", "G", 'G', "description"); // four positional arguments
//! let d = item!("Delta", description = "desc");    // key and short are 'd' derived from long
//! let e = item!("Epsilon", key = 'x');             // key and short are 'x'
//! let f = item!("Zeta", short = "Z", description = "desc"); // key is 'Z' (derived from short)
//! let g = item!("Eta", key = 'z', description = "desc");    // short is 'z' (derived from key)
//! let h = item!("Theta", description = "first", key = 't', short = "T");  // order doesn't matter
//! let i = item!("", description = "empty");        // empty name then key and short are '\0'
//! ```
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse::{Parse, ParseStream}, parse_macro_input, Expr, Ident, Token, Result};

struct ItemInput {
    long: Expr,
    named_args: Vec<(String, Expr)>,
    positional_args: Vec<Expr>,
}

impl Parse for ItemInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let long = input.parse()?;
        let mut named_args = Vec::new();
        let mut positional_args = Vec::new();

        while !input.is_empty() {
            input.parse::<Token![,]>()?;
            if input.is_empty() { break; }

            // name = value の形式かチェック
            if input.peek(Ident) && input.peek2(Token![=]) {
                let name: Ident = input.parse()?;
                input.parse::<Token![=]>()?;
                let value: Expr = input.parse()?;
                named_args.push((name.to_string(), value));
            } else {
                positional_args.push(input.parse()?);
            }
        }
        Ok(ItemInput { long, named_args, positional_args })
    }
}

#[proc_macro]
pub fn item(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemInput);
    let long = &input.long;
    let path = quote! { ::pick_a_boo };

    // Conditional branching based on the number of positional arguments)
    if input.named_args.len() == 0 {
        if input.positional_args.len() == 0 {
            return quote! { #path::Item::parse(#long) }.into();
        }
        if input.positional_args.len() == 1 {
            let short = &input.positional_args[0];
            let key = quote! {
                #short.chars().next()
                    .and_then(|c| c.to_lowercase().next())
                    .unwrap_or('\0')
            };
            return quote! { #path::Item::new(#long, #short, #key) }.into();
        }
        if input.positional_args.len() == 2 {
            let short = &input.positional_args[0];
            let desc = &input.positional_args[1];
            let key = quote! {
                #short.chars().next()
                    .and_then(|c| c.to_lowercase().next())
                    .unwrap_or('\0')
            };
            return quote! { #path::Item::new_full(#long, &#short, #key, Some(#desc)) }.into();
        }
        if input.positional_args.len() == 3 {
            let short = &input.positional_args[0];
            let key_str = &input.positional_args[1];
            let desc = &input.positional_args[2];
            let key = quote! {
                #key_str.to_string()
                    .chars().next()
                    .unwrap_or('\0')
            };
            return quote! { #path::Item::new_full(#long, #short, #key, Some(#desc)) }.into();
        }
    }
    // Processing named arguments
    let mut short = quote! { None };
    let mut key = quote! { None };
    let mut desc = quote! { None };

    for (name, val) in input.named_args {
        match name.as_str() {
            "short" => short = quote! { Some(#val.to_string()) },
            "key" => key = quote! { Some(#val) },
            "description" => desc = quote! { Some(#val.to_string()) },
            _ => { todo!("generate compile errors") }
        }
    }

    // extracting code (specify Item by the absolute path)
    quote! {{
        let long_val = #long.to_string();
        let short_opt: Option<String> = #short;
        let key_opt: Option<char> = #key;
        let desc_opt: Option<String> = #desc;

        let s_final = short_opt.unwrap_or_else(|| {
            key_opt.as_ref().map(|k| k.to_string()).unwrap_or_else(|| {
                long_val.chars().next()
                    .map(|c| c.to_lowercase().to_string())
                    .unwrap_or_else(|| "\0".to_string())
            })
        });

        let k_final = key_opt.unwrap_or_else(|| {
            s_final.chars().next()
                .unwrap_or('\0')
        });

        #path::Item {
            long_label: long_val,
            short_label: s_final,
            key: k_final,
            description: desc_opt,
        }
    }}.into()
}
