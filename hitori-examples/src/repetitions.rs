//! Repetition is done by annotating an all-pattern or an any-pattern with
//! `#[hitori::repeat]`.
//!
//! There are 5 possible name-value arguments:
//!
//! - **`eq = x`** – exactly `x` times
//! - **`lt = x`** – less than `x` times.
//! - **`le = x`** – less or equal to `x` times.
//! - **`gt = x`** – greater than `x` times.
//! - **`ge = x`** – greater or equal to `x` times.
//!
//! Value assigned to the bound must be either literal
//! [`usize`] (like `lt = 410` or `ge = 20usize`)
//! or literal string containing an expression that evaluates to
//! [`usize`] (like `eq = "self.name.len()"`).
//!
//! ```
#![doc = include_str!("repetitions/identifier.rs")]
//!
//! for s in ["_", "x1", "my_var32"] {
//!     assert!(hitori::string::starts_with(Identifier, s).is_some());
//! }
//! ```
//! *equivalent to `[[:alpha:]_]\w*` in [regex] syntax*
//!
//! ### Combining bounds
//!
//! Lower bounds (`gt` and `ge`) can be combined with upper bounds  (`lt` and `le`).
//! Default lower bound is `ge = 0`, while an upper bound is unbounded by default.
//!
//! ```
#![doc = include_str!("repetitions/binary_u32.rs")]
//!
//! assert!(hitori::string::starts_with(BinaryU32, "0b110011010").is_some());
//! ```
//! *equivalent to `0b[01]{1,32}` in [regex] syntax*
//!
//! ### Expression bounds
//!
//! Expression bounds can be used when the number of times to repeat
//! is not a literal [`usize`] (e.g. constants, function outputs
//! and [`ExprMut`] implementor's fields and methods).
//!
//! ```
#![doc = include_str!("repetitions/would_you_kindly.rs")]
//!
//! let s = "Would you kindly lower that weapon for a moment?";
//! let expr = WouldYouKindly::default();
//! let matched = hitori::string::starts_with(expr, s).unwrap();
//! assert_eq!(&s[matched.capture.request.unwrap()], "lower that weapon for a moment");
//!```
//! *equivalent to `Would you kindly (?P<request>[^?!]+)[?!]` in [regex] syntax*
//!
//! [regex]: https://docs.rs/regex
//! [`ExprMut`]: hitori::ExprMut

mod binary_u32;
mod identifier;
mod would_you_kindly;

pub use binary_u32::{BinaryU32, BinaryU32Capture};
pub use identifier::{Identifier, IdentifierCapture};
pub use would_you_kindly::{WouldYouKindly, WouldYouKindlyCapture};
