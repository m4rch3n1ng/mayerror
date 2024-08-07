use mayerror::MayError;

#[derive(MayError)]
struct Error {
	#[location]
	location: &'static std::panic::Location<'static>,
	#[code]
	code: ErrorCode,
	#[backtrace]
	backtrace: mayerror::Backtrace,
}

#[derive(Debug, thiserror::Error)]
enum ErrorCode {
	#[error("error reading config")]
	ConfigError(#[source] ConfigErrorCode),
}

impl From<ConfigError> for Error {
	fn from(value: ConfigError) -> Self {
		Error {
			location: value.location,
			code: ErrorCode::ConfigError(value.code),
			backtrace: value.backtrace,
		}
	}
}

#[derive(MayError)]
struct ConfigError {
	#[location]
	location: &'static std::panic::Location<'static>,
	#[code]
	code: ConfigErrorCode,
	#[backtrace]
	backtrace: mayerror::Backtrace,
}

#[derive(Debug, thiserror::Error)]
enum ConfigErrorCode {
	#[error("file not found")]
	FileNotFound,
}

fn main() -> Result<(), Error> {
	let () = read_config()?;
	Ok(())
}

fn read_config() -> Result<(), ConfigError> {
	let () = Err(ConfigErrorCode::FileNotFound)?;
	Ok(())
}
