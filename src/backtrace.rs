use owo_colors::OwoColorize;
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

impl Display for Frame {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{:>2}: ", self.n)?;

		let name = self.name.as_deref().unwrap_or("<unknown>");
		writeln!(f, "{}", name)?;

		write!(f, "    at ")?;
		if let Some(file) = self.file.as_deref() {
			write!(f, "{}", file.display())?;
		} else {
			write!(f, "<unknown source file>")?;
		}
		if let Some(line) = self.line {
			writeln!(f, ":{}", line)?;
		} else {
			writeln!(f, ":<unknown line number>")?;
		}

		Ok(())
	}
}

#[doc(hidden)]
pub struct PrettyBacktrace<'a>(pub &'a backtrace::Backtrace);

impl PrettyBacktrace<'_> {
	fn frames(&self) -> Vec<Frame> {
		let frames = self
			.0
			.frames()
			.iter()
			.flat_map(|frame| frame.symbols().iter().map(|sym| (frame.ip(), sym)))
			.zip(1..)
			.map(|((_ip, sym), n)| Frame {
				n,
				name: sym.name().map(|name| name.to_string()),
				line: sym.lineno(),
				file: sym.filename().map(ToOwned::to_owned),
			})
			.collect::<Vec<_>>();

		frames
	}
}

fn filter_frames(mut frames: Vec<Frame>) -> Vec<Frame> {
	let mayerror_cutoff = frames
		.iter()
		.rposition(|frame| frame.is_mayerror_code())
		.map(|idx| idx + 2) // frames are 1-indexed
		.unwrap_or(0);

	let runtime_init_cutoff = frames
		.iter()
		.position(|frame| frame.is_runtime_init_code())
		.map(|idx| idx + 1) // frames are 1-indexed
		.unwrap_or(usize::MAX);

	let range = mayerror_cutoff..runtime_init_cutoff;
	frames.retain(|frame| range.contains(&frame.n));

	frames
}

fn print_hidden(amt: usize, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
	let tmp = format!(
		"{decor} {amt} frame{plural} hidden {decor}",
		decor = "⋮",
		plural = if amt == 1 { "" } else { "s" }
	);

	writeln!(f, "{:^80}", tmp.cyan())
}

impl Display for PrettyBacktrace<'_> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		writeln!(f, "{:━^80}", " BACKTRACE ")?;

		let frames = self.frames();
		let last_frame_n = frames.last().map(|frame| frame.n);

		let filtered_frames = filter_frames(frames);

		if filtered_frames.is_empty() {
			return writeln!(f, "<empty backtrace>");
		}

		let mut last_printed = 0;
		for frame in &filtered_frames {
			let delta = frame.n - last_printed;
			if delta > 1 {
				print_hidden(delta - 1, f)?;
			}

			write!(f, "{}", frame)?;

			last_printed = frame.n;
		}

		let last_filtered = filtered_frames.last().unwrap();
		let last_frame_n = last_frame_n.unwrap();
		if last_filtered.n < last_frame_n {
			print_hidden(last_frame_n - last_filtered.n, f)?;
		}

		Ok(())
	}
}
