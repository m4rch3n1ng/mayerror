use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{spanned::Spanned, Data, DeriveInput, Index, Member, Type};

#[proc_macro_derive(MayError, attributes(code, location, backtrace))]
pub fn mayerror_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	let ast = match syn::parse::<DeriveInput>(input) {
		Ok(ast) => ast,
		Err(err) => return err.to_compile_error().into(),
	};

	let may_error = match Struct::from_syn(ast) {
		Ok(may_error) => may_error,
		Err(err) => return err.to_compile_error().into(),
	};

	let from = may_error.from();
	let display = may_error.display();
	let debug = may_error.debug();
	let error = may_error.error();

	quote! {
		#from
		#display
		#debug
		#error
	}
	.into()
}

struct Struct {
	fields: Fields,
	ident: syn::Ident,
}

impl Struct {
	fn init_loc(&self) -> Option<(TokenStream, TokenStream)> {
		if let Some(loc) = &self.fields.location {
			let body = quote! {
				let location = ::core::panic::Location::caller();
			};
			let init = quote! {
				#loc: ::core::convert::Into::into(location),
			};
			Some((body, init))
		} else {
			None
		}
	}

	#[cfg(feature = "backtrace")]
	fn init_backtrace(&self) -> Option<(TokenStream, TokenStream)> {
		if let Some(trace) = &self.fields.backtrace {
			let body = quote! {
				let backtrace = ::mayerror::__private::trace();
			};
			let init = quote! {
				#trace: backtrace,
			};
			Some((body, init))
		} else {
			None
		}
	}

