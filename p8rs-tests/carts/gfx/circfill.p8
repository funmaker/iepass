pico-8 cartridge // http://www.pico-8.com
version 43

__lua__
circfill(32, 96)
circfill(96, 96, 16, 8)
circfill(64, 64, 16)
p8rs.test_scr("Basic");

cls()
color(11)
circfill(32, 96)
circfill(96, 96, 16, 8)
circfill(64, 64, 16)
p8rs.test_scr("Color");

cls()
color(8)
circfill(32)
color(11)
circfill()
p8rs.test_scr("Less args");

cls()
camera(-63, -65)
circfill(-32, 16)
circfill(32, 16, 16, 8)
circfill(0, 0, 16)
p8rs.test_scr("Camera");

cls()
camera(-63, -65)
clip(30, 34, 64, 50)
circfill(-32, 16)
circfill(32, 16, 16, 8)
circfill(0, 0, 16)
p8rs.test_scr("Clip");
