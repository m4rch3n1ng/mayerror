pub use mayerror_derive::*;

#[doc(hidden)]
pub mod __private {
	#[doc(hidden)]
	pub struct Chain<'a> {
		state: Option<&'a (dyn std::error::Error + 'static)>,
	}

	impl<'a> Iterator for Chain<'a> {
		type Item = &'a (dyn std::error::Error + 'static);
		fn next(&mut self) -> Option<Self::Item> {
			if let Some(error) = self.state {
				self.state = error.source();
				Some(error)
			} else {
				None
			}
		}
	}

	impl<'a> Chain<'a> {
		#[doc(hidden)]
		pub fn new(head: &'a (dyn std::error::Error + 'static)) -> Self {
			Chain { state: Some(head) }
		}
	}

	pub use owo_colors::OwoColorize;
}
