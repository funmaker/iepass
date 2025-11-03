pico-8 cartridge // http://www.pico-8.com
version 43

__lua__
a = 5
p8rs.test("Assign", a)
a = 5
a += 5
p8rs.test("AssignAdd", a)
a = 5
a -= 5
p8rs.test("AssignSub", a)
a = 5
a *= 5
p8rs.test("AssignMul", a)
a = 5
a /= 2
p8rs.test("AssignDiv", a)
a = 5
a \= 2
p8rs.test("AssignIDiv", a)
a = 5
a %= 3
p8rs.test("AssignMod", a)
a = 5
a ^= 5
p8rs.test("AssignPow", a)
a = 5
a &= 3
p8rs.test("AssignBitAnd", a)
a = 5
a |= 3
p8rs.test("AssignBitOr", a)
a = 5
a ^^= 3
p8rs.test("AssignBitXor", a)
a = -5
a >>= 2
p8rs.test("AssignShiftRightArithmetic", a)
a = -5
a >>>= 2
p8rs.test("AssignShiftRightLogical", a)
a = 5
a <<= 2
p8rs.test("AssignShiftLeft", a)
a = 32767
a >><= 20
p8rs.test("AssignRotateRight", a)
a = 32767
a <<>= 4
p8rs.test("AssignRotateLeft", a)
a = "hello"
a ..= " world"
p8rs.test("AssignConcat", a)