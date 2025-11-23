pico-8 cartridge // http://www.pico-8.com
version 43

__lua__

local text = "0123456789"

print(text, 10, 123)
local cursor1 = {@0x5f26, @0x5f27}
p8rs.test_scr("Make space 1")

print(text, 100, 127)
p8rs.test_scr("Make space 2")

p8rs.test("Cursors", {cursor1, {@0x5f26, @0x5f27}})

cls()