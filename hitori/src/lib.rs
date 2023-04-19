//! Hitori is generic compile-time regular expressions library.
//! It works by creating series of if-statements and for-loops for each expression.
//!  
//! *See code samples along with the traits, impls and structs they expand to in [`examples`].*
//!
//! # Limitations
//!
//! Pattern matching is step-by-step. It is impossible to to detach last element of a repetition.
//! For example, using [regex] one can rewrite `a+` as `a*a` and it would still match any
//! sequence of `a`s longer than zero. With [hitori], however, `a*` would consume
//! all the `a`s, and the expression won't match.
//!
//! Step-by step pattern matching also leads to diminished performance when matching
//! large texts with an expression that contains repetitions of characters frequent in the text.
//!
//! # Crate features
//!
//! - **`alloc`** *(enabled by default)* – string replace functions and blanket implementations
//!   of [hitori] traits for boxes using alloc crate.
//! - **`macros`** *(enabled by default)* – [`impl_expr_mut`] and [`impl_expr`] macros.
//! - **`find-hitori`** – finds hitori package to be used in macros
//!   even if it has been renamed in Cargo.toml. **`macros`** feature is required.
//! - **`examples`** – includes [`examples`] module into the build.
//!
//! [hitori]: https://docs.rs/hitori
//! [regex]: https://docs.rs/regex

#![no_std]
#![cfg_attr(
    doc,
    feature(doc_cfg),
    allow(mixed_script_confusables, confusable_idents)
)]

#[cfg(all(feature = "find-hitori", not(feature = "hitori-macros")))]
core::compile_error!(
    r#""find-hitori" feature doesn't do anything unless "macros" feature is enabled"#
);

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(all(
    any(doc, feature = "examples"),
    feature = "alloc",
    feature = "macros",
    not(feature = "find_hitori")
))]
#[cfg_attr(doc, doc(cfg(feature = "examples")))]
pub mod examples;
pub mod string;

mod expr;
mod generic;

pub use expr::{Expr, ExprMut, Match};
pub use generic::{find, starts_with};

/// Implements [`Expr`] and [`ExprMut`] for the type.
///
/// *See [`examples`] for code samples along with impls and structs they expand to.*
///
/// # Arguments
///
/// - **`with_capture`** – sets the name of [`ExprMut::Capture`] struct.
/// - **`with_capture_vis`** – sets visibility of [`ExprMut::Capture`] struct.
///
/// [`ExprMut::Capture`]: crate::expr::ExprMut::Capture
#[cfg(feature = "macros")]
#[cfg_attr(doc, doc(cfg(feature = "macros")))]
pub use hitori_macros::impl_expr;

/// Implements [`ExprMut`] for the type.
///
/// *See [`examples`] for code samples along with impls and structs they expand to.*
///
/// *See [`impl_expr`] for arguments description.*
#[cfg(feature = "macros")]
#[cfg_attr(doc, doc(cfg(feature = "macros")))]
pub use hitori_macros::impl_expr_mut;
