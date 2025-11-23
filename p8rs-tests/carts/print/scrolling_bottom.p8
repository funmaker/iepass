pico-8 cartridge // http://www.pico-8.com
version 43

__lua__

for offset=0,15 do
        cls()
        local y = 115 + offset
        print("xx", 0, y)
        local cursor1 = {@0x5f26, @0x5f27}
        print("yy")
        local cursor2 = {@0x5f26, @0x5f27}

        p8rs.test_scr("printing starting "..y.." (#"..offset..")")
        p8rs.test("Cursors", { cursor1, cursor2 })
end

-- large text
poke(0x5f58, 1 | (1<<2) | (1<<3))

for offset=0,15 do
        cls()
        local y = 115 + offset
        print("xx", 0, y)
        local cursor1 = {@0x5f26, @0x5f27}
        print("yy")
        local cursor2 = {@0x5f26, @0x5f27}

        p8rs.test_scr("printing starting "..y.." (#"..offset..") - large text")
        p8rs.test("Cursors", { cursor1, cursor2 })
end