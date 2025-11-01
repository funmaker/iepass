use std::str::pattern::Pattern;

pub fn replace(mut source: &[u8], from: &[u8], to: &[u8]) -> Vec<u8> {
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

pub fn str_splitn_array<const N: usize, P: Pattern>(s: &str, pat: P) -> Option<[&str; N]> {
	let mut iter = s.splitn(N, pat);
	std::array::try_from_fn(|_| iter.next())
}

#[cfg(test)]
mod tests {
	use super::*;
	
	#[test]
	fn replace_test() {
		assert_eq!(replace(b"Test lel kek lel wew", b"lel", b"banana"), b"Test banana kek banana wew")
	}
}