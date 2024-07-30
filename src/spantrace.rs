use std::fmt::Display;
use tracing_error::{SpanTrace, SpanTraceStatus};

#[doc(hidden)]
pub struct PrettySpanTrace<'a>(pub &'a SpanTrace);

impl Display for PrettySpanTrace<'_> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		if self.0.status() == SpanTraceStatus::CAPTURED {
			write!(f, "\n\n{}", color_spantrace::colorize(self.0))
		} else {
			Ok(())
		}
	}
}

#[doc(hidden)]
pub fn spantrace() -> SpanTrace {
	SpanTrace::capture()
}
