pico-8 cartridge // http://www.pico-8.com
version 43

__lua__

cls()

camera(5, 5)

clip(20, 20, 40, 40)
print("test\nTEST\n1234\nqwer", 15, 15)
p8rs.test_scr("Camera test")

camera()

clip(20, 20, 40, 40)
print("test\nTEST\n1234\nqwer", 15, 15)
p8rs.test_scr("Clip test")

camera(5, 5)
clip(20, 20, 40, 40)
print("test\nTEST\n1234\nqwer", 15, 15)
p8rs.test_scr("Clip + camera test")
