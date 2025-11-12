pico-8 cartridge // http://www.pico-8.com
version 43

__lua__
rectfill(56, 56, 72, 72)
rectfill(16, 64, 32, 32, 8)
rectfill(104, 104, 48, 88)
p8rs.test_scr("Basic");

cls()
rectfill(56, 56, 72, 72)
rectfill(16.9, 64.9, 32.9, 32.9, 8)
rectfill(104.9, 104.9, 48.9, 88.9)
p8rs.test_scr("Frac");

cls()
rectfill(56, -56, 72, 72)
rectfill(-16.9, 64.9, 32.9, 32.9, 8)
rectfill(104.9, 104.9, -48.9, 88.9)
p8rs.test_scr("Outside");

cls()
color(11)
rectfill(56, 56, 72, 72)
rectfill(16, 64, 32, 32, 8)
rectfill(104, 104, 48, 88)
p8rs.test_scr("Color");

cls()
color(8)
rectfill(56, 56, 72)
color(11)
rectfill(16, 64)
color(12)
rectfill(104)
color(14)
rectfill()
p8rs.test_scr("Less args");

cls()
camera(-63, -65)
rectfill(-8, -8, 8, 8)
rectfill(-48, 0, -32, -32, 8)
rectfill(40, 40, -16, 16)
p8rs.test_scr("Camera");

cls()
clip(26, 34, 68, 60)
camera(-63, -65)
rectfill(-8, -8, 8, 8)
rectfill(-48, 0, -32, -32, 8)
rectfill(40, 40, -16, 16)
p8rs.test_scr("Clip");

cls()
clip(26, 34, 68, 60)
camera(-63, -65)
fillp(✽)
rectfill(-8, -8, 8, 8)
fillp(0b0011001111001100)
rectfill(-48, 0, -32, -32, 0xb8)
fillp(♥)
rectfill(40, 40, -16, 16)
fillp(…)
rectfill(-32, -32, 32, 32, 0x0c)
p8rs.test_scr("Fill Pattern");
