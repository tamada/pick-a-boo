# pick-a-boo-macros

This crate provides procedural macros for the `pick-a-boo` crate.
It includes the `item!` macro, which simplifies the creation of `Item` instances by allowing users to specify only the desired fields, with sensible defaults for others.

Note that, **no need to use this crate directly**.
Just include `pick-a-boo` in your `Cargo.toml`, and the macros will be available for use.
