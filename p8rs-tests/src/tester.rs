use std::fs;
use std::path::{Path, PathBuf};

pub fn test_cartridge(path: impl AsRef<Path>) {
	let path = path.as_ref();
	fs::create_dir(
		PathBuf::from(std::env::var("CARGO_TARGET_TMPDIR").expect("must be compiled as an test or benchmark"))
			.join(path.as_os_str()
			          .to_string_lossy()
			          .replace(|c: char| c.is_ascii_alphanumeric(), "_"))
	).expect("Unable to create tmp dir");
	
	let cart = fs::read(path).expect("Unable to read cart file");
	let cart = replace(
		&cart,
		b"\n__lua__\n",
		concat!(
			"\n__lua__\n",
			include_str!("polyfill.lua"),
			"\n"
		).as_bytes(),
	);
}

fn replace(mut source: &[u8], from: &[u8], to: &[u8]) -> Vec<u8> {
	let mut out = Vec::with_capacity(source.len().saturating_sub(from.len()) + to.len());
	
	while let Some(pos) = source.windows(from.len())
	                            .position(|window| window == from) {
		out.extend_from_slice(&source[.. pos]);
		out.extend_from_slice(to);
		source = &source[pos + from.len() ..];
	}
	
	out.extend_from_slice(source);
	
	out
}

#[cfg(test)]
mod tests {
	use crate::tester::replace;
	
	#[test]
	fn replace_test() {
		assert_eq!(replace(b"Test lel kek lel wew", b"lel", b"banana"), b"Test banana kek banana wew")
	}
}
