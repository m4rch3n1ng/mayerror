use once_cell::sync::Lazy;
use owo_colors::OwoColorize;
use std::{
	fmt::Display,
	fs::File,
	io::{BufRead, BufReader},
	path::PathBuf,
};

#[doc(hidden)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Verbosity {
	Minimal,
	Medium,
	Full,
}

impl Verbosity {
	fn from_env() -> Self {
		match std::env::var("RUST_LIB_BACKTRACE").or_else(|_| std::env::var("RUST_BACKTRACE")) {
			Ok(s) if s == "full" => Verbosity::Full,
			Ok(s) if s != "0" => Verbosity::Medium,
			_ => Verbosity::Minimal,
		}
	}
}

#[doc(hidden)]
pub static VERBOSITY: Lazy<Verbosity> = Lazy::new(Verbosity::from_env);

#[doc(hidden)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorBt {
	Show,
	Hide,
}

impl ColorBt {
	fn from_env() -> Self {
		match std::env::var("COLORBT_SHOW_HIDDEN") {
			Ok(s) if s != "0" => ColorBt::Show,
			_ => ColorBt::Hide,
		}
	}
}

#[doc(hidden)]
pub static COLOR_BT: Lazy<ColorBt> = Lazy::new(ColorBt::from_env);

#[doc(hidden)]
pub type Backtrace = backtrace::Backtrace;

#[doc(hidden)]
pub fn trace() -> self::Backtrace {
	if *VERBOSITY >= Verbosity::Medium {
		backtrace::Backtrace::new()
	} else {
		backtrace::Backtrace::new_unresolved()
	}
}

#[derive(Debug)]
struct Source<'a>(&'a Frame);

impl Display for Source<'_> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let Some((file, lineno)) = self.0.file.as_deref().zip(self.0.line) else {
			return Ok(());
		};

		let file = match File::open(file) {
			Ok(file) => file,
			Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
			e @ Err(_) => e.unwrap(),
		};

		// 2 lines at the start, since lines are 1-indexed
		let start = lineno.saturating_sub(3);

		let reader = BufReader::new(file);
		let lines = reader
			.lines()
			.map_while(Result::ok)
			.zip(1..)
			.skip(start as usize)
			.take(5);

		for (line, curr_lineno) in lines {
			if curr_lineno == lineno {
				write!(
					f,
					"\n{:>8} {} {}",
					curr_lineno.bold(),
					">".bold(),
					line.bold()
				)?;
			} else {
				write!(f, "\n{:>8} | {}", curr_lineno, line)?;
			}
		}

		Ok(())
	}
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

	/// taken from
	/// <https://github.com/eyre-rs/eyre/blob/dded7dededca017b23dde6126bd5596eddb2deca/color-eyre/src/config.rs#L284-L328>
	///
	/// licensed under MIT or APACHE 2.0
	fn is_dependency_code(&self) -> bool {
		const SYM_PREFIXES: &[&str] = &[
			"std::",
			"core::",
			"backtrace::backtrace::",
			"_rust_begin_unwind",
			"color_traceback::",
			"__rust_",
			"___rust_",
			"__pthread",
			"_main",
			"main",
			"__scrt_common_main_seh",
			"BaseThreadInitThunk",
			"_start",
			"__libc_start_main",
			"start_thread",
		];

		if let Some(name) = &self.name {
			if SYM_PREFIXES.iter().any(|x| name.starts_with(x)) {
				return true;
			}
		}

		const FILE_PREFIXES: &[&str] = &[
			"/rustc/",
			"src/libstd/",
			"src/libpanic_unwind/",
			"src/libtest/",
		];

		if let Some(file) = &self.file {
			let filename = file.to_string_lossy();
			if FILE_PREFIXES.iter().any(|x| filename.starts_with(x))
				|| filename.contains("/.cargo/registry/src/")
			{
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
		let (name, hash_suffix) = match name.len().checked_sub(19).map(|x| name.split_at(x)) {
			Some((name, hash_suffix)) if !name.is_empty() && hash_suffix.starts_with("::h") => {
				(name, hash_suffix)
			}
			_ => (name, "<unknown>"),
		};

		if self.is_dependency_code() {
			write!(f, "{}", name.green())?;
		} else {
			write!(f, "{}", name.red())?;
		}
		writeln!(f, "{}", hash_suffix)?;

		write!(f, "    at ")?;
		if let Some(file) = self.file.as_deref() {
			write!(f, "{}", file.display().purple())?;
		} else {
			write!(f, "{}", "<unknown source file>".purple())?;
		}
		if let Some(line) = self.line {
			write!(f, ":{}", line.purple())?;
		} else {
			write!(f, ":{}", "<unknown line number>".purple())?;
		}

		if *VERBOSITY >= Verbosity::Full {
			write!(f, "{}", Source(self))?;
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

fn filter_frames(frames: &mut Vec<Frame>) {
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
}

fn print_hidden(amt: usize, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
	let tmp = format!(
		"{decor} {amt} frame{plural} hidden {decor}",
		decor = "⋮",
		plural = if amt == 1 { "" } else { "s" }
	);

	write!(f, "{:^80}", tmp.cyan())
}

impl Display for PrettyBacktrace<'_> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{:━^80}", " BACKTRACE ")?;

		let mut frames = self.frames();
		let last_frame_n = frames.last().map(|frame| frame.n);

		if *COLOR_BT == ColorBt::Hide {
			filter_frames(&mut frames);
		}

		if frames.is_empty() {
			return writeln!(f, "<empty backtrace>");
		}

		let mut last_printed = 0;
		for frame in &frames {
			let delta = frame.n - last_printed;
			if delta > 1 {
				f.write_str("\n")?;
				print_hidden(delta - 1, f)?;
			}

			write!(f, "\n{}", frame)?;

			last_printed = frame.n;
		}

		let last_filtered = frames.last().unwrap();
		let last_frame_n = last_frame_n.unwrap();
		if last_filtered.n < last_frame_n {
			f.write_str("\n")?;
			print_hidden(last_frame_n - last_filtered.n, f)?;
		}

		Ok(())
	}
}

#[doc(hidden)]
pub struct BacktraceOmitted;

impl Display for BacktraceOmitted {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match (*VERBOSITY, *COLOR_BT) {
			(Verbosity::Full, ColorBt::Show) => {}
			(Verbosity::Full, ColorBt::Hide) => {
				f.write_str("\n\n")?;
				f.write_str(
				"Run with COLORBT_SHOW_HIDDEN=1 environment variable to disable frame filtering.",
			)?;
			}
			(Verbosity::Medium, ColorBt::Show) => {
				f.write_str("\n\n")?;
				f.write_str("Run with RUST_BACKTRACE=full to include source snippets.")?;
			}
			(Verbosity::Medium, ColorBt::Hide) => {
				f.write_str("\n\n")?;
				f.write_str(
				"Run with COLORBT_SHOW_HIDDEN=1 environment variable to disable frame filtering.\n",
			)?;
				f.write_str("Run with RUST_BACKTRACE=full to include source snippets.")?;
			}
			(Verbosity::Minimal, _) => {
				f.write_str("\n\n")?;
				f.write_str(
				"Backtrace omitted. Run with RUST_BACKTRACE=1 environment variable to display it.\n",
			)?;
				f.write_str("Run with RUST_BACKTRACE=full to include source snippets.")?;
			}
		}

		Ok(())
	}
}
