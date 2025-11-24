pico-8 cartridge // http://www.pico-8.com
version 43

__lua__

local val = @0x1234
p8rs.test("Peek Simple", val)

poke(0x1234, 123)

val = "x" .. @0x1234 .. "x"
p8rs.test("Peek Concat", val)

