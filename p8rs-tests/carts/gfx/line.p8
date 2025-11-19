pico-8 cartridge // http://www.pico-8.com
version 43

__lua__
line(64, 8, 120, 64)
line(120, 64, 64, 120)
line(64, 120, 8, 64, 8)
line(8, 64, 64, 8)
line(32, 64, 96, 64, 11)
line(64, 32, 64, 96)
line(40, 56, 64, 64, 12)
line(56, 40, 64, 64)
line(72, 40, 64, 64)
line(88, 56, 64, 64)
line(88, 72, 64, 64)
line(72, 88, 64, 64)
line(56, 88, 64, 64)
line(40, 72, 64, 64)
p8rs.test_scr("Basic");

cls()
line(64.9, 8.9, 120.9, 64.1)
line(120.9, 64.9, 64.9, 120.1)
line(64.9, 120.9, 8.9, 64.1, 8)
line(8.9, 64.9, 64.9, 8.1)
line(32.9, 64.9, 96.9, 64.1, 11)
line(64.9, 32.9, 64.9, 96.1)
line(40.9, 56.9, 64.9, 64.1, 12)
line(56.9, 40.9, 64.9, 64.1)
line(72.9, 40.9, 64.9, 64.1)
line(88.9, 56.9, 64.9, 64.1)
line(88.9, 72.9, 64.9, 64.1)
line(72.9, 88.9, 64.9, 64.1)
line(56.9, 88.9, 64.9, 64.1)
line(40.9, 72.9, 64.9, 64.1)
p8rs.test_scr("Frac");

cls()
line(64, -8, 136, 64)
line(136, 64, 64, 136)
line(64, 136, -8, 64, 8)
line(-8, 64, 64, -8)
line(-32, 64, 160, 64, 11)
line(64, -32, 64, 160)
line(-40, -56, 64, 64, 12)
line(-56, -40, 64, 64)
line(184, -40, 64, 64)
line(168, -56, 64, 64)
line(168, 184, 64, 64)
line(184, 168, 64, 64)
line(-56, 168, 64, 64)
line(-40, 184, 64, 64)
p8rs.test_scr("Outside");

cls()
line(64, 32, 83, 90, 12)
line(34, 54)
line(94, 54)
line(45, 90)
line(64, 32)
p8rs.test_scr("Continuation");

cls()
p8rs.test_mem("Memory - cls", 0x5f00, 0x80);
line(64, 32, 83, 90, 8)
p8rs.test_mem("Memory - full", 0x5f00, 0x80);
line(34, 54, 11)
p8rs.test_mem("Memory - cont 1", 0x5f00, 0x80);
line(12)
p8rs.test_mem("Memory - col", 0x5f00, 0x80);
line(94, 54)
p8rs.test_mem("Memory - set pos", 0x5f00, 0x80);
line(45, 90)
p8rs.test_mem("Memory - cont 2", 0x5f00, 0x80);
line()
p8rs.test_mem("Memory - reset", 0x5f00, 0x80);
line(64, 32)
p8rs.test_mem("Memory - set pos 2", 0x5f00, 0x80);
line(64, 64)
p8rs.test_mem("Memory - cont 3", 0x5f00, 0x80);
p8rs.test_scr("Less args");

cls()
camera(-5, -15)
line(64, 8, 120, 64)
line(120, 64, 64, 120)
line(64, 120, 8, 64, 8)
line(8, 64, 64, 8)
line(32, 64, 96, 64, 11)
line(64, 32, 64, 96)
line(40, 56, 64, 64, 12)
line(56, 40, 64, 64)
line(72, 40, 64, 64)
line(88, 56, 64, 64)
line(88, 72, 64, 64)
line(72, 88, 64, 64)
line(56, 88, 64, 64)
line(40, 72, 64, 64)
p8rs.test_scr("Camera");

cls()
clip(30, 34, 64, 50)
camera(-6, -15)
line(64, 8, 120, 64)
line(120, 64, 64, 120)
line(64, 120, 8, 64, 8)
line(8, 64, 64, 8)
line(32, 64, 96, 64, 11)
line(64, 32, 64, 96)
line(40, 56, 64, 64, 12)
line(56, 40, 64, 64)
line(72, 40, 64, 64)
line(88, 56, 64, 64)
line(88, 72, 64, 64)
line(72, 88, 64, 64)
line(56, 88, 64, 64)
line(40, 72, 64, 64)
p8rs.test_scr("Clip");

cls()
clip(30, 34, 64, 50)
camera(-6, -15)
fillp(▤)
line(64, 8, 120, 64)
line(120, 64, 64, 120)
line(64, 120, 8, 64, 8)
line(8, 64, 64, 8)
fillp(0b0011001111001100)
line(32, 64, 96, 64, 0xb8)
line(64, 32, 64, 96)
fillp(♥)
line(40, 56, 64, 64, 12)
line(56, 40, 64, 64)
line(72, 40, 64, 64)
line(88, 56, 64, 64)
line(88, 72, 64, 64)
line(72, 88, 64, 64)
line(56, 88, 64, 64)
line(40, 72, 64, 64)
p8rs.test_scr("Fill Pattern");
