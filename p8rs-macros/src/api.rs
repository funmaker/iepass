use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{Result, FnArg, Error, Type, PatType, Index, ReturnType, GenericParam, Ident, PathArguments, GenericArgument};
use syn::spanned::Spanned;

pub fn make_callback(func: &syn::ItemFn) -> Result<TokenStream> {
	let name = &func.sig.ident;
	let vis = &func.vis;
	
	let mut free_count = 0;
	let mut free_ty = TokenStream::new();
	let mut fn_args = TokenStream::new();
	let mut uses_stack = false;
	
	for arg in func.sig.inputs.iter() {
		match ArgKind::classify(arg)? {
			ArgKind::Context => fn_args.extend(quote!( ctx, )),
			ArgKind::Execution => fn_args.extend(quote!( exec.reborrow(), )),
			ArgKind::RuntimeRef => fn_args.extend(quote!( rt.reborrow(), )),
			ArgKind::RuntimeDowncast => fn_args.extend(quote!( rt.downcast(), )),
			ArgKind::Stack => {
				uses_stack = true;
				fn_args.extend(quote!( stack.reborrow(), ))
			},
			ArgKind::Free(arg) => {
				let ty = &arg.ty;
				let idx = Index::from(free_count);
				free_ty.extend(quote!( #ty, ));
				fn_args.extend(quote!( args.#idx, ));
				
				free_count += 1;
			}
		}
	}
	
	let mut args = None;
	if free_count > 0 {
		args = Some(quote! {
			let Ok(args) = stack.consume::<(#free_ty)>(ctx) else { return Ok(CallbackReturn::Return) };
		});
	} else if !uses_stack {
		args = Some(quote! {
			stack.clear();
		});
	}
	
	let mut ret_try = None;
	let mut write_stack = None;
	let mut ret_value = quote! {
		Ok(CallbackReturn::Return)
	};
	
	if let ReturnType::Type(_, ty) = &func.sig.output {
		write_stack = Some(quote! {
			stack.replace(ctx, ret);
		});
		
		if let Type::Path(path) = &**ty
		&& let Some(mut seg) = path.path.segments.last() {
			if seg.ident == "Result" {
				ret_try = Some(quote! {
					let ret = ret?;
				});
				
				if let PathArguments::AngleBracketed(args) = &seg.arguments
				&& let Some(GenericArgument::Type(Type::Path(path))) = args.args.first()
				&& let Some(inner) = path.path.segments.last() {
					seg = inner;
				}
			}
			
			if seg.ident == "CallbackReturn" {
				write_stack = None;
				ret_value = quote! {
					Ok(ret)
				};
			}
		}
	}
	
	let generics: TokenStream =
		func.sig.generics.params.iter()
		                        .filter(|param| !matches!(param, GenericParam::Lifetime(_)))
		                        .map(|param| param.into_token_stream())
		                        .collect();
	
	let generics_args: TokenStream =
		func.sig.generics.params.iter()
		                        .flat_map(generic_to_ident)
		                        .map(|ident| quote!( #ident, ))
		                        .collect();
	
	let where_clause = &func.sig.generics.where_clause;
	
	Ok(quote! {
		#vis mod #name {
			use super::*;
			use ::p8rs_piccolo::{Callback, Context, Execution, Stack, RuntimeRef, CallbackReturn, IntoValue};
			use ::alloc::format;
			
			pub fn callback<'gc, #generics>(ctx: Context<'gc>) -> Callback<'gc> #where_clause {
				Callback::from_fn(&ctx, move |ctx, mut exec, mut stack, rt| {
					#args
					let ret = #name::<#generics_args>(#fn_args);
					#ret_try
					#write_stack
					#ret_value
				})
			}
		}
	})
}

fn generic_to_ident(param: &GenericParam) -> Option<&Ident> {
	match param {
		GenericParam::Lifetime(_) => None,
		GenericParam::Type(ty) => Some(&ty.ident),
		GenericParam::Const(cst) => Some(&cst.ident),
	}
}

enum ArgKind<'a> {
	Context,
	Execution,
	Stack,
	RuntimeRef,
	RuntimeDowncast,
	Free(&'a PatType),
}

impl<'a> ArgKind<'a> {
	fn classify(arg: &'a FnArg) -> Result<Self> {
		let arg = match arg {
			FnArg::Receiver(arg) => return Err(Error::new(arg.span(), "`self` argument is not allowed")),
			FnArg::Typed(arg) => arg,
		};
		
		if let Type::Path(path) = &*arg.ty {
			match path.path.segments.last().map(|seg| seg.ident.to_string()).as_deref() {
				Some("Context") => return Ok(ArgKind::Context),
				Some("Execution") => return Ok(ArgKind::Execution),
				Some("Stack") => return Ok(ArgKind::Stack),
				Some("RuntimeRef") => return Ok(ArgKind::RuntimeRef),
				Some("Runtime") => return Err(Error::new(path.span(), "Runtime argument must be taken as a reference")),
				_ => {},
			}
		} else if let Type::Reference(rf) = &*arg.ty {
			if let Type::Path(path) = &*rf.elem {
				if path.path.segments.last().is_some_and(|segment| segment.ident == "Runtime") {
					return Ok(ArgKind::RuntimeDowncast);
				}
			}
		}
		
		Ok(ArgKind::Free(arg))
	}
}
