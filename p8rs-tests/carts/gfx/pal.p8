pico-8 cartridge // http://www.pico-8.com
version 43

__lua__
pal(1, 2)
pal(2, 3)
pal(3, 1)
pal(4, 10, 0)
pal(1, 5, 1)
pal(1, 6, 2)
p8rs.test_mem("Basic", 0x5f00, 0x80)

pal(1)
p8rs.test_mem("One arg", 0x5f00, 0x80);

pal(1, 2)
pal(2, 3)
pal(3, 1)
palt(1, true)
palt(2, false)
pal()
p8rs.test_mem("Reset", 0x5f00, 0x80);

pal()
pal(-1, 4)
pal(20, 5)
pal(-2, 6, -1)
pal(21, 7, 3)
p8rs.test_mem("Overflow", 0x5f00, 0x80)

pal()
pal({ 5, 4, 3, 2, 1 })
pal({ 6, 7, 8, 9, 10 }, 1)
pal({ 14, 13, 12, 11 }, 2)
p8rs.test_mem("Table", 0x5f00, 0x80)

pal()
pal({ 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29 })
pal({ 19, 20, [-10] = 5, [22] = 6 }, 1)
pal({ -2.5, -10, 100, 5 }, 2)
p8rs.test_mem("Table overflow", 0x5f00, 0x80)
