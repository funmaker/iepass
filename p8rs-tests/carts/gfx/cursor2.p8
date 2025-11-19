pico-8 cartridge // http://www.pico-8.com
version 43

__lua__
p8rs.test_mem("Initial", 0x5f00, 0x80)

cursor(250, 20)
p8rs.test_mem("Outside", 0x5f00, 0x80)
print("outside")

cursor(-6, 30)
p8rs.test_mem("Negative", 0x5f00, 0x80)
print("clipped")

poke(0x5f24, 5)
poke(0x5f26, 5)
poke(0x5f27, 40)
p8rs.test_mem("Poked", 0x5f00, 0x80)
print("poked")

p8rs.test_scr("Screen", 0x5f00, 0x80)
