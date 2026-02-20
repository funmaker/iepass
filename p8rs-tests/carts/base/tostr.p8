pico-8 cartridge // http://www.pico-8.com
version 43
__lua__

function test(label, ...)
  p8rs.test("nil " .. label, tostr(nil, ...))
  p8rs.test("number " .. label, tostr(1.5, ...))
  p8rs.test("string " .. label, tostr("Hello World!", ...))
  p8rs.test("boolean " .. label, tostr(true, ...))
  p8rs.test("list " .. label, sub(tostr({1, 2, 3}, ...), 1, 9))
  p8rs.test("table " .. label, sub(tostr({ foo = 42, bar = "baz" }, ...), 1, 9))
  p8rs.test("function " .. label, sub(tostr(function() end, ...), 1, 12))
  p8rs.test("thread " .. label, tostr(cocreate(function() end), ...))
  p8rs.test("empty", tostr())
end

test("")
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
