pico-8 cartridge // http://www.pico-8.com
version 43

__lua__
p8rs.test("Basic - Return", clip(5, 10, 64, 64))
rectfill(30, 34, 95, 97, 8)
p8rs.test_scr("Basic");
p8rs.test_mem("Basic - Memory", 0x5f00, 0x80)

cls()
p8rs.test("Negative - Return", clip(-5, -10, 64, 64))
rectfill(30, 34, 95, 97, 8)
p8rs.test_scr("Negative");
p8rs.test_mem("Negative - Memory", 0x5f00, 0x80)

cls()
p8rs.test("Negative Size - Return", clip(60, 70, -10, -5))
rectfill(30, 34, 95, 97, 8)
p8rs.test_scr("Negative Size");
p8rs.test_mem("Negative Size - Memory", 0x5f00, 0x80)

cls()
p8rs.test("Frac - Return", clip(5.2, 10.8, 64.2, 64.8))
rectfill(30, 34, 95, 97, 8)
p8rs.test_scr("Frac");
p8rs.test_mem("Frac - Memory", 0x5f00, 0x80)

cls()
p8rs.test("Frac 2 - Return", clip(5.5, 10.5, 64.5, 64.5))
rectfill(30, 34, 95, 97, 8)
p8rs.test_scr("Frac 2");
p8rs.test_mem("Frac 2 - Memory", 0x5f00, 0x80)

cls()
p8rs.test("Outside - Return", clip(-64, -64, 32, 32))
rectfill(30, 34, 95, 97, 8)
p8rs.test_scr("Outside");
p8rs.test_mem("Outside - Memory", 0x5f00, 0x80)

cls()
p8rs.test("Only XYW - Return", clip(5, 10, 64))
rectfill(30, 34, 95, 97, 8)
p8rs.test_scr("Only XYW");
p8rs.test_mem("Only XYW - Memory", 0x5f00, 0x80)

cls()
p8rs.test("Only XY - Return", clip(5, 10))
rectfill(30, 34, 95, 97, 8)
p8rs.test_scr("Only XY");
p8rs.test_mem("Only XY - Memory", 0x5f00, 0x80)

cls()
p8rs.test("Only X - Return", clip(5))
rectfill(30, 34, 95, 97, 8)
p8rs.test_scr("Only X");
p8rs.test_mem("Only X - Memory", 0x5f00, 0x80)

cls()
p8rs.test("No args - Return", clip())
rectfill(30, 34, 95, 97, 8)
p8rs.test_scr("No args");
p8rs.test_mem("No args - Memory", 0x5f00, 0x80)

cls()
clip(0, 0, 64, 64)
rectfill(30, 34, 95, 97, 8)
clip(64, 60, 2, 64)
rectfill(30, 34, 95, 97, 11)
clip(40, 66, 100, 4)
rectfill(30, 34, 95, 97, 12)
p8rs.test_scr("Draw");

cls()
clip(0, 0, 64, 64)
camera(-4, -4)
rectfill(30, 34, 95, 97, 8)
clip(64, 60, 2, 64)
camera(2, -2)
rectfill(30, 34, 95, 97, 11)
clip(40, 66, 100, 4)
camera(-1, -3)
rectfill(30, 34, 95, 97, 12)
p8rs.test_scr("Draw with camera");
