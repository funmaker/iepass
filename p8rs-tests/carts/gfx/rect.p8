pico-8 cartridge // http://www.pico-8.com
version 43

__lua__
rect(56, 56, 72, 72)
rect(16, 64, 32, 32, 8)
rect(104, 104, 48, 88)
p8rs.test_scr("basic");

cls()
rect(56, 56, 72, 72)
rect(16.9, 64.9, 32.9, 32.9, 8)
rect(104.9, 104.9, 48.9, 88.9)
p8rs.test_scr("frac");

cls()
rect(56, -56, 72, 72)
rect(-16.9, 64.9, 32.9, 32.9, 8)
rect(104.9, 104.9, -48.9, 88.9)
p8rs.test_scr("outside");

cls()
color(11)
rect(56, 56, 72, 72)
rect(16, 64, 32, 32, 8)
rect(104, 104, 48, 88)
p8rs.test_scr("color");

cls()
color(8)
rect(56, 56, 72)
color(11)
rect(16, 64)
color(12)
rect(104)
color(14)
rect()
p8rs.test_scr("less args");

cls()
camera(-63, -65)
rect(-8, -8, 8, 8)
rect(-48, 0, -32, -32, 8)
rect(40, 40, -16, 16)
camera()
p8rs.test_scr("camera");

cls()
clip(26, 34, 68, 60)
camera(-63, -65)
rect(-8, -8, 8, 8)
rect(-48, 0, -32, -32, 8)
rect(40, 40, -16, 16)
camera()
clip()
p8rs.test_scr("clip");

cls()
clip(26, 34, 68, 60)
camera(-63, -65)
fillp(✽)
rect(-8, -8, 8, 8)
fillp(0b0011001111001100)
rect(-48, 0, -32, -32, 0xb8)
fillp(♥)
rect(40, 40, -16, 16)
fillp(…)
rect(-32, -32, 32, 32, 0x0c)
camera()
clip()
fillp()
p8rs.test_scr("fill pattern");

cls()
pal(4, 0x23)
pal(2, 0x34)
pal(4, 0x53, 2)
pal(5, 0x48, 2)
palt(4, false)
fillp(0b0011001111001100.000)
rect(0, 32, 48, 48, 64, 4)
fillp(0b0011001111001100.001)
rect(0, 48, 48, 64, 64, 4)
fillp(0b0011001111001100.010)
rect(0, 48, 64, 64, 80, 4)
fillp(0b0011001111001100.011)
rect(0, 32, 64, 48, 80, 4)
fillp(0b0011001111001100.100)
rect(0, 64, 48, 80, 64, 4)
fillp(0b0011001111001100.101)
rect(0, 80, 48, 96, 64, 4)
fillp(0b0011001111001100.110)
rect(0, 80, 64, 96, 80, 4)
fillp(0b0011001111001100.111)
rect(0, 64, 64, 80, 80, 4)
fillp()
p8rs.test_scr("Pattern flags");
