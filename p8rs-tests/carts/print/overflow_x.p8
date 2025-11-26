pico-8 cartridge // http://www.pico-8.com
version 43

__lua__

local text = "0123456789"

print(text, 100, 10)
local cursor1 = {@0x5f26, @0x5f27}
p8rs.test_scr("Overflow disabled")

poke(0x5f36, 0x80)
print(text, 100, 40)
p8rs.test_scr("Overflow enabled")

p8rs.test("Cursors", {cursor1, {@0x5f26, @0x5f27}})

cls()