use color_backtrace::BacktracePrinter;
use std::{fmt::Display, path::PathBuf};

#[doc(hidden)]
pub type Backtrace = backtrace::Backtrace;

#[doc(hidden)]
pub fn trace() -> self::Backtrace {
	backtrace::Backtrace::new()
}

#[derive(Debug)]
struct Frame {
	n: usize,
	name: Option<String>,
	line: Option<u32>,
	file: Option<PathBuf>,
}

impl Frame {
	fn is_mayerror_code(&self) -> bool {
		let Some(name) = self.name.as_deref() else {
			return false;
		};

		name.starts_with("mayerror")
	}

	/// taken from
	/// <https://github.com/eyre-rs/eyre/blob/dded7dededca017b23dde6126bd5596eddb2deca/color-eyre/src/config.rs#L360-L382>
	///
	/// licensed under MIT or APACHE 2.0
	fn is_runtime_init_code(&self) -> bool {
		const SYM_PREFIXES: &[&str] = &[
			"std::rt::lang_start::",
			"test::run_test::run_test_inner::",
			"std::sys_common::backtrace::__rust_begin_short_backtrace",
			"std::sys::backtrace::__rust_begin_short_backtrace",
		];

		let Some(name) = self.name.as_deref() else {
			return false;
		};

		if SYM_PREFIXES.iter().any(|x| name.starts_with(x)) {
			return true;
		}

		if let Some(file) = self.file.as_deref() {
			let file = file.to_string_lossy();

			// for linux, this is the best rule for skipping test init i found
			if name == "{{closure}}" && file == "src/libtest/lib.rs" {
				return true;
			}
		}

		false
	}
}

#[doc(hidden)]
pub struct PrettyBacktrace<'a>(pub &'a backtrace::Backtrace);

impl<'a> PrettyBacktrace<'a> {
	fn frames(&self) -> Vec<Frame> {
		let mut frames = self
			.0
			.frames()
			.iter()
			.flat_map(|frame| frame.symbols().iter().map(|sym| (frame.ip(), sym)))
			.enumerate()
			.map(|(n, (_ip, sym))| Frame {
				n,
				name: sym.name().map(|name| name.to_string()),
				line: sym.lineno(),
				file: sym.filename().map(ToOwned::to_owned),
			})
			.collect::<Vec<_>>();

		let mayerror_cutoff = frames
			.iter()
			.rposition(|frame| frame.is_mayerror_code())
			.map(|idx| idx + 1)
			.unwrap_or(0);

		let runtime_init_cutoff = frames
			.iter()
			.position(|frame| frame.is_runtime_init_code())
			.unwrap_or(usize::MAX);

		let range = mayerror_cutoff..runtime_init_cutoff;
		frames.retain(|frame| range.contains(&frame.n));

		frames
	}
}

impl Display for PrettyBacktrace<'_> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let printer = BacktracePrinter::new();
		let str = printer.format_trace_to_string(self.0).unwrap();
		write!(f, "{}", str)?;

		let frames = self.frames();
		write!(f, "{:?}", frames)?;

		Ok(())
	}
}
