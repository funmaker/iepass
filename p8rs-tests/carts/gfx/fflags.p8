pico-8 cartridge // http://www.pico-8.com
version 43

__lua__
fset(0, 0b11100101)
fset(1, 0b11001010)
flag1 = fget(0)
flag2 = fget(1)
p8rs.test_mem("Basic - Memory", 0x3000, 0x100)
p8rs.test("Basic", { flag1, flag2 })

fset(2, 0, true)
fset(2, 1, false)
flag1 = fget(2, 0)
flag2 = fget(2, 1)
p8rs.test_mem("Bits - Memory", 0x3000, 0x100)
p8rs.test("Bits", { flag1, flag2 })

fset(3, 8, true)
fset(3, -1, true)
fset(-1, 1, true)
fset(256, 2, true)
flag1 = fget(3, 8)
flag2 = fget(3, -1)
flag3 = fget(-1, 1)
flag4 = fget(256, 2)
p8rs.test_mem("Outside - Memory", 0x3000, 0x100)
p8rs.test("Outside", { flag1, flag2, flag3, flag4 })

fset()
flag1 = fget()
p8rs.test_mem("No args - Memory", 0x3000, 0x100)
p8rs.test("No args", flag1)
