//! An any-pattern matches if one of its subpatterns matches.
//! In [hitori] syntax it is represented as an array of its subpatterns.
//!
//! ```
#![doc = include_str!("any_patterns/float_type.rs")]
//!
//! assert!(hitori::string::starts_with(FloatType, "f64").is_some());
//! assert!(hitori::string::starts_with(FloatType, "f128").is_none());
//! ```
//! *equivalent to `f(32|64)` in [regex] syntax*
//!
//! ### Empty any-pattern
//!
//! An empty any-pattern is always false.
//!
//! ```
#![doc = include_str!("any_patterns/false_.rs")]
//!
//! for s in ["Hello, world!", "34", "hitori"] {
//!     assert!(hitori::string::starts_with(False, s).is_none());
//! }
//! ```
//!
//! [hitori]: https://docs.rs/hitori
//! [regex]: https://docs.rs/regex

mod false_;
mod float_type;

pub use false_::{False, FalseCapture};
pub use float_type::{FloatType, FloatTypeCapture};
