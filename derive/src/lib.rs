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

	quote! {
		#body
		#from
		#display
		#debug
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

	fn from(&self) -> TokenStream {
		let ident = &self.ident;
		let ty = &self.fields.code.ty;

		let func = match &self.fields.location {
			None => {
				let field = &self.fields.code.member;

				quote! {
					#ident {
						#field: value.into(),
					}
				}
			}
			Some(loc) => {
				let f1 = &self.fields.code.member;
				let f2 = &loc.member;

				quote! {
					let location = ::std::panic::Location::caller();

					#ident {
						#f1: value.into(),
						#f2: location.into(),
					}
				}
			}
		};

		quote! {
			impl<T> ::std::convert::From<T> for #ident
			where
				T: ::std::convert::Into<#ty>
			{
				#[track_caller]
				fn from(value: T) -> Self {
					#func
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
				::std::write!(f, "{} @ {}", self.#cfield, self.#lfield)?;
			}
		} else {
			quote! {
				::std::write(f, "{}", self.#cfield)?;
			}
		};

		quote! {
			impl ::std::fmt::Display for #ident {
				fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
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
				let code = ::owo_colors::OwoColorize::red(&self.#cfield);
				let location = ::owo_colors::OwoColorize::cyan(&self.#lfield);
				::std::write!(f, "{} @ {}", code, location)?;
			}
		} else {
			quote! {
				let code = ::owo_colors::OwoColorize::red(&self.#cfield);
				::std::write!(f, "{}", code)?;
			}
		};

		let source = quote! {
			if let Some(source) = ::std::error::Error::source(&self.#cfield) {
				let chain = Chain::new(source);

				::std::writeln!(f, "\n\nSource:")?;
				for (idx, source) in chain.enumerate() {
					let source = ::owo_colors::OwoColorize::magenta(&source);
					::std::writeln!(f, "   {}: {}", idx, source)?;
				}
			}
		};

		// todo
		let chain = quote! {
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

			struct Chain<'a> {
				state: Option<&'a (dyn std::error::Error + 'static)>,
			}

			impl<'a> Chain<'a> {
				fn new(head: &'a (dyn std::error::Error + 'static)) -> Self {
					Chain { state: Some(head) }
				}
			}
		};

		quote! {
			impl ::std::fmt::Debug for #ident {
				fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
					if f.alternate() {
						return ::std::fmt::Debug::fmt(&self.#cfield, f);
					}

					extern crate owo_colors;

					#wr_debug

					#source

					Ok(())
				}
			}

			#chain
		}
	}
}

impl Struct {
	fn from_syn(ast: syn::DeriveInput) -> Self {
		let data = match ast.data {
			Data::Struct(data) => data,
			_ => todo!(),
		};

		let ident = ast.ident;
		let fields = Fields::from_syn(data.fields);

		Struct { fields, ident }
	}
}

struct Fields {
	location: Option<Field>,
	code: Field,
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
				if ident.is_ident("location") {
					assert!(location.is_none());

					let field = Field::from_syn(idx, field);
					location = Some(field);
					continue 'outer;
				} else if ident.is_ident("code") {
					assert!(code.is_none());

					let field = Field::from_syn(idx, field);
					code = Some(field);
					continue 'outer;
				}
			}

			todo!("error message");
		}

		let code = code.unwrap();

		Fields { location, code }
	}
}
