pico-8 cartridge // http://www.pico-8.com
version 43

__lua__

cls()
print("xx", 0, 118)
local cursor1 = {@0x5f26, @0x5f27}
printh("Printed, now cursor " .. (cursor1[1]) .. ", " .. (cursor1[2]))

p8rs.test("Cursor", cursor1)
p8rs.test_scr("Single line on the bottom")

print("yy")
cursor1 = {@0x5f26, @0x5f27}
printh("Printed second, now cursor " .. (cursor1[1]) .. ", " .. (cursor1[2]))

print("zz")
cursor1 = {@0x5f26, @0x5f27}
printh("Printed third, now cursor " .. (cursor1[1]) .. ", " .. (cursor1[2]))

p8rs.test_scr("Three lines on the bottom")

-- large text
poke(0x5f58, 1 | (1<<2) | (1<<3))


for offset=0,7 do
        cls()
        print('==', 0, 20 + offset)
        for i=0,8 do
                print("line" .. i .. "\n--2--\n--3--")
        end
        p8rs.test_scr("Newlines, offset="..offset)
end