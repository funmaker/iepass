pico-8 cartridge // http://www.pico-8.com
version 43

__lua__

cls()

print("xd", 15, 15)
p8rs.test_scr("Plain test")
printh("Cursor now: "..(@0x5f26)..", "..(@0x5f27))

cls()

camera(5, 5)

clip(20, 20, 40, 40)
print("test\nTEST\n1234\nqwer", 15, 15)
p8rs.test_scr("Camera test")
printh("Cursor now: "..(@0x5f26)..", "..(@0x5f27))

camera()

clip(20, 20, 40, 40)
print("test\nTEST\n1234\nqwer", 15, 15)
p8rs.test_scr("Clip test")
printh("Cursor now: "..(@0x5f26)..", "..(@0x5f27))

camera(5, 5)
clip(20, 20, 40, 40)
print("test\nTEST\n1234\nqwer", 15, 15)
p8rs.test_scr("Clip + camera test")
printh("Cursor now: "..(@0x5f26)..", "..(@0x5f27))

cls()
fillp(✽ | 0b0.1111)
print("test\nTEST\n1234\nqwer", 15, 15)
print("test\nTEST\n1234\nqwer", 20, 20)
p8rs.test_scr("Fillp test")
printh("Cursor now: "..(@0x5f26)..", "..(@0x5f27))