//! Sequences of subpatterns can be matched using an all-pattern.
//! In [hitori] syntax it is represented as a tuple of its subpatterns.
//!
//! ```
#![doc = include_example!("all_patterns/hello")]
//!
//! assert!(hitori::string::starts_with(Hello, "hello").is_some());
//! assert!(hitori::string::starts_with(Hello, "world").is_none());
//! ```
//! *equivalent to `hello` in [regex] syntax*
//!
//! ### Trailing comma
//!
//! The only way to apply attributes such as `#[hitori::capture]` or
//! `#[hitori::repeat]` to a single character test is by wrapping it
//! inside of an all-pattern. In that case trailing comma
//! is **not** optional.
//!
//! ```
#![doc = include_example!("all_patterns/bad_password")]
//!
//! assert!(hitori::string::starts_with(BadPassword, "12345").is_some());
//! assert!(hitori::string::starts_with(BadPassword, "cUFK^06#43Gs").is_none());
//! ```
//! *equivalent to `\d{1, 8}` in [regex] syntax*
//!
//! ### Empty all-pattern
//!
//! An empty all-pattern is always true.
//!
//! ```
#![doc = include_example!("all_patterns/true_")]
//!
//! for s in ["Hello, world!", "34", "hitori"] {
//!     assert!(hitori::string::starts_with(True, s).is_some());
//! }
//! ```
//!
//! [hitori]: https://docs.rs/hitori
//! [regex]: https://docs.rs/regex

mod bad_password;
mod hello;
mod true_;

pub use bad_password::{BadPassword, BadPasswordCapture};
pub use hello::{Hello, HelloCapture};
pub use true_::{True, TrueCapture};

use super::include_example;
