#![allow(dead_code)]

use mayerror::MayError;

#[derive(MayError)]
struct Error {
	#[location]
	location: &'static std::panic::Location<'static>,
	#[code]
	code: ErrorCode,
	#[backtrace]
	backtrace: backtrace::Backtrace,
}

#[derive(Debug, thiserror::Error)]
enum ErrorCode {
	#[error("source error")]
	Source(#[from] MayValError),
	#[error("unit error")]
	Unit,
}

#[derive(Debug, thiserror::Error)]
#[error("may val error")]
struct MayValError;

fn main() -> Result<(), Error> {
	one()?;
	Ok(())
}

fn one() -> Result<(), Error> {
	let () = Err(ErrorCode::Source(MayValError))?;
	Ok(())
}
