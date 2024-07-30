//! to use it you have to first create an enum to use as an error code,
//! for example with [thiserror](https://github.com/dtolnay/thiserror).
//!
//! ```
//! #[derive(Debug, thiserror::Error)]
//! pub enum ErrorCode {
//!     #[error("io error")]
//!     Io(#[from] std::io::Error),
//!     #[error("config file empty")]
//!     EmptyFile,
//! }
//! ```
//!
//! then you can use that error code in a `MayError` struct with the `#[code]` attribute
//!
//! ```
//! use mayerror::MayError;
//!
//! # #[derive(Debug, thiserror::Error)]
//! # pub enum ErrorCode {
//! #     #[error("io error")]
//! #     Io(#[from] std::io::Error),
//! #     #[error("config file empty")]
//! #     EmptyFile,
//! # }
//! #
//! #[derive(MayError)]
//! pub struct Error {
//!     #[code]
//!     code: ErrorCode,
//! }
//! ```
//!
//! if you want to have more context for the error, you can add a `#[location]` and even a `#[backtrace]`
//!
//! ```
//! use mayerror::MayError;
//!
//! # #[derive(Debug, thiserror::Error)]
//! # pub enum ErrorCode {
//! #     #[error("io error")]
//! #     Io(#[from] std::io::Error),
//! #     #[error("config file empty")]
//! #     EmptyFile,
//! # }
//! #
//! #[derive(MayError)]
//! pub struct Error {
//!     #[code]
//!     code: ErrorCode,
//!     #[location]
//!     location: &'static std::panic::Location<'static>,
//!     #[backtrace]
//!     backtrace: backtrace::Backtrace,
//! }
//! ```
//!
//! to use the error you have to create an error code and then use the `?` operator to convert it into a proper error,
//! or you can directly convert any error that you can convert into the error code directly into the error.
//!
//! ```
//! # use mayerror::MayError;
//! #
//! # #[derive(Debug, thiserror::Error)]
//! # pub enum ErrorCode {
//! #     #[error("io error")]
//! #     Io(#[from] std::io::Error),
//! #     #[error("config file empty")]
//! #     EmptyFile,
//! # }
//! #
//! # #[derive(MayError)]
//! # pub struct Error {
//! #     #[code]
//! #     code: ErrorCode,
//! #     #[location]
//! #     location: &'static std::panic::Location<'static>,
//! #     #[backtrace]
//! #     backtrace: backtrace::Backtrace,
//! # }
//! #
//! struct Word(String);
//!
//! impl Word {
//!     fn read() -> Result<Word, Error> {
//!         // read_to_string returns an `io::Error` which can be converted into an `ErrorCode`,
//!         // which means it can be converted into an `Error`
//!         let content = std::fs::read_to_string("file.txt")?;
//!
//!         let word = content.split_whitespace().next();
//!         // explicity created `ErrorCode` errors can be directly converted into an `Error`
//!         let word = word.filter(|s| !s.is_empty()).ok_or(ErrorCode::EmptyFile)?;
//!         let word = Word(word.to_owned());
//!
//!         Ok(word)
//!     }
//! }
//! ```

pub use self::install::install;
pub use mayerror_derive::*;

#[cfg(feature = "backtrace")]
mod backtrace;
mod chain;
mod install;
mod spantrace;

#[doc(hidden)]
pub mod __private {
	#[cfg(feature = "backtrace")]
	pub use super::backtrace::*;
	pub use super::chain::*;
	pub use super::spantrace::*;

	pub use owo_colors::OwoColorize;
}
