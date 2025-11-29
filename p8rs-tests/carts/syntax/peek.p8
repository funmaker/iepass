pico-8 cartridge // http://www.pico-8.com
version 43

__lua__

poke(0x1234, 0x12)
poke(0x1235, 0x34)
poke(0x1236, 0x56)
poke(0x1237, 0x78)
p8rs.test("Peek Operator", @0x1234)
p8rs.test("Peek2 Operator", %0x1234)
p8rs.test("Peek4 Operator", $0x1234)


poke(0xfffe, 0x12)
poke(0xffff, 0x34)
poke(0x0000, 0x56)
poke(0x0001, 0x78)
p8rs.test("Peek2 Operator - boundary", %0xffff)
p8rs.test("Peek4 Operator - boundary", $0xfffe)
