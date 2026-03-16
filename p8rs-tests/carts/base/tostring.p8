pico-8 cartridge // http://www.pico-8.com
version 43
__lua__

p8rs.test("0", tostring(0))
p8rs.test("0x7fff.ffff", tostring(0x7fff.ffff))
p8rs.test("-1", tostring(-1))
p8rs.test("0x8000.0000", tostring(0x8000.0000))
p8rs.test("0x0000.0001", tostring(0x0000.0001))
p8rs.test("0xffff.ffff", tostring(0xffff.ffff))
p8rs.test("0x1234.5678", tostring(0x1234.5678))
p8rs.test("nil", tostring(nil))
p8rs.test("string", tostring("Hello World!"))
p8rs.test("false", tostring(false))
p8rs.test("true", tostring(true))
p8rs.test("list", sub(tostring({1, 2, 3}), 1, 9))
p8rs.test("table", sub(tostring({ foo = 42, bar = "baz" }), 1, 9))
p8rs.test("function", sub(tostring(function() end), 1, 12))
p8rs.test("thread", sub(tostring(cocreate(function() end)), 1, 10))
