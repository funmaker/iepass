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
	let bits_type = &field.ty;
	
	Ok(quote!{
		impl #ident {
			fn from_bits_ref(inner: &#bits_type) -> &Self {
				unsafe { ::core::mem::transmute(inner) }
			}
			fn from_bits_mut(inner: &mut #bits_type) -> &mut Self {
				unsafe { ::core::mem::transmute(inner) }
			}
			fn from_bits_boxed<A: ::core::alloc::Allocator>(inner: Box<#bits_type, A>) -> Box<Self, A> {
				let (ptr, alloc) = Box::into_raw_with_allocator(inner);
				unsafe { Box::from_raw_in(ptr as *mut Self, alloc) }
			}
			fn to_bits_ref(&self) -> &#bits_type {
				unsafe { ::core::mem::transmute(self) }
			}
			fn to_bits_mut(&mut self) -> &mut #bits_type {
				unsafe { ::core::mem::transmute(self) }
			}
			fn to_bits_boxed<A: ::core::alloc::Allocator>(self: Box<Self, A>) -> Box<#bits_type, A> {
				let (ptr, alloc) = Box::into_raw_with_allocator(self);
				unsafe { Box::from_raw_in(ptr as *mut #bits_type, alloc) }
			}
		}
	})
}
