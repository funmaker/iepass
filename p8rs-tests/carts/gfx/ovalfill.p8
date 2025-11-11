pico-8 cartridge // http://www.pico-8.com
version 43

__lua__
ovalfill(56, 56, 72, 72)
ovalfill(16, 64, 32, 32, 8)
ovalfill(104, 104, 48, 88)
p8rs.test_scr("Basic");

cls()
color(11)
ovalfill(56, 56, 72, 72)
ovalfill(16, 64, 32, 32, 8)
ovalfill(104, 104, 48, 88)
p8rs.test_scr("Color");

cls()
color(8)
ovalfill(56, 56, 72)
color(11)
ovalfill(16, 64)
color(12)
ovalfill(104)
color(14)
ovalfill()
p8rs.test_scr("Less args");

cls()
camera(-63, -65)
ovalfill(-8, -8, 8, 8)
ovalfill(-48, 0, -32, -32, 8)
ovalfill(40, 40, -16, 16)
p8rs.test_scr("Camera");

cls()
clip(30, 34, 64, 50)
camera(-63, -65)
ovalfill(-8, -8, 8, 8)
ovalfill(-48, 0, -32, -32, 8)
ovalfill(40, 40, -16, 16)
p8rs.test_scr("Clip");
