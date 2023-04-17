[![hitori crate](https://img.shields.io/crates/v/hitori.svg)](https://crates.io/crates/hitori)
[![hitori documentation](https://docs.rs/hitori/badge.svg)](https://docs.rs/hitori)

Hitori is a generic partially regular expressions library. It works by creating series of if-statements for each expression at compile-time. Capturing is done through structs.
 
*See code samples along with the traits, impls and struct they expand to in [`examples`].*

# Crate features

- **`box`** *(enabled by default)* – blanket implementations of [hitori] traits for boxes using alloc crate.
- **`macros`** *(enabled by default)* – [`impl_expr_mut`] and [`impl_expr`] macros.
- **`find-hitori`** – finds hitori package to be used in macros even if it has been renamed in Cargo.toml. **`macros`** feature is required.
- **`examples`** – includes [`examples`] module into the build

# License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT) at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.

[`examples`]: https://docs.rs/hitori/latest/hitori/examples/index.html
[hitori]: https://docs.rs/hitori
[`impl_expr_mut`]: https://docs.rs/hitori/latest/hitori/attr.impl_expr.html
[`impl_expr`]: https://docs.rs/hitori/latest/hitori/attr.impl_expr.html