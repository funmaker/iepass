pico-8 cartridge // http://www.pico-8.com
version 43

__lua__
p8rs.test("Basic - Return", cursor(10, 20))
p8rs.test_mem("Basic - Memory", 0x5f00, 0x80)
print("Text")
p8rs.test_scr("Basic")

p8rs.test("Color - Return", cursor(30, 40, 8))
p8rs.test_mem("Color - Memory", 0x5f00, 0x80)
print("Text")
p8rs.test_scr("Color")

p8rs.test("Only X - Return", cursor(50))
p8rs.test_mem("Only X - Memory", 0x5f00, 0x80)
print("Text")
p8rs.test_scr("Only X")

p8rs.test("No args - Return", cursor())
p8rs.test_mem("No args - Memory", 0x5f00, 0x80)
print("Text")
p8rs.test_scr("No args")

p8rs.test("Negative - Return", cursor(-6, -2))
p8rs.test_mem("Negative - Memory", 0x5f00, 0x80)
print("Text")
p8rs.test_scr("Negative")

p8rs.test("Outside - Return", cursor(250, -254))
p8rs.test_mem("Outside - Memory", 0x5f00, 0x80)
print("Text")
p8rs.test_scr("Outside")

