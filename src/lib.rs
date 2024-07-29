pub use self::install::install;
pub use mayerror_derive::*;

#[cfg(feature = "backtrace")]
mod backtrace;
mod chain;
mod install;

#[doc(hidden)]
pub mod __private {
	#[cfg(feature = "backtrace")]
	pub use super::backtrace::*;
	pub use super::chain::*;

	pub use owo_colors::OwoColorize;
}
