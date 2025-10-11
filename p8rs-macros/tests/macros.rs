use p8rs_macros::p8;
use p8rs_types::p8num::P8Num;

#[test]
fn test_char() {
	assert_eq!(p8!('a'), b'a');
	assert_eq!(p8!(' '), b' ');
	assert_eq!(p8!('\n'), b'\n');
	assert_eq!(p8!('あ'), 154);
	assert_eq!(p8!('ア'), 204);
	assert_eq!(p8!('◝'), 255);
	assert_eq!(p8!('\0'), 0);
	assert_eq!(p8!('ᶠ'), 15);
}

#[test]
fn test_str() {
	assert_eq!(
		&p8!("The quick brown fox jumps over the lazy dog."),
		b"The quick brown fox jumps over the lazy dog.",
	);
	assert_eq!(
		p8!("みく、みくにしてあけ゛る。"),
		[185, 161, 28, 185, 161, 175, 165, 172, 154, 162, 30, 194, 29],
	);
	assert_eq!(
		p8!("⬆️⬇️⬅️➡️🅾️❎█▒░▤▥"),
		[148, 131, 139, 145, 142, 151, 128, 129, 132, 152, 153],
	);
	assert_eq!(
		p8!("\0¹²³⁴⁵⁶⁷⁸\t\nᵇᶜ\rᵉᶠ"),
		[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
	);
}

#[test]
fn test_num() {
	assert_eq!(p8!(0), P8Num::from(0_i16));
	assert_eq!(p8!(1234), P8Num::from(1234_i16));
	assert_eq!(p8!(-42), P8Num::from(-42_i16));
	
	assert_eq!(p8!(0x12AB), P8Num::from(0x12AB_i16));
	assert_eq!(p8!(-0x12AB), P8Num::from(-0x12AB_i16));
	
	assert_eq!(p8!(0b101010101010101), P8Num::from(0b101010101010101_i16));
	assert_eq!(p8!(-0b111000011110000), P8Num::from(-0b111000011110000_i16));
	
	assert_eq!(p8!(0.0), P8Num::new_f64(0.0));
	assert_eq!(p8!(5.0), P8Num::new_f64(5.0));
	assert_eq!(p8!(0.5), P8Num::new_f64(0.5));
	assert_eq!(p8!(-4.25), P8Num::new_f64(-4.25));
	
	assert_eq!(p8!(hex ABC), P8Num::from_raw(0x0ABC_0000));
	assert_eq!(p8!(hex 1234.5678), P8Num::from_raw(0x1234_5678));
	assert_eq!(p8!(hex ABC.ABCD), P8Num::from_raw(0x0ABC_ABCD));
	assert_eq!(p8!(hex -1234.5678), P8Num::from_raw(-0x1234_5678));
	assert_eq!(p8!(hex -0.0), P8Num::from_raw(0x0000_0000));
	assert_eq!(p8!(hex 7FFF.FFFF), P8Num::from_raw(0x7FFF_FFFF));
	assert_eq!(p8!(hex -8000.0000), P8Num::from_raw(-0x8000_0000));
	assert_eq!(p8!(hex 8000.0000), P8Num::from_raw(-0x8000_0000));
	assert_eq!(p8!(hex 100123.0), P8Num::from_raw(0x0123_0000));
	
	assert_eq!(p8!(bin 1010), P8Num::from_raw(0x000A_0000));
	assert_eq!(p8!(bin 1010.0101), P8Num::from_raw(0x000A_5000));
	assert_eq!(p8!(bin 1111000011100000.1101000011000000), P8Num::from_raw(0b1111000011100000_1101000011000000_u32.cast_signed()));
	assert_eq!(p8!(bin -0111000011100000.1101000011000000), P8Num::from_raw(-0b111000011100000_1101000011000000));
}
