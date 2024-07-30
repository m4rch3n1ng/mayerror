use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{spanned::Spanned, Data, DeriveInput, Index, Member, Type};

#[proc_macro_derive(MayError, attributes(location, code))]
pub fn hello_macro_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	let ast = syn::parse::<DeriveInput>(input).expect("syn parsing failed");
	let may_error = Struct::from_syn(ast);

	let body = may_error.body();
	let from = may_error.from();
	let display = may_error.display();
	let debug = may_error.debug();
	let error = may_error.error();

	quote! {
		#body
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
	fn body(&self) -> TokenStream {
		let ty = &self.fields.code.ty;
		let field = &self.fields.code;
		let ident = &self.ident;

		quote! {
			impl #ident {
				fn code(&self) -> &#ty{
					&self.#field
				}
			}
		}
	}

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

	fn init(&self) -> TokenStream {
		let ident = &self.ident;
		let code = &self.fields.code;

		let (loc_body, loc_init) = self.init_loc().unzip();

		quote! {
			#loc_body

			#ident {
				#code: ::core::convert::Into::into(value),
				#loc_init
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
					::core::write!(f, "\n   {}: {}", idx, source)?;
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

		quote! {
			impl ::core::fmt::Debug for #ident {
				fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
					if f.alternate() {
						return ::core::fmt::Debug::fmt(&self.#cfield, f);
					}

					#error
					#source
					#location

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
	fn from_syn(ast: syn::DeriveInput) -> Self {
		let Data::Struct(data) = ast.data else {
			todo!()
		};

		let ident = ast.ident;
		let fields = Fields::from_syn(data.fields);

		Struct { fields, ident }
	}
}

struct Fields {
	code: Field,
	location: Option<Field>,
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
	fn from_syn(fields: syn::Fields) -> Fields {
		let mut location = None;
		let mut code = None;

		'outer: for (idx, field) in fields.into_iter().enumerate() {
			// let attrs = field.attrs
			for attr in &field.attrs {
				let ident = attr.path();
				if ident.is_ident("code") {
					assert!(code.is_none());

					let field = Field::from_syn(idx, field);
					code = Some(field);
					continue 'outer;
				} else if ident.is_ident("location") {
					assert!(location.is_none());

					let field = Field::from_syn(idx, field);
					location = Some(field);
					continue 'outer;
				}
			}

			todo!("error message");
		}

		let code = code.expect("error struct has to have a #[code]");

		Fields { code, location }
	}
}
