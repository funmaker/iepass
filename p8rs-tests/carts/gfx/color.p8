pico-8 cartridge // http://www.pico-8.com
version 43

__lua__
oldColor = color(8)
p8rs.test_mem("Basic", 0x5f00, 0x80)
p8rs.test("Basic - Return", oldColor)

oldColor = color(0xcb)
p8rs.test_mem("Pat color", 0x5f00, 0x80)
p8rs.test("Pat color - Return", oldColor)

oldColor = color()
p8rs.test_mem("No arg", 0x5f00, 0x80)
p8rs.test("No arg - Return", oldColor)

oldColor = color(-3)
p8rs.test_mem("Negative", 0x5f00, 0x80)
p8rs.test("Negative - Return", oldColor)

oldColor = color(0x1234)
p8rs.test_mem("Overflow", 0x5f00, 0x80)
p8rs.test("Overflow - Return", oldColor)
