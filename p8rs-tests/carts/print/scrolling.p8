pico-8 cartridge // http://www.pico-8.com
version 43

__lua__

--[[

print(text, 0, 120)
  -- cursor 0, 126
  print(text)
  -- cursor 0, 132


print(text, 0, 118)
  -- cursor 0, 124
  print(text)
  -- cursor 0, 130


print(text, 0, 116)
  -- cursor 0, 122
  print(text)
  -- cursor 0, 128


print(text, 0, 114)
  -- cursor 0, 120
  print(text)
  -- cursor 0, 126
  print(text)
  -- cursor 0, 126


]]

cls()
print("xx", 0, 117)
local cursor1 = {@0x5f26, @0x5f27}
p8rs.test("Cursor", cursor1)
p8rs.test_scr("Single line on the bottom")
print("yy")
p8rs.test_scr("Second line on the bottom")
p8rs.test("Cursor 2", {cursor1, {@0x5f26, @0x5f27}})

--cls()
--for offset=0,7 do
--        cls()
--        print("start", 0, offset)
--        for i=1,30 do
--                print("line " .. i)
--        end
--        p8rs.test_scr("Scrolling with offset " .. offset)
--end
