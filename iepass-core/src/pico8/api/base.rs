use std::alloc::Allocator;
use anyhow::anyhow;
use crate::pico8::numeric::{number_from_ascii, NumberConversionFlags};
use p8rs_macros::api_callback;
use p8rs_piccolo::{Context, IntoValue, RuntimeError, Value, String};
use p8rs_types::p8num::{P8Num, P8NumStringConversionFlags};
use crate::pico8::Runtime;

pub fn install_pico8_base<A: Allocator + 'static>(ctx: Context) {
	// implements: assert, type, select, rawget, rawset,
	//             getmetatable, setmetatable, next, pairs, ipairs
	// extra functions: tostring, error, pcall, collectgarbage
	p8rs_piccolo::stdlib::load_base(ctx);
	
	ctx.set_global("tostr", tostr::callback(ctx));
	ctx.set_global("tonum", tonum::callback(ctx));
	ctx.set_global("printh", printh::callback(ctx));
	ctx.set_global("type", get_type::callback(ctx));
	ctx.set_global("btn", btn::callback::<A>(ctx));
	ctx.set_global("stat", stat::callback::<A>(ctx));
}

#[api_callback]
pub fn stat<'gc, A: Allocator + 'static>(rt: &mut Runtime<A>, stat_cmd: i16) -> Result<Option<Value<'gc>>, RuntimeError> {
	Ok(match stat_cmd {
		7 => { Some(Value::Number(P8Num::from(rt.fps.cast_signed()))) }
		other => { return Err(anyhow!("stat({}) not implemented!", other).into())}
	})
}

#[api_callback]
pub fn btn<'gc, A: Allocator + 'static>(rt: &mut Runtime<A>, btn_idx: Option<i16>, player_idx: Option<i16>) -> Result<Option<Value<'gc>>, RuntimeError> {
	match btn_idx {
		None => Ok(Some(Value::Number(P8Num::from((rt.buttons.get_bits_for_player(0) as u16 | (rt.buttons.get_bits_for_player(1) as u16) << 8).cast_signed())))),
		Some(button_idx) => {
			let player_idx = player_idx.unwrap_or(0);
			if player_idx < 0 || player_idx > 7 { return Ok(Some(Value::Boolean(false))) }
			
			Ok(Some(Value::Boolean(rt.buttons.is_down(player_idx as usize, button_idx as usize))))
		}
	}
}

#[api_callback]
pub fn get_type<'gc>(ctx: Context<'gc>, val: Value<'gc>) -> Result<Option<Value<'gc>>, RuntimeError> {
	Ok(Some(match val {
		Value::Nil => "nil".into_value(ctx),
		Value::Boolean(_) => "boolean".into_value(ctx),
		Value::Number(_) => "number".into_value(ctx),
		Value::String(_) => "string".into_value(ctx),
		Value::Table(_) => "table".into_value(ctx),
		Value::Function(_) => "function".into_value(ctx),
		Value::Thread(_) => "thread".into_value(ctx),
		Value::UserData(_) => "userdata".into_value(ctx),
	}))
}

#[api_callback]
pub fn tostr<'gc>(ctx: Context<'gc>, val: Value<'gc>, opts: Option<Value<'gc>>) -> Result<Option<Value<'gc>>, RuntimeError> {
	let flags = P8NumStringConversionFlags::from_bits_truncate(match opts {
		Some(Value::Boolean(true)) => 1,
		Some(Value::Number(num)) => num.to_integer() as u8,
		_ => 0,
	});
	
	let result = match val {
		Value::Nil => "[nil]".into_value(ctx),
		Value::Boolean(x) => if x { "true" } else { "false" }.into_value(ctx),
		Value::Number(num) => String::from_slice(ctx.mutation(), num.to_str_fmt(flags).as_ref().as_bytes()).into_value(ctx),
		Value::String(s) => s.into_value(ctx),
		Value::Table(_) => "[table]".into_value(ctx),
		Value::Function(_) => "[function]".into_value(ctx),
		Value::Thread(_) => "[thread]".into_value(ctx),
		Value::UserData(_) => "[userdata]".into_value(ctx),
	};
	
	Ok(Some(result))
}

#[api_callback]
pub fn tonum<'gc>(val: String, opts: Option<u8>) -> Result<Option<Value<'gc>>, RuntimeError> {
	let flags: NumberConversionFlags = NumberConversionFlags::from_bits_truncate(opts.unwrap_or(0));
	let conversion = number_from_ascii(&val, flags);
	if conversion.is_ok() {
		Ok(Some(conversion.unwrap()))
	}else{
		Ok(None)
	}
}

#[api_callback]
pub fn printh(text: String, filename: Option<String>, _overwrite: Option<bool>, _save_to_desktop: Option<bool>) -> Result<(), RuntimeError> {
	if let Some(filename_str) = filename {
		info!("[printh/{}] {}", filename_str, text);
	} else {
		info!("[printh] {}", text);
	}
	Ok(())
}




