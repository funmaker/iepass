pico-8 cartridge // http://www.pico-8.com
version 43
__lua__

p8rs.test("nil", printh(nil))
p8rs.test("number", printh(1.5))
p8rs.test("string", printh("Hello World!"))
p8rs.test("boolean", printh(true))
p8rs.test("list", printh({1, 2, 3}))
p8rs.test("table", printh({
	foo = 42,
	bar = "baz",
}))
p8rs.test("function", printh(function() end))
p8rs.test("thread", printh(cocreate(function() end)))
p8rs.test("empty", printh())
