pico-8 cartridge // http://www.pico-8.com
version 43

__lua__


function clear()
    cls(8)
    local str = ""
    for i=0,15 do
        local hex = sub(tostr(i, true), 6, 6)
        str = str .. "\f" .. hex .. hex .. " "
    end
    print(str)
end

clear()

p8rs.test_scr("init")


print("Test 0\f4test", 3)

rect(80,10, 85,15)


function chpal(offset, pal_idx)
    for i=0,15 do
        pal(i, (i + offset) & 0xf, pal_idx)
    end
end

chpal(2, 0)
print("Test 1\f3test", 3)
--p8rs.test_scr("Pal 0 changed")
chpal(0, 0)

chpal(2, 1)
print("Test 2\f3test", 3)
--p8rs.test_scr("Pal 1 changed")
chpal(0, 1)

chpal(2, 2)
print("Test 3\f4test", 3)
print("Test 3 noclr\f4test")
p8rs.test_scr("Pal 2 changed")
chpal(0, 2)


-- czy zmiana kolory wplywa na inne funkcje - tak
-- co trafia do screen buforu po draw palecie
-- flaga force secondary palette


local fill_patterns = {
    0b0011001111001100.000,
    0b0011001111001100.001,
    0b0011001111001100.010,
    0b0011001111001100.011,
    0b0011001111001100.100,
    0b0011001111001100.101,
    0b0011001111001100.110,
    0b0011001111001100.111,
}

chpal(0, 0)
chpal(0, 1)
chpal(0, 2)

clear()
chpal(1, 0)
chpal(2, 1)
chpal(4, 2)
for _, pattern in ipairs(fill_patterns) do
    fillp(pattern)
    poke(0x5f58, 1 | 1<<4)
    print("Test\f8zx\#2cv!"..tostr(pattern, true), 1 | (6<<4))
    poke(0x5f58, 1 | 0)
end
chpal(0, 0)
chpal(0, 1)
chpal(0, 2)
print(".")
p8rs.test_scr("Patterns")



