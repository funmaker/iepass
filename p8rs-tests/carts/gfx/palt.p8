pico-8 cartridge // http://www.pico-8.com
version 43

__lua__
palt(1, true)
palt(2, false)
palt(3, true)
palt(4, false)
p8rs.test_mem("Basic", 0x5f00, 0x80)

palt()
p8rs.test_mem("Reset", 0x5f00, 0x80)

palt(0b1100111100001010)
p8rs.test_mem("Bitfield", 0x5f00, 0x80)

palt()
palt(-1, true)
palt(-2, false)
palt(20, true)
palt(21, false)
p8rs.test_mem("Overflow", 0x5f00, 0x80)
