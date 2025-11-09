pico-8 cartridge // http://www.pico-8.com
version 43

__lua__
print("Initial")
print("Offset", 10, 20)
print("Continue")
p8rs.test_scr("Basic")

print("Red", 8)
print("Green", 20, 50, 11)
p8rs.test_scr("Colors")

print("Clip left", -20, 60)
print("Clip right", 110, 60)
print("Clip top", 40, -3)
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