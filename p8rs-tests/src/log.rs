use crate::utils::str_splitn_array;

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum Log {
	TEST(String, String),
	MEM(String, u16, Box<[u8]>),
	SCR(String, [u8; 16], Box<[[u8; 128]; 128]>),
	OTHER(String),
}

impl Log {
	pub fn parse(text: &str) -> Vec<Log> {
		let mut logs = vec![];
		
		let mut lines = text.lines();
		while let Some(line) = lines.next() {
			let log =
				line.strip_prefix("INFO: ")
				    .and_then(|s| str_splitn_array(s, " | "))
				    .and_then(|[kind, name, content]| {
					    match kind {
						    "TEST" => Some(Log::TEST(name.into(), content.into())),
						    "MEM" => {
							    let (offset, data) = content.split_once(" | ")?;
							    let offset = offset.strip_prefix("0x").and_then(|offset| u16::from_str_radix(offset, 16).ok())?;
							    let data = parse_ascii_hex_string(data.as_bytes())?.into_boxed_slice();
							    Some(Log::MEM(name.to_string(), offset, data))
						    },
						    "SCR" => {
							    let pal = content.strip_prefix("pal | ")
							                     .and_then(|pal| parse_ascii_hex_string(pal.as_bytes()))
							                     .and_then(|pal| pal.try_into().ok())?;
							    let mut lookahead = lines.clone();
							    let rows = parse_scr_lines(&mut lookahead, name)?;
							    lines = lookahead;
							    
							    Some(Log::SCR(name.to_string(), pal, rows))
						    },
						    _ => None,
					    }
				    });
			
			logs.push(log.unwrap_or_else(|| Log::OTHER(line.to_string())));
		}
		
		logs
	}
	
	pub fn name(&self) -> Option<&str> {
		match self {
			Log::TEST(name, _) => Some(name),
			Log::MEM(name, _, _) => Some(name),
			Log::SCR(name, _, _) => Some(name),
			Log::OTHER(_) => None,
		}
	}
	
	pub fn kind(&self) -> &str {
		match self {
			Log::TEST(_, _) => "TEST",
			Log::MEM(_, _, _) => "MEM",
			Log::SCR(_, _, _) => "SCR",
			Log::OTHER(_) => "OTHER",
		}
	}
}

fn parse_ascii_hexdigit(digit: u8) -> Option<u8> {
	match digit {
		b'0'..=b'9' => Some(digit - b'0'),
		b'a'..=b'f' => Some(digit - b'a' + 10),
		b'A'..=b'F' => Some(digit - b'A' + 10),
		_ => None,
	}
}

fn parse_ascii_hex_string(data: &[u8]) -> Option<Vec<u8>> {
	data.iter()
	    .copied()
	    .filter(|ch| !ch.is_ascii_whitespace())
	    .map(parse_ascii_hexdigit)
	    .array_chunks()
	    .map(|[hi, lo]| hi.zip(lo).map(|(hi, lo)| (hi << 4) | lo))
	    .collect()
}

fn parse_scr_line(cur_name: &str, cur_row: usize, line: &str) -> Option<[u8; 128]> {
	let line = line.strip_prefix("INFO: ")?;
	let [kind, name, row, data] = str_splitn_array(line, " | ")?;
	let row = row.trim().parse().ok()?;
	
	if kind != "SCR" || name != cur_name || cur_row != row {
		return None;
	}
	
	data.bytes()
	    .filter(|ch| !ch.is_ascii_whitespace())
	    .map(parse_ascii_hexdigit)
		.array_chunks()
		.flat_map(|[a, b]| [b, a])
	    .collect::<Option<Vec<_>>>()?
		.try_into()
		.ok()
}

fn parse_scr_lines<'a>(iter: impl Iterator<Item = &'a str>, name: &str) -> Option<Box<[[u8; 128]; 128]>> {
	iter.take(128)
	    .enumerate()
	    .map(|(id, line)| parse_scr_line(name, id, line))
	    .collect::<Option<Vec<_>>>()?
		.into_boxed_slice()
		.try_into()
		.ok()
}
