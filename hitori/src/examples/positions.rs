//! Annotating an all-pattern or an any-pattern with `#[hitori::position]` checks
//! position of a matched subpattern relative to input start.
//!
//! There are 2 possible arguments:
//!
//! - **`first`** – subpattern matched from the beginning of an input
//! - **`last`** – subpattern matched to the end of an input
//!
//! ```
#![doc = include_example!("positions/train_cars")]
//!
//! assert!(hitori::string::starts_with(TrainCars, "🚃").is_some());
//! assert!(hitori::string::starts_with(TrainCars, "🚃🚃🚃🚃🚃").is_some());
//! assert!(hitori::string::starts_with(TrainCars, " 🚃").is_none());
//! assert!(hitori::string::starts_with(TrainCars, "🚃 ").is_none());
//! assert!(hitori::string::starts_with(TrainCars, "🚃🚃🚃🚃🚃 ").is_none());
//! assert!(hitori::string::starts_with(TrainCars, " 🚃🚃🚃🚃🚃").is_none());
//! ```
//! *equivalent to
//! `^(?P<last_car>(?P<first_car>🚃))$|^(?P<first_car1>🚃)🚃{3}(?P<last_car1>🚃)$`
//!  in [regex] syntax*
//!
//! [regex]: https://docs.rs/regex

mod train_cars;

pub use train_cars::{TrainCars, TrainCarsCapture};

use super::include_example;
