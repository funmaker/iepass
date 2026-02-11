pico-8 cartridge // http://www.pico-8.com
version 43
__lua__

for flags = 0b00,0b11 do
  p8rs.test("nil " .. flags, tostr(nil, flags))
  p8rs.test("number " .. flags, tostr(1.5, flags))
  p8rs.test("string " .. flags, tostr("Hello World!", flags))
  p8rs.test("boolean " .. flags, tostr(true, flags))
  p8rs.test("list " .. flags, sub(tostr({1, 2, 3}, flags), 1, 9))
  p8rs.test("table " .. flags, sub(tostr({ foo = 42, bar = "baz" }, flags), 1, 9))
  p8rs.test("function " .. flags, sub(tostr(function() end, flags), 1, 12))
  p8rs.test("thread " .. flags, tostr(cocreate(function() end), flags))
  p8rs.test("empty", tostr())
end
