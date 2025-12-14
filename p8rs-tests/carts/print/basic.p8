pico-8 cartridge // http://www.pico-8.com
version 43

__lua__
cls(1)
p8rs.test_mem("Draw state - initial", 0x5f00, 0x80)
print("Initial")
p8rs.test_mem("Draw state - basic", 0x5f00, 0x80)
print("Offset", 10, 20)
p8rs.test_mem("Draw state - offsets", 0x5f00, 0x80)
print("Continue")
print("With\n  newline")
p8rs.test_mem("Draw state - after newline", 0x5f00, 0x80)
print("aFTER nEWLINE")
p8rs.test_scr("Basic")

print("Red", 8)
print("Green", 14, 56, 11)
print("No args")
p8rs.test_scr("Colors")

print("Clip left", -20, 60)
print("Clip right", 110, 60)
p8rs.test_mem("Draw state - horizonal clips", 0x5f00, 0x80)
print("Clip top - off-screen", 40, -22)
p8rs.test_mem("Draw state - clipped off top", 0x5f00, 0x80)
print("Clip top", 40, -3)
p8rs.test_mem("Draw state - clipped partial", 0x5f00, 0x80)
print("Clip bottom", 40, 125)
p8rs.test_scr("Clipped")

print("Outside left", -50, 60)
print("Outside right", 130, 60)
print("Outside top", 40, -10)
print("Outside bottom", 40, 130)
p8rs.test_scr("Outside")

print("Overlap", 40, 60, 2)
print("Overlap", 42, 62, 12)
print("Overlap", 44, 64, 9)
p8rs.test_scr("Overlap")

print("かなカナ", 40, 40)
print("★⬇️✽●")
p8rs.test_scr("Non-ASCII")
