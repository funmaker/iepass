pico-8 cartridge // http://www.pico-8.com
version 43

__lua__
oldX, oldY, oldC = cursor(10, 20)
p8rs.test_mem("Basic - Memory", 0x5f00, 0x80)
print("Text")
p8rs.test_scr("Basic")
p8rs.test("Basic - Return", { oldX, oldY, oldC })

oldX, oldY, oldC = cursor(30, 40, 8)
p8rs.test_mem("Color - Memory", 0x5f00, 0x80)
print("Text")
p8rs.test_scr("Color")
p8rs.test("Color - Return", { oldX, oldY, oldC })

oldX, oldY, oldC = cursor(50)
p8rs.test_mem("Only X - Memory", 0x5f00, 0x80)
print("Text")
p8rs.test_scr("Only X")
p8rs.test("Only X - Return", { oldX, oldY, oldC })

oldX, oldY, oldC = cursor()
p8rs.test_mem("No args - Memory", 0x5f00, 0x80)
print("Text")
p8rs.test_scr("No args")
p8rs.test("No args - Return", { oldX, oldY, oldC })

oldX, oldY, oldC = cursor(-2, -2)
p8rs.test_mem("Negative - Memory", 0x5f00, 0x80)
print("Text")
p8rs.test_scr("Negative")
p8rs.test("Negative - Return", { oldX, oldY, oldC })

oldX, oldY, oldC = cursor(-300, -400)
p8rs.test_mem("Outside - Memory", 0x5f00, 0x80)
print("Text")
p8rs.test_scr("Outside")
p8rs.test("Outside - Return", { oldX, oldY, oldC })

