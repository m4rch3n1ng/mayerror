#[cfg(feature = "backtrace")]
use crate::backtrace::{BacktraceOmitted, PrettyBacktrace, Verbosity, VERBOSITY};
use owo_colors::OwoColorize;
use std::panic::PanicHookInfo;

/// installs the `mayerror` panic hook.
///
/// a standalone version of the hook can be found under [`mayerror::panic_hook`].
///
/// ```
/// mayerror::install();
/// ```
///
/// [`mayerror::panic_hook`]: crate::panic_hook
///
pub fn install() {
	std::panic::set_hook(Box::new(panic_hook));
}

/// the `mayerror` panic hook.
///
/// can be installed via [`mayerror::install`].
///
/// ```
/// use mayerror::panic_hook;
///
/// std::panic::set_hook(Box::new(panic_hook));
/// ```
///
/// [`mayerror::install`]: crate::install()
pub fn panic_hook(info: &PanicHookInfo) {
	let payload = info.payload();
	let payload = if let Some(&s) = payload.downcast_ref::<&str>() {
		s
	} else if let Some(s) = payload.downcast_ref::<String>() {
		s.as_ref()
	} else {
		"<non string panic payload>"
	};

	println!("{}", "The application panicked.".red());
	println!("Message: {}", payload.cyan());

	if let Some(location) = info.location() {
		print!("Location: {}", location.magenta());
	} else {
		print!("Location: {}", "<unknown>".magenta());
	}

	#[cfg(feature = "backtrace")]
	if *VERBOSITY >= Verbosity::Medium {
		let backtrace = ::backtrace::Backtrace::new();
		print!("\n\n{}", PrettyBacktrace(&backtrace));
	}
	#[cfg(not(feature = "backtrace"))]
	println!();

	#[cfg(feature = "backtrace")]
	println!("{}", BacktraceOmitted);
}
