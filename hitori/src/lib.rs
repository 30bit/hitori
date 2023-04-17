//! Hitori is a generic regular expressions library. It works by creating series of
//! if-statements for each expression at compile-time. Capturing is done through the traits.
//!
//! # Example
//!
//! ```
//! // TODO
//! ```
//!  
//! *See more code samples along with impls and structs they expand to in [`examples`].*
//!
//! # Crate features
//!
//! - **`box`** *(enabled by default)* – blanket implementations of `hitori` traits
//!   for boxes using alloc crate.
//! - **`macros`** *(enabled by default)* – `impl_expr_mut` and `impl_expr` macros.
//! - **`find-hitori`** – finds hitori package to be used in macros
//!   even if it has been renamed in Cargo.toml. **`macros`** feature is required.

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

#[cfg(feature = "box")]
extern crate alloc;

#[cfg(all(
    any(feature = "examples", doc),
    feature = "box",
    feature = "macros",
    not(feature = "find_hitori")
))]
#[cfg_attr(doc, doc(cfg(doc)))]
pub mod examples;
pub mod string;

mod expr;
mod generic;

pub use expr::{Expr, ExprMut, Matched};
pub use generic::{find, matches, Found};

/// Implements `Expr` and optionally `ExprMut` for the struct.
///
/// *See [`examples`] for code samples along with impls and structs they expand to.*
///
/// # Arguments
///
/// - **`with_capture`** – sets the name of `Self::Capture` struct
/// - **`with_capture_vis`** – sets visibility of `Self::Capture` struct
#[cfg(feature = "macros")]
#[cfg_attr(doc, doc(cfg(feature = "macros")))]
pub use hitori_macros::impl_expr;

/// Implements `ExprMut` for the struct.
///
/// *See [`examples`] for code samples along with impls and structs they expand to.*
///
/// # Arguments
///
/// *Same as [`impl_expr`]*
#[cfg(feature = "macros")]
#[cfg_attr(doc, doc(cfg(feature = "macros")))]
pub use hitori_macros::impl_expr_mut;
