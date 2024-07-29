use crate::backtrace::{BacktraceOmitted, PrettyBacktrace, Verbosity, VERBOSITY};
use owo_colors::OwoColorize;
use std::panic::PanicHookInfo;

pub fn install() {
	std::panic::set_hook(Box::new(panic_hook));
}

fn panic_hook(info: &PanicHookInfo) {
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

	if *VERBOSITY >= Verbosity::Medium {
		let backtrace = ::backtrace::Backtrace::new();
		print!("\n\n{}", PrettyBacktrace(&backtrace));
	}

	println!("{}", BacktraceOmitted);
}
