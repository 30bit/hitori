//! [`ExprMut`] implementor type can be generic and implementation
//! of the trait can be blanket.
//!
//! ```
#![doc = include_str!("generics/all_in.rs")]
//!
//! let lang = AllIn(&['+', '-', '<', '>', '.', ',', '[', ']', '\t', '\n', '\r']);
//! let prog = ">++++++++[<+++++++++>-]<.";
//! assert!(hitori::string::starts_with(lang, prog).is_some())
//! ```
//!
//! [`ExprMut`]: hitori::ExprMut

mod all_in;

pub use all_in::{AllIn, AllInCapture};
