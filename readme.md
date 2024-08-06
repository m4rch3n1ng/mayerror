
# mayerror

a convenient way to create an error struct with a known error code enum and a pretty representation.

as of right now you still need [thiserror](https://github.com/dtolnay/thiserror) to create the actual error code enum.

## usage

to use it you have to first create an enum to use as an error code,
for example with [thiserror](https://github.com/dtolnay/thiserror).

```rs
#[derive(Debug, thiserror::Error)]
pub enum ErrorCode {
    #[error("io error")]
    Io(#[from] std::io::Error),
    #[error("config file empty")]
    EmptyFile,
}
```

then you can use that error code in a `MayError` struct with the `#[code]` attribute

```rs
use mayerror::MayError;

#[derive(MayError)]
pub struct Error {
    #[code]
    code: ErrorCode,
}
```

if you want to have more context for the error, you can add a `#[location]` and even a `#[backtrace]` (see [#backtrace](#backtrace) for more info)

```rs
use mayerror::MayError;

#[derive(MayError)]
pub struct Error {
    #[code]
    code: ErrorCode,
    #[location]
    location: &'static std::panic::Location<'static>,
    #[backtrace]
    backtrace: backtrace::Backtrace,
}
```

to use the error you have to create an error code and then use the `?` operator to convert it into a proper error,
or you can directly convert any error that you can convert into the error code directly into the error.

```rs
struct Word(String);

impl Word {
    fn read() -> Result<Word, Error> {
        // read_to_string returns an `io::Error` which can be converted into an `ErrorCode`,
        // which means it can be converted into an `Error`
        let content = std::fs::read_to_string("file.txt")?;

        let word = content.split_whitespace().next();
        // explicity created `ErrorCode` errors can be directly converted into an `Error`
        let word = word.filter(|s| !s.is_empty()).ok_or(ErrorCode::EmptyFile)?;
        let word = Word(word.to_owned());

        Ok(word)
    }
}
```

you can see a full example in [usage.rs](./examples/usage.rs)

### backtrace

the `#[backtrace]` you also need to add the [backtrace](https://github.com/rust-lang/backtrace-rs) crate.

```toml
[dependencies]
backtrace = "0.3"
```

and then use `backtrace::Backtrace` in your `MayError` struct.