	fn init(&self) -> TokenStream {
		let ident = &self.ident;
		let code = &self.fields.code;

		let (loc_body, loc_init) = self.init_loc().unzip();

		#[cfg(feature = "backtrace")]
		let (trace_body, trace_init) = self.init_backtrace().unzip();
		#[cfg(not(feature = "backtrace"))]
		let (trace_body, trace_init) = (quote! {}, quote! {});

		quote! {
			#loc_body
			#trace_body

			#ident {
				#code: ::core::convert::Into::into(value),
				#loc_init
				#trace_init
			}
		}
	}

	fn from(&self) -> TokenStream {
		let ident = &self.ident;
		let ty = &self.fields.code.ty;

		let init = self.init();

		quote! {
			impl<T> ::core::convert::From<T> for #ident
			where
				T: ::core::convert::Into<#ty>
			{
				#[track_caller]
				fn from(value: T) -> Self {
					#init
				}
			}
		}
	}

	fn display(&self) -> TokenStream {
		let ident = &self.ident;
		let cfield = &self.fields.code;

		let wr_display = if let Some(lfield) = &self.fields.location {
			quote! {
				::core::write!(f, "{} @ {}", self.#cfield, self.#lfield)?;
			}
		} else {
			quote! {
				::core::write!(f, "{}", self.#cfield)?;
			}
		};

		quote! {
			impl ::core::fmt::Display for #ident {
				fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
					#wr_display
					Ok(())
				}
			}
		}
	}

	fn debug(&self) -> TokenStream {
		let ident = &self.ident;
		let cfield = &self.fields.code;

		let error = quote! {
			::core::write!(f, "{}", ::mayerror::__private::OwoColorize::red(&self.#cfield))?;
		};

		let source = quote! {
			if let Some(source) = ::std::error::Error::source(&self.#cfield) {
				let chain = ::mayerror::__private::Chain::new(source);

				::core::write!(f, "\n\nSource:")?;
				for (idx, source) in chain.enumerate() {
					let source = ::mayerror::__private::OwoColorize::magenta(&source);
					::core::write!(f, "\n{:4}: {}", idx, source)?;
				}
			}
		};

		let location = if let Some(location) = &self.fields.location {
			quote! {
				::core::write!(f, "\n\nLocation:")?;
				::core::write!(f, "\n   {}", ::mayerror::__private::OwoColorize::cyan(&self.#location))?;
			}
		} else {
			quote! {}
		};

		#[cfg(feature = "backtrace")]
		let backtrace = if let Some(trace) = &self.fields.backtrace {
			quote! {
				if *::mayerror::__private::VERBOSITY >= ::mayerror::__private::Verbosity::Medium {
					let backtrace = ::mayerror::__private::PrettyBacktrace(&self.#trace);
					::core::write!(f, "\n\n{}", backtrace)?;
				}

				::core::write!(f, "{}", ::mayerror::__private::BacktraceOmitted)?;
			}
		} else {
			quote! {}
		};
		#[cfg(not(feature = "backtrace"))]
		let backtrace = quote! {};

		quote! {
			impl ::core::fmt::Debug for #ident {
				fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
					if f.alternate() {
						return ::core::fmt::Debug::fmt(&self.#cfield, f);
					}

					#error
					#source
					#location
					#backtrace

					Ok(())
				}
			}
		}
	}

	fn error(&self) -> TokenStream {
		let ident = &self.ident;
		let code = &self.fields.code;

		let source = quote! {
			fn source(&self) -> ::core::option::Option<&(dyn ::std::error::Error + 'static)> {
				::std::error::Error::source(&self.#code)
			}
		};

		quote! {
			impl ::std::error::Error for #ident {
				#source
			}
		}
	}
}

impl Struct {
	fn from_syn(ast: syn::DeriveInput) -> Result<Self, syn::Error> {
		let Data::Struct(data) = ast.data else {
			return Err(syn::Error::new_spanned(
				ast,
				"#[derive(MayError)] is only supported for structs",
			));
		};

		let ident = ast.ident;
		let fields = Fields::from_syn(data.fields)?;

		Ok(Struct { fields, ident })
	}
}

struct Fields {
	code: Field,
	location: Option<Field>,
	#[cfg(feature = "backtrace")]
	backtrace: Option<Field>,
}

struct Field {
	member: Member,
	ty: Type,
}

impl ToTokens for Field {
	fn to_tokens(&self, tokens: &mut TokenStream) {
		self.member.to_tokens(tokens);
	}
}

impl Field {
	fn from_syn(idx: usize, field: syn::Field) -> Field {
		let span = field.span();
		let member = field.ident.map(Member::Named).unwrap_or_else(|| {
			Member::Unnamed(Index {
				index: idx as u32,
				span,
			})
		});

		let ty = field.ty;
		Field { member, ty }
	}
}

impl Fields {
	fn from_syn(fields: syn::Fields) -> Result<Fields, syn::Error> {
		let mut location = None;
		let mut code = None;

		#[cfg(feature = "backtrace")]
		let mut backtrace = None;

		'outer: for (idx, field) in fields.into_iter().enumerate() {
			for attr in &field.attrs {
				let ident = attr.path();
				if ident.is_ident("code") {
					if code.is_some() {
						return Err(syn::Error::new_spanned(attr, "#[code] is already defined"));
					}

					let field = Field::from_syn(idx, field);
					code = Some(field);
					continue 'outer;
				} else if ident.is_ident("location") {
					if location.is_some() {
						return Err(syn::Error::new_spanned(
							attr,
							"#[location] is already defined",
						));
					}

					let field = Field::from_syn(idx, field);
					location = Some(field);

					continue 'outer;
				} else if ident.is_ident("backtrace") {
					#[cfg(not(feature = "backtrace"))]
					return Err(syn::Error::new_spanned(
						attr,
						"enable feature \"backtrace\" to use #[backtrace]",
					));

					#[cfg(feature = "backtrace")]
					{
						if backtrace.is_some() {
							return Err(syn::Error::new_spanned(
								attr,
								"#[backtrace] is already defined",
							));
						}

						let field = Field::from_syn(idx, field);
						backtrace = Some(field);
						continue 'outer;
					}
				}
			}

			return Err(syn::Error::new_spanned(
				field,
				"fields without attributes are not allowed",
			));
		}

		let Some(code) = code else {
			return Err(syn::Error::new(
				Span::call_site(),
				"error struct has to have a #[code] field",
			));
		};

		Ok(Fields {
			code,
			location,
			#[cfg(feature = "backtrace")]
			backtrace,
		})
	}
}
