pico-8 cartridge // http://www.pico-8.com
version 43

__lua__
function test(text)
        local name = " - '" .. sub(text, 1, 5)
        if #text > 5 then name = name .. '...' end
        name = name .. "'"
        cls()
        p8rs.test("Normal - Pos "..name, print(text, 20, 90))
        p8rs.test("Normal - Col "..name, print(text, 10, 100, 2))
        p8rs.test("Normal - Simple "..name, print(text))
        p8rs.test("Normal - Negative "..name, print(text, -100, -5))
        p8rs.test("Normal - Far "..name, print(text, 250, 5))
        cls()
        -- large text
        poke(0x5f58, 1 | (1<<2) | (1<<3))
        p8rs.test("Large - Pos "..name, print(text, 20, 80))
        p8rs.test("Large - Col "..name, print(text, 10, 95, 2))
        p8rs.test("Large - Simple "..name, print(text))
        -- reset large text
        poke(0x5f58, 0)
end

test("test")
test("3 lines print\nsecond\nthird")
test("very long line with many characters that will not fit on the screen\n")
