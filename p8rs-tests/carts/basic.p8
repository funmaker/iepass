pico-8 cartridge // http://www.pico-8.com
version 43

__lua__
p8rs.test("nil", nil)
p8rs.test("number", 1.5)
p8rs.test("string", "Hello World!")
p8rs.test("boolean", true)
p8rs.test("list", {1, 2, 3})
p8rs.test("table", {
	1, 2, 3,
	foo = 42,
	bar = "baz",
	[true] = "true",
	[123] = 456,
	[{ nested = "key" }] = { nested = "table" }
})
p8rs.test_mem("draw state", 0x5f00, 0x40)
poke(0x5f44, 0, 0, 0, 0, 0, 0, 0, 0) -- reset rnd seed
p8rs.test_mem("hardware state", 0x5f40, 0x40)
p8rs.test_scr("screen")
