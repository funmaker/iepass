pico-8 cartridge // http://www.pico-8.com
version 43

__lua__
for offset=0,7 do
        cls()
        print("start", 0, offset)
        for i=1,30 do
                print("line " .. i)
        end
        p8rs.test_scr("Scrolling with offset " .. offset)
end
