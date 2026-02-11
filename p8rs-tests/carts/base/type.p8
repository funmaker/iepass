pico-8 cartridge // http://www.pico-8.com
version 43
__lua__

p8rs.test("nil", type(nil))
p8rs.test("number", type(1.5))
p8rs.test("string", type("Hello World!"))
p8rs.test("boolean", type(true))
p8rs.test("list", type({1, 2, 3}))
p8rs.test("table", type({
	foo = 42,
	bar = "baz",
}))
p8rs.test("function", type(function() end))
p8rs.test("thread", type(cocreate(function() end)))
p8rs.test("empty", type())
