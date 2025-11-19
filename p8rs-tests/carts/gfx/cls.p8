pico-8 cartridge // http://www.pico-8.com
version 43

__lua__
rectfill(32, 32, 96, 96, 8)
camera(1, 2)
clip(3, 4, 5, 6)
cursor(7, 8)
color(9)
cls()
p8rs.test_scr("Basic");
p8rs.test_mem("Basic - Draw State", 0x5f00, 0x80)

cls(11)
p8rs.test_scr("Color");
p8rs.test_mem("Color - Draw State", 0x5f00, 0x80)

pal(12, 8)
cls(12)
p8rs.test_scr("Ignore pal");
p8rs.test_mem("Ignore pal - Draw State", 0x5f00, 0x80)
