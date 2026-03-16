pico-8 cartridge // http://www.pico-8.com
version 43

__lua__
a = 5
a = a + 5
p8rs.test("Add", a)
a = 5
a = a - 5
p8rs.test("Sub", a)
a = 5
a = a * 5
p8rs.test("Mul", a)
a = 5
a = a / 2
p8rs.test("Div", a)
a = 5
a = a \ 2
p8rs.test("IDiv", a)
a = 5
a = a % 3
p8rs.test("Mod", a)
a = 5
a = a ^ 5
p8rs.test("Pow", a)
a = 5
a = a & 3
p8rs.test("BitAnd", a)
a = 5
a = a | 3
p8rs.test("BitOr", a)
a = 5
a = a ^^ 3
p8rs.test("BitXor", a)
a = -5
a = a >> 2
p8rs.test("ShiftRightArithmetic", a)
a = -5
a = a >>> 2
p8rs.test("ShiftRightLogical", a)
a = 5
a = a << 2
p8rs.test("ShiftLeft", a)
a = 32767
a = a >>< 20
p8rs.test("RotateRight", a)
a = 32767
a = a <<> 4
p8rs.test("RotateLeft", a)
a = "hello"
a = a .. " world"
p8rs.test("Concat", a)