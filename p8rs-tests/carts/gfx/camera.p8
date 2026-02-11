pico-8 cartridge // http://www.pico-8.com
version 43

__lua__
p8rs.test("Basic - Return", camera(10, 10))
rectfill(-16, -16, 16, 16, 8)
p8rs.test_scr("Basic");
p8rs.test_mem("Basic - Memory", 0x5f00, 0x80)

cls()
p8rs.test("Negative - Return", camera(-10, -10))
rectfill(-16, -16, 16, 16, 8)
p8rs.test_scr("Negative");
p8rs.test_mem("Negative - Memory", 0x5f00, 0x80)

cls()
p8rs.test("Frac - Return", camera(-10.2, -10.8))
rectfill(-16, -16, 16, 16, 8)
p8rs.test_scr("Frac");
p8rs.test_mem("Frac - Memory", 0x5f00, 0x80)

cls()
p8rs.test("Frac 2 - Return", camera(-10.5, 10.5))
rectfill(-16, -16, 16, 16, 8)
p8rs.test_scr("Frac 2");
p8rs.test_mem("Frac 2 - Memory", 0x5f00, 0x80)

cls()
p8rs.test("Outside - Return", camera(1000, -1000))
rectfill(-16, -16, 16, 16, 8)
p8rs.test_scr("Outside");
p8rs.test_mem("Outside - Memory", 0x5f00, 0x80)

cls()
p8rs.test("Only X - Return", camera(-64))
rectfill(-16, -16, 16, 16, 8)
p8rs.test_scr("Only X");
p8rs.test_mem("Only X - Memory", 0x5f00, 0x80)

cls()
p8rs.test("No args - Return", camera())
rectfill(-16, -16, 16, 16, 8)
p8rs.test_scr("No args");
p8rs.test_mem("No args - Memory", 0x5f00, 0x80)

cls()
camera(-64, -64)
rectfill(-10, -10, 10, 10, 8)
camera(5, 5)
rectfill(-10, -10, 10, 10, 11)
camera(-133, -133)
rectfill(-10, -10, 10, 10, 12)
p8rs.test_scr("Draw");

cls()
clip(32, 32, 64, 64)
camera(-64, -64)
rectfill(-10, -10, 10, 10, 8)
camera(-32, -32)
rectfill(-10, -10, 10, 10, 11)
camera(-96, -96)
rectfill(-10, -10, 10, 10, 12)
p8rs.test_scr("Draw with clip");
