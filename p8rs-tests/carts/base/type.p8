pico-8 cartridge // http://www.pico-8.com
version 43
__lua__

function test(label, ...)
	p8rs.test(label, type(...), __type(...))
end

test("nil", nil)
test("number", 1.5)
test("string", "Hello World!")
test("boolean", true)
test("list", {1, 2, 3})
test("table", {
	foo = 42,
	bar = "baz",
})
test("function", function() end)
test("thread", cocreate(function() end))

p8rs.test("empty", type())
p8rs.test_err("empty __", function() __type() end)
