use p8rs_macros::api_callback;
use p8rs_piccolo::{Context};
use crate::utils::once;

pub fn install(ctx: Context) {
	ctx.set_global(b"help", help::callback(ctx));
	ctx.set_global(b"info", info::callback(ctx));
	ctx.set_global(b"install_demos", install_demos::callback(ctx));
	ctx.set_global(b"install_games", install_games::callback(ctx));
	ctx.set_global(b"splore", splore::callback(ctx));
	ctx.set_global(b"backup", backup::callback(ctx));
	ctx.set_global(b"cd", cd::callback(ctx));
	ctx.set_global(b"export", export::callback(ctx));
	ctx.set_global(b"folder", folder::callback(ctx));
	ctx.set_global(b"import", import::callback(ctx));
	ctx.set_global(b"load", load::callback(ctx));
	ctx.set_global(b"mkdir", mkdir::callback(ctx));
	ctx.set_global(b"save", save::callback(ctx));
	ctx.set_global(b"reboot", reboot::callback(ctx));
	ctx.set_global(b"reset", reset::callback(ctx));
	ctx.set_global(b"run", run::callback(ctx));
	ctx.set_global(b"keyconfig", keyconfig::callback(ctx));
	ctx.set_global(b"login", login::callback(ctx));
	ctx.set_global(b"scoresub", scoresub::callback(ctx));
	
	let shutdown = shutdown::callback(ctx);
	ctx.set_global(b"shutdown", shutdown);
	ctx.set_global(b"exit", shutdown);
	
	let dir = dir::callback(ctx);
	ctx.set_global(b"dir", dir);
	ctx.set_global(b"ls", dir);
}

#[api_callback]
pub fn help() {
	once!{ warn!("help is not implemented yet!"); }
}

#[api_callback]
pub fn info() {
	once!{ warn!("info is not implemented yet!"); }
}

#[api_callback]
pub fn install_demos() {
	once!{ warn!("install_demos is not implemented yet!"); }
}

#[api_callback]
pub fn install_games() {
	once!{ warn!("install_games is not implemented yet!"); }
}

#[api_callback]
pub fn splore() {
	once!{ warn!("splore is not implemented yet!"); }
}

#[api_callback]
pub fn backup() {
	once!{ warn!("backup is not implemented yet!"); }
}

#[api_callback]
pub fn cd() {
	once!{ warn!("cd is not implemented yet!"); }
}

#[api_callback]
pub fn dir() {
	once!{ warn!("dir is not implemented yet!"); }
}

#[api_callback]
pub fn export() {
	once!{ warn!("export is not implemented yet!"); }
}

#[api_callback]
pub fn folder() {
	once!{ warn!("folder is not implemented yet!"); }
}

#[api_callback]
pub fn import() {
	once!{ warn!("import is not implemented yet!"); }
}

#[api_callback]
pub fn load() {
	once!{ warn!("load is not implemented yet!"); }
}

#[api_callback]
pub fn mkdir() {
	once!{ warn!("mkdir is not implemented yet!"); }
}

#[api_callback]
pub fn save() {
	once!{ warn!("save is not implemented yet!"); }
}

#[api_callback]
pub fn reboot() {
	once!{ warn!("reboot is not implemented yet!"); }
}

#[api_callback]
pub fn reset() {
	once!{ warn!("reset is not implemented yet!"); }
}

#[api_callback]
pub fn run() {
	once!{ warn!("run is not implemented yet!"); }
}

#[api_callback]
pub fn shutdown() {
	once!{ warn!("shutdown is not implemented yet!"); }
}

#[api_callback]
pub fn keyconfig() {
	once!{ warn!("keyconfig is not implemented yet!"); }
}

#[api_callback]
pub fn login() {
	once!{ warn!("login is not implemented yet!"); }
}

#[api_callback]
pub fn scoresub() {
	once!{ warn!("scoresub is not implemented yet!"); }
}

