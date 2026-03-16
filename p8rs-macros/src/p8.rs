use std::str::FromStr;
use quote::{quote, ToTokens, TokenStreamExt};
use proc_macro2::{Literal, Span, TokenStream};
use syn::{Error, LitChar, LitFloat, LitInt, LitStr};
use syn::parse::{Parse, ParseStream};
use p8rs_types::p8num::P8Num;
use p8rs_types::p8scii;

#[derive(Clone)]
pub enum P8Lit {
	Str(LitStr),
	Char(LitChar),
	Int(LitInt),
	Float(LitFloat),
}

impl Parse for P8Lit {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		let lookahead = input.lookahead1();
		if lookahead.peek(LitStr) {
			input.parse().map(P8Lit::Str)
		} else if lookahead.peek(LitChar) {
			input.parse().map(P8Lit::Char)
		} else if lookahead.peek(LitInt) {
			input.parse().map(P8Lit::Int)
		} else if lookahead.peek(LitFloat) {
			input.parse().map(P8Lit::Float)
		} else {
			Err(lookahead.error())
		}
	}
}

impl ToTokens for P8Lit {
	fn to_tokens(&self, tokens: &mut TokenStream) {
		let token = match self {
			P8Lit::Float(val) => p8_number_lit(val.base10_digits(), val.span(), 10),
			P8Lit::Str(val) if val.suffix() == "hex" => p8_number_lit(trim_prefix(&val.value(), "0x"), val.span(), 16),
			P8Lit::Str(val) if val.suffix() == "bin" => p8_number_lit(trim_prefix(&val.value(), "0b"), val.span(), 2),
			P8Lit::Str(val) if val.suffix() == "dec" => p8_number_lit(&val.value(), val.span(), 2),
			P8Lit::Str(val) => p8_string_lit(val),
			P8Lit::Char(val) => p8_char_lit(val),
			P8Lit::Int(val) => p8_integer_lit(val),
		};
		
		tokens.append_all(token);
	}
}

fn p8_string_lit(lit: &LitStr) -> TokenStream {
	let string = lit.value();
	match p8scii::from_str(&string).collect::<Result<Vec<_>, _>>() {
		Ok(chars) => quote!( [ #( #chars ),* ] ).into(),
		Err(err) => {
			let span = lit.span()
			              .source_text()
			              .and_then(|string| string.find(err.char))
			              .and_then(|pos| lit.token().subspan(pos .. pos + err.char.len_utf8()))
			              .unwrap_or(lit.span());
			Error::new(span, "Unknown P8SCII character.").into_compile_error()
		},
	}
}

fn p8_char_lit(lit: &LitChar) -> TokenStream {
	match p8scii::from_char(lit.value()) {
		Ok(Some(char)) => quote!( #char ).into(),
		_ => Error::new(lit.span(), "Unknown P8SCII character.").into_compile_error(),
	}
}

fn p8_integer_lit(lit: &LitInt) -> TokenStream {
	match lit.base10_parse::<i16>() {
		Ok(int) => quote!( ::p8rs_types::p8num::P8Num::from(#int) ).into(),
		Err(err) => err.to_compile_error(),
	}
}

fn p8_number_lit(source: &str, span: Span, radix: u32) -> TokenStream {
	match P8Num::from_ascii_radix(source.as_bytes(), radix) {
		Ok(num) => {
			let sign = if num.is_negative() { "-" } else { "" };
			let raw_abs = (num.to_raw() as i64).abs() as u32;
			let raw_top = raw_abs >> 16;
			let raw_bottom = raw_abs & 0xFFFF;
			let raw_lit = if radix == 2 {
				Literal::from_str(&format!("{sign}0b{raw_top:016b}_{raw_bottom:016b}_i32")).unwrap()
			} else {
				Literal::from_str(&format!("{sign}0x{raw_top:04X}_{raw_bottom:04X}_i32")).unwrap()
			};
			quote!( ::p8rs_types::p8num::P8Num::from_raw(#raw_lit) ).into()
		},
		Err(err) => Error::new(span, format!("Unexpected character in numeric literal. {err:?}")).into_compile_error(),
	}
}

fn trim_prefix<'a>(text: &'a str, prefix: &str) -> &'a str {
	text.strip_prefix(prefix).unwrap_or(text)
}
