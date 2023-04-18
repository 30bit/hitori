//! More complex examples of using [hitori]
//!
//! ### Email
//!
//! ```
#![doc = include_example!("putting_everything_together/email")]
//!
//! let s = "user@example.com";
//! let matched = hitori::string::starts_with(Email, s).unwrap();
//! assert_eq!(&s[matched.capture.user.unwrap()], "user");
//! assert_eq!(&s[matched.capture.domain_with_extension.unwrap()], "example.com");
//! //assert_eq!(&s[matched.capture.domain_extension.unwrap()], "com");
//! ```
//! *equivalent to `[\w\.+-]+@[\w\.-]+\.[\w\.-]+` in [regex] syntax*
//!
//! ### Uri
//!
//! ```
#![doc = include_example!("putting_everything_together/uri")]
//!
//! let s = "postgres://user@localhost:5432/my_db";
//! let matched = hitori::string::starts_with(Uri, s).unwrap();
//! assert_eq!(&s[matched.capture.schema.unwrap()], "postgres");
//! assert_eq!(&s[matched.capture.path.unwrap()], "user@localhost:5432/my_db");
//! assert!(matched.capture.query.is_none());
//! assert!(matched.capture.fragment.is_none());
//! ```
//! *equivalent to `[\w]+://[^/\s?#][^\s?#]+(?:\?[^\s#]*)?(?:#[^\s]*)?`
//! in [regex] syntax*
//!
//! ### IpV4
//!
//! ```
#![doc = include_example!("putting_everything_together/ipv4")]
//!
//! assert!(hitori::string::starts_with(IpV4, "255.240.111.255").is_some());
//! assert!(hitori::string::starts_with(IpV4, "66.249.64.13").is_some());
//! assert!(hitori::string::starts_with(IpV4, "216.58.214.14").is_some());
//! assert!(hitori::string::starts_with(IpV4, "255.256.111.255").is_none());
//! ```
//! *equivalent to
//! `(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9])\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9])`
//! in [regex] syntax*
//!
//! [hitori]: https://docs.rs/hitori
//! [regex]: https://docs.rs/regex

mod email;
mod ipv4;
mod uri;

pub use email::{Email, EmailCapture};
pub use ipv4::{IpV4, IpV4Capture};
pub use uri::{Uri, UriCapture};

use super::include_example;
