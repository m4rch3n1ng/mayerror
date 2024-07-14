use proc_macro2::TokenStream;
use quote::quote;
use syn::{spanned::Spanned, Data, DeriveInput, Index, Member, Type};

#[proc_macro_derive(MayError, attributes(location, code))]
pub fn hello_macro_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	let ast = syn::parse::<DeriveInput>(input).unwrap();
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
		let member = &self.fields.code.member;
		let ident = &self.ident;

		quote! {
			impl #ident {
				fn code(&self) -> &#ty{
					&self.#member
				}
			}
		}
	}

	fn init(&self) -> TokenStream {
		let ident = &self.ident;
		let code = &self.fields.code.member;

		match &self.fields.location {
			None => {
				quote! {
					#ident {
						#code: ::core::convert::Into::into(value),
					}
				}
			}
			Some(ref loc) => {
				let loc = &loc.member;
				quote! {
					let location = ::core::panic::Location::caller();
					#ident {
						#code: ::core::convert::Into::into(value),
						#loc: ::core::convert::Into::into(location),
					}
				}
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
		let cfield = &self.fields.code.member;

		let wr_display = if let Some(location) = &self.fields.location {
			let lfield = &location.member;
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
		let cfield = &self.fields.code.member;

		let wr_debug = if let Some(location) = &self.fields.location {
			let lfield = &location.member;

			quote! {
				let code = ::mayerror::__private::OwoColorize::red(&self.#cfield);
				let location = ::mayerror::__private::OwoColorize::cyan(&self.#lfield);
				::core::write!(f, "{} @ {}", code, location)?;
			}
		} else {
			quote! {
				let code = ::mayerror::__private::OwoColorize::red(&self.#cfield);
				::core::write!(f, "{}", code)?;
			}
		};

		let source = quote! {
			if let Some(source) = ::std::error::Error::source(&self.#cfield) {
				let chain = ::mayerror::__private::Chain::new(source);

				::core::writeln!(f, "\n\nSource:")?;
				for (idx, source) in chain.enumerate() {
					let source = ::mayerror::__private::OwoColorize::magenta(&source);
					::core::writeln!(f, "   {}: {}", idx, source)?;
				}
			}
		};

		quote! {
			impl ::core::fmt::Debug for #ident {
				fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
					if f.alternate() {
						return ::core::fmt::Debug::fmt(&self.#cfield, f);
					}

					#wr_debug

					#source

					Ok(())
				}
			}
		}
	}

	fn error(&self) -> TokenStream {
		let ident = &self.ident;
		let code = &self.fields.code.member;

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

impl Field {
	fn from_syn(idx: usize, field: syn::Field) -> Field {
		let span = field.span();
		let member = field.ident.map(Member::Named).unwrap_or_else(|| {
			Member::Unnamed(Index {
				index: idx as u32,
				span,
			})
		});

		let r#type = field.ty;

		Field { member, ty: r#type }
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

		let code = code.unwrap();

		Fields { code, location }
	}
}
