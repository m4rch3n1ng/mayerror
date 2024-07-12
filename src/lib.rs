pub use mayerror_derive::*;

mod backtrace;
mod chain;

#[doc(hidden)]
pub mod __private {
	pub use super::backtrace::*;
	pub use super::chain::*;

	pub use owo_colors::OwoColorize;
}
