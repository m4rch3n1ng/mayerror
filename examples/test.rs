#![allow(dead_code)]

use mayerror::MayError;

#[derive(MayError)]
struct MayError {
	#[location]
	location: &'static std::panic::Location<'static>,
	#[code]
	thing: MayErrorCode,
	#[backtrace]
	traces: backtrace::Backtrace,
}

#[derive(Debug, thiserror::Error)]
enum MayErrorCode {
	#[error("source error")]
	Source(#[from] MayValError),
	#[error("unit error")]
	Unit,
}

#[derive(Debug, thiserror::Error)]
#[error("may val error")]
struct MayValError;

fn main() -> Result<(), MayError> {
	let () = Err(MayErrorCode::Source(MayValError))?;
	Ok(())
}
