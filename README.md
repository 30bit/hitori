[![hitori crate](https://img.shields.io/crates/v/hitori.svg)](https://crates.io/crates/hitori)
[![hitori documentation](https://docs.rs/hitori/badge.svg)](https://docs.rs/hitori)

Hitori is a generic regular expressions library. It works by creating series of if-statements for each expression at compile-time. Capturing is done through the traits.

# Example

```rust
struct Let {
    max: u32,
}

#[hitori::impl_expr(and_expr_mut)]
#[hitori::and_define(capture_mut, capture_ranges)]
impl<C: LetCaptureMut<Idx>, Idx: Clone> Expr<C, Idx, char> for Let {
    const PATTERN: _ = (
        |ch| ch == 'l',
        |ch| ch == 'e',
        |ch| ch == 't',
        char::is_whitespace,
        #[hitori::capture(var)]
        (char::is_alphabetic),
        char::is_whitespace,
        |ch| ch == '=',
        char::is_whitespace,
        #[hitori::capture(val)]
        (
            |ch: char| ch.to_digit(10).map(|d| d < self.max).unwrap_or_default(),
            |ch| ch == '.' || ch == ',',
            |ch: char| ch.is_digit(10),
        ),
        |ch| ch == ';'
    );
}

let text = "... let x = 5.1; ...";

let mut capture = LetCaptureRanges::default();
let found = hitori::string::find(Let { max: 6 }, &mut capture, text)
    .unwrap()
    .unwrap();
assert_eq!(&text[found], "let x = 5.1;");
assert_eq!(&text[capture.var.unwrap()], "x");
assert_eq!(&text[capture.val.unwrap()], "5.1");

let not_found = hitori::string::find_no_capture(Let { max: 4 }, text);
assert_eq!(not_found, None);
```
 
*See more code samples along with the traits, impls and struct they expand to in [`examples`](https://docs.rs/hitori/latest/hitori/examples/index.html).*

# Crate features

- **`box`** *(enabled by default)* – blanket implementations of all traits for boxes using alloc crate.
- **`macros`** *(enabled by default)* – `impl_expr_mut` and `impl_expr` macros.
- **`find-hitori`** – finds hitori package to be used in macros even if it has been renamed in Cargo.toml. **`macros`** feature is required.

# License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT) at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
