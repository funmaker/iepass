use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, Fields};

pub fn make_derive(item: &syn::DeriveInput) -> syn::Result<TokenStream> {
	let mut is_transparent = false;
	for attr in &item.attrs {
		if attr.path().is_ident("repr") {
			attr.parse_nested_meta(|meta| {
				if meta.path.is_ident("transparent") {
					is_transparent = true;
					Ok(())
				} else {
					Err(meta.error("expected `#[repr(transparent)]`"))
				}
			})?;
		}
	}
	
	if !is_transparent {
		return Err(syn::Error::new(item.ident.span(), "Item must be `#[repr(transparent)]`"));
	}
	
	let item_struct = match &item.data {
		Data::Struct(item_struct) => item_struct,
		_ => return Err(syn::Error::new(item.ident.span(), "Item must be a struct")),
	};
	
	let field = match &item_struct.fields {
		Fields::Named(fields) if fields.named.len() == 1 => &fields.named[0],
		Fields::Unnamed(fields) if fields.unnamed.len() == 1 => &fields.unnamed[0],
		_ => return Err(syn::Error::new(item.ident.span(), "Struct must have single field"))
	};
	
	let ident = &item.ident;
	let field_type = &field.ty;
	
	Ok(quote!{
		impl #ident {
			fn from_ref(inner: &#field_type) -> &Self {
				unsafe { ::core::mem::transmute(inner) }
			}
			fn from_mut(inner: &mut #field_type) -> &mut Self {
				unsafe { ::core::mem::transmute(inner) }
			}
		}
	})
}
