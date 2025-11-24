pico-8 cartridge // http://www.pico-8.com
version 43

__lua__
pal(1, 2)
pal(2, 3)
poke(0x5f03, 0xFF)
pal(3, 1)
pal(4, 10, 0)
pal(1, 5, 1)
pal(1, 6, 2)
p8rs.test_mem("basic", 0x5f00, 0x80)

pal(1)
p8rs.test_mem("one arg", 0x5f00, 0x80);

pal(1, 2)
pal(2, 3)
pal(3, 1)
palt(1, true)
palt(2, false)
poke(0x5f05, 0xFF)
poke(0x5f15, 0xFF)
poke(0x5f65, 0xFF)
pal(4, 10, 0)
pal(1, 5, 1)
pal(1, 6, 2)
pal()
p8rs.test_mem("reset", 0x5f00, 0x80);

pal()
pal(2, 0xFFFF)
pal(3, -0xFFFF)
pal(-1, 4)
pal(20, 5)
pal(-2, 6, -1)
pal(21, 7, 3)
p8rs.test_mem("overflow", 0x5f00, 0x80)

pal()
poke(0x5f02, 0xFF)
poke(0x5f12, 0xFF)
poke(0x5f62, 0xFF)
poke(0x5f03, 0xFF)
poke(0x5f13, 0xFF)
poke(0x5f63, 0xFF)
pal(2, 3, 0)
pal(2, 3, 1)
pal(2, 3, 2)
pal(3, 0xEE, 0)
pal(3, 0xEE, 1)
pal(3, 0xEE, 2)
p8rs.test_mem("overwrite", 0x5f00, 0x80);

pal()
pal({ 5, 4, 3, 2, 1 })
pal({ 6, 7, 8, 9, 10 }, 1)
pal({ 14, 13, 12, 11 }, 2)
p8rs.test_mem("table", 0x5f00, 0x80)

pal()
pal({ 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30 })
p8rs.test_mem("table overflow 1", 0x5f00, 0x80)
pal({ 19, 20, [-10] = 5, [23] = 6 }, 1)
p8rs.test_mem("table overflow 2", 0x5f00, 0x80)
pal({ -2.5, -10, 100, 5 }, 2)
p8rs.test_mem("table overflow 3", 0x5f00, 0x80)

pal()
poke(0x5f02, 0xFF)
poke(0x5f12, 0xFF)
poke(0x5f62, 0xFF)
poke(0x5f03, 0xFF)
poke(0x5f13, 0xFF)
poke(0x5f63, 0xFF)
pal({ 1, 2, 0xEE }, 0)
pal({ 1, 2, 0xEE }, 1)
pal({ 1, 2, 0xEE }, 2)
p8rs.test_mem("table overwrite", 0x5f00, 0x80);
