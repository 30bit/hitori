[![hitori crate](https://img.shields.io/crates/v/hitori.svg)](https://crates.io/crates/hitori)
[![hitori documentation](https://docs.rs/hitori/badge.svg)](https://docs.rs/hitori)
[![Build status](https://github.com/30bit/hitori/workflows/build/badge.svg)](https://github.com/30bit/hitori/actions)

Hitori is generic compile-time regular expressions library. 
It works by creating series of if-statements and for-loops for each expression. 
 
*See code samples along with the traits, impls and structs they expand to in [examples].*

# Limitations

Pattern matching is step-by-step. It is impossible to to detach last element of a repetition. 
For example, using [regex] one can rewrite `a+` as `a*a` and it would still match any 
sequence of `a`s longer than zero. With [hitori], however, `a*` would consume
all the `a`s, and the expression won't match. 

Step-by step pattern matching also leads to diminished performance when matching
large texts and an expression contains repetitions of frequent characters.

# Crate features

- **`alloc`** *(enabled by default)* – string replace functions and blanket implementations 
  of [hitori] traits for boxes using alloc crate.
- **`macros`** *(enabled by default)* – [`impl_expr_mut`] and [`impl_expr`] macros.
- **`find-hitori`** – finds hitori package to be used in macros 
  even if it has been renamed in Cargo.toml. **`macros`** feature is required.
- **`examples`** – includes [examples] module into the build.

# License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT) at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.

[examples]: https://docs.rs/hitori-examples
[regex]: https://docs.rs/regex
[hitori]: https://docs.rs/hitori
[`impl_expr_mut`]: https://docs.rs/hitori/latest/hitori/attr.impl_expr.html
[`impl_expr`]: https://docs.rs/hitori/latest/hitori/attr.impl_expr.html