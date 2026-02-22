pico-8 cartridge // http://www.pico-8.com
version 43
__lua__

function test(label, ...)
  p8rs.test(
    label .. " numbers",
    tostr(0, ...),
    tostr(0x7fff.ffff, ...),
    tostr(-1, ...),
    tostr(0x8000.0000, ...),
    tostr(0x0000.0001, ...),
    tostr(0xffff.ffff, ...),
    tostr(0x1234.5678, ...)
  );
  p8rs.test(
    label .. " other",
    tostr(nil, ...),
    tostr("Hello World!", ...),
    tostr(true, ...),
    sub(tostr({1, 2, 3}, ...), 1, 9),
    sub(tostr({ foo = 42, bar = "baz" }, ...), 1, 9),
    sub(tostr(function() end, ...), 1, 12),
    tostr(cocreate(function() end), ...),
    tostr()
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
test("table", {
	foo = 42,
	bar = "baz",
})
test("function", function() end)
test("thread", cocreate(function() end))
