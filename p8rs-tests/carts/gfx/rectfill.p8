pico-8 cartridge // http://www.pico-8.com
version 43

__lua__
rectfill(56, 56, 72, 72)
rectfill(16, 64, 32, 32, 8)
rectfill(104, 104, 48, 88)
p8rs.test_scr("basic");

cls()
rectfill(56, 56, 72, 72)
rectfill(16.9, 64.9, 32.9, 32.9, 8)
rectfill(104.9, 104.9, 48.9, 88.9)
p8rs.test_scr("frac");

cls()
rectfill(56, -56, 72, 72)
rectfill(-16.9, 64.9, 32.9, 32.9, 8)
rectfill(104.9, 104.9, -48.9, 88.9)
p8rs.test_scr("outside");

cls()
color(11)
rectfill(56, 56, 72, 72)
rectfill(16, 64, 32, 32, 8)
rectfill(104, 104, 48, 88)
p8rs.test_scr("color");

cls()
color(8)
rectfill(56, 56, 72)
color(11)
rectfill(16, 64)
color(12)
rectfill(104)
color(14)
rectfill()
p8rs.test_scr("less args");

cls()
camera(-63, -65)
rectfill(-8, -8, 8, 8)
rectfill(-48, 0, -32, -32, 8)
rectfill(40, 40, -16, 16)
camera()
p8rs.test_scr("camera");

cls()
clip(26, 34, 68, 60)
camera(-63, -65)
rectfill(-8, -8, 8, 8)
rectfill(-48, 0, -32, -32, 8)
rectfill(40, 40, -16, 16)
camera()
clip()
p8rs.test_scr("clip");

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
camera()
clip()
fillp()
p8rs.test_scr("fill pattern");
