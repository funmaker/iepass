pico-8 cartridge // http://www.pico-8.com
version 43

__lua__
rect(56, 56, 72, 72)
rect(16, 64, 32, 32, 8)
rect(104, 104, 48, 88)
p8rs.test_scr("Basic");

cls()
rect(56, 56, 72, 72)
rect(16.9, 64.9, 32.9, 32.9, 8)
rect(104.9, 104.9, 48.9, 88.9)
p8rs.test_scr("Frac");

cls()
rect(56, -56, 72, 72)
rect(-16.9, 64.9, 32.9, 32.9, 8)
rect(104.9, 104.9, -48.9, 88.9)
p8rs.test_scr("Outside");

cls()
color(11)
rect(56, 56, 72, 72)
rect(16, 64, 32, 32, 8)
rect(104, 104, 48, 88)
p8rs.test_scr("Color");

cls()
color(8)
rect(56, 56, 72)
color(11)
rect(16, 64)
color(12)
rect(104)
color(14)
rect()
p8rs.test_scr("Less args");

cls()
camera(-63, -65)
rect(-8, -8, 8, 8)
rect(-48, 0, -32, -32, 8)
rect(40, 40, -16, 16)
p8rs.test_scr("Camera");

cls()
clip(26, 34, 68, 60)
camera(-63, -65)
rect(-8, -8, 8, 8)
rect(-48, 0, -32, -32, 8)
rect(40, 40, -16, 16)
p8rs.test_scr("Clip");

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
p8rs.test_scr("Fill Pattern");
