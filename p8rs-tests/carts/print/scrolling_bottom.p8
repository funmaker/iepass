pico-8 cartridge // http://www.pico-8.com
version 43

__lua__

cls()
print("xx", 0, 115)
local cursor1 = {@0x5f26, @0x5f27}
print("yy")
local cursor2 = {@0x5f26, @0x5f27}

local test_data = { cursor1, cursor2 }

p8rs.test("Cursor", cursor1)
p8rs.test_scr("Single line on the bottom")
print("yy")
p8rs.test_scr("Second line on the bottom")
