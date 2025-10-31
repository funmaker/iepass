use quote::{quote, ToTokens};
use syn::parse_macro_input;

mod p8;
mod api;

#[proc_macro]
pub fn p8(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	let literal = parse_macro_input!(input as p8::P8Lit);
	literal.into_token_stream().into()
}

#[proc_macro_attribute]
pub fn api_callback(_attr: proc_macro::TokenStream, item: proc_macro::TokenStream) -> proc_macro::TokenStream {
	let function = parse_macro_input!(item as syn::ItemFn);
	let callback = api::make_callback(&function).unwrap_or_else(syn::Error::into_compile_error);
	
	quote!(
		#function
		#callback
	).into()
}
