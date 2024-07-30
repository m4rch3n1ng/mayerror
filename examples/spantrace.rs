use mayerror::MayError;
use tracing::instrument;
use tracing_error::{ErrorLayer, SpanTrace};
use tracing_subscriber::{prelude::*, registry::Registry};

#[derive(MayError)]
struct Error {
	#[code]
	code: ErrorCode,
	#[location]
	location: &'static std::panic::Location<'static>,
	#[spantrace]
	spantrace: SpanTrace,
}

#[derive(Debug, thiserror::Error)]
enum ErrorCode {
	#[error("main error")]
	Main,
}

#[instrument]
fn main() -> Result<(), Error> {
	Registry::default().with(ErrorLayer::default()).init();

	one()?;
	Ok(())
}

#[instrument]
fn one() -> Result<(), Error> {
	two()
}

#[instrument]
fn two() -> Result<(), Error> {
	let () = Err(ErrorCode::Main)?;
	Ok(())
}
