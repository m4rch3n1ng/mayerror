use mayerror::MayError;
use std::{
	num::ParseIntError,
	path::{Path, PathBuf},
};

#[derive(MayError)]
struct Error {
	#[code]
	code: ErrorCode,
	#[location]
	location: &'static std::panic::Location<'static>,
	#[backtrace]
	backtrace: backtrace::Backtrace,
}

#[derive(Debug, thiserror::Error)]
enum ErrorCode {
	#[error("file {0:?} not found")]
	FileNotFound(PathBuf),
	#[error("io error")]
	Io(#[from] std::io::Error),
	#[error("couldn't parse content")]
	ParseError(#[from] ParseIntError),
}

pub struct Parser(String);

impl Parser {
	fn read<P: AsRef<Path>>(path: P) -> Result<Parser, Error> {
		let content = std::fs::read_to_string(&path).map_err(|err| {
			if err.kind() == std::io::ErrorKind::NotFound {
				ErrorCode::FileNotFound(path.as_ref().to_owned())
			} else {
				ErrorCode::Io(err)
			}
		})?;

		let parser = Parser(content);
		Ok(parser)
	}

	fn parse(&self) -> Result<u32, Error> {
		let parsed = self.0.parse::<u32>()?;
		Ok(parsed)
	}
}

fn main() -> Result<(), Error> {
	let parser = Parser::read("file.txt")?;
	let content = parser.parse()?;
	println!("content is {}", content);

	Ok(())
}
