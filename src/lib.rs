pub use mayerror_derive::*;

#[cfg(feature = "backtrace")]
mod backtrace;
mod chain;

#[doc(hidden)]
pub mod __private {
	#[cfg(feature = "backtrace")]
	pub use super::backtrace::*;
	pub use super::chain::*;

	pub use owo_colors::OwoColorize;
}
