pico-8 cartridge // http://www.pico-8.com
version 43

__lua__
circ(32, 96)
circ(96, 96, 16, 8)
circ(64, 64, 16)
p8rs.test_scr("Basic");

cls()
color(11)
circ(32, 96)
circ(96, 96, 16, 8)
circ(64, 64, 16)
p8rs.test_scr("Color");

cls()
color(8)
circ(32)
color(11)
circ()
p8rs.test_scr("Less args");

cls()
camera(-63, -65)
circ(-32, 16)
circ(32, 16, 16, 8)
circ(0, 0, 16)
p8rs.test_scr("Camera");

cls()
camera(-63, -65)
clip(30, 34, 64, 50)
circ(-32, 16)
circ(32, 16, 16, 8)
circ(0, 0, 16)
p8rs.test_scr("Clip");
