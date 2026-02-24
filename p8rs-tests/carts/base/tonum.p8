pico-8 cartridge // http://www.pico-8.com
version 43
__lua__

function test(label, ...)
  p8rs.test(
    label .. " dec",
    tonum("1234", ...),
    tonum("-1234", ...),
    tonum("+1234", ...),
    tonum("1234.5678", ...),
    tonum("-1234.5678", ...),
    tonum("-12345678.987654321", ...)
  );
  p8rs.test(
    label .. " hex",
    tonum("0x1234", flags),
    tonum("-0x1234", flags),
    tonum("+0x1234", flags),
    tonum("0x1234.5678", flags),
    tonum("-0x1234.5678", flags),
    tonum("-0x12345678.987654321", flags)
  );
  p8rs.test(
    label .. " bin",
    tonum("0b11110000101101", flags),
    tonum("-0b11110000101101", flags),
    tonum("+0b11110000101101", flags),
    tonum("0b11110000101101.10110000111101", flags),
    tonum("-0b11110000101101.10110000111101", flags),
    tonum("-0b111100001011011010101110.101100001111010111011101", flags)
  );
  p8rs.test(
    label .. " sci",
    tonum("2.55e2", flags),
    tonum("-2.55e2", flags),
    tonum("+2.55e2", flags),
    tonum("2.55e+2", flags),
    tonum("2.55e-2", flags),
    tonum("2.5539e2", flags),
    tonum("-2.5539e2", flags),
    tonum("-2.123456789123e2", flags),
    tonum("-12345678e2", flags),
    tonum("-12345678e-2", flags),
    tonum("-12345678e10", flags)
  );
  p8rs.test(
    label .. " bad",
    tonum("0123abc", flags),
    tonum("0b123", flags),
    tonum("0o123", flags),
    tonum("0x123yzw", flags),
    tonum("0x12x45", flags),
    tonum("123.456.789", flags),
    tonum("123e--10", flags),
    tonum("123e++10", flags),
    tonum("123e-5e4e+6", flags),
    tonum("123.456.789e-5e4e+6", flags)
  );
  p8rs.test(
    label .. " limit",
    tonum("140737488355326.9", ...),
    tonum("140737488355327", ...),
    tonum("140737488355327.1", ...),
    tonum("140737488355327.9", ...),
    tonum("140737488355328", ...),
    tonum("140737488355328.1", ...),
    tonum("140737488355328.9", ...),
    tonum("140737488355329", ...),
    tonum("140737488355329.1", ...)
  );
  p8rs.test(
    label .. " limit fract",
    tonum("140737488355327", ...),
    tonum("140737488355327.5", ...),
    tonum("140737488355327.25", ...),
    tonum("140737488355327.125", ...),
    tonum("140737488355327.0625", ...),
    tonum("140737488355327.03125", ...),
    tonum("140737488355327.015625", ...),
    tonum("140737488355327.0078125", ...),
    tonum("140737488355327.00390625", ...),
    tonum("140737488355327.001953125", ...),
    tonum("140737488355327.0009765625", ...),
    tonum("140737488355327.00048828125", ...),
    tonum("140737488355327.000244140625", ...),
    tonum("140737488355327.0001220703125", ...),
    tonum("140737488355327.00006103515625", ...),
    tonum("140737488355327.000030517578125", ...),
    tonum("140737488355327.0000152587890625", ...)
  );
  p8rs.test(
    label .. " other",
    tonum(nil, flags),
    tonum(false, flags),
    tonum(true, flags),
    tonum("Hello World!", flags),
    tonum({1, 2, 3}, flags),
    tonum({ foo = 42, bar = "baz" }, flags),
    tonum(function() end, flags),
    tonum(cocreate(function() end), flags),
    tonum()
  );
end

test("no arg")
test("0b000", 0b000)
test("0b001", 0b001)
test("0b010", 0b010)
test("0b011", 0b011)
test("0b100", 0b100)
test("0b101", 0b101)
test("0b110", 0b110)
test("0b111", 0b111)
test("nil", nil)
test("number", 1.5)
test("string", "Hello World!")
test("true", true)
test("false", false)
test("list", {1, 2, 3})
test("function", function() end)

