use p8rs_macros::api_callback;
use p8rs_piccolo::{Context};
use p8rs_types::p8num::P8Num;
use crate::utils::once;
use crate::vm::memory::MemoryAccess;
use crate::vm::Runtime;

pub fn install(ctx: Context) {
	ctx.set_global(b"cartdata", cartdata::callback(ctx));
	ctx.set_global(b"dget", dget::callback(ctx));
	ctx.set_global(b"dset", dset::callback(ctx));
	ctx.set_global(b"cstore", cstore::callback(ctx));
	ctx.set_global(b"reload", reload::callback(ctx));
}

#[api_callback]
pub fn cartdata() {
	once!{ warn!("cartdata is not implemented yet!"); }
}

#[api_callback]
pub fn dget(rt: &mut Runtime, index: i16) -> P8Num {
	// TODO: emit error if cartdata was not called
	if index >= 0 && index < 64 {
		rt.memory.persistent_data().read(index as u16 * 4)
	} else {
		P8Num::ZERO
	}
}

#[api_callback]
pub fn dset(rt: &mut Runtime, index: i16, value: P8Num) {
	// TODO: emit error if cartdata was not called
	if index >= 0 && index < 64 {
		rt.memory.persistent_data().write(index as u16 * 4, value)
	}
}

#[api_callback]
pub fn cstore() {
	once!{ warn!("cstore is not implemented yet!"); }
}

#[api_callback]
pub fn reload() {
	once!{ warn!("reload is not implemented yet!"); }
}


