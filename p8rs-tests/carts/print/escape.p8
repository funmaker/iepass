pico-8 cartridge // http://www.pico-8.com
version 43

__lua__

print("test1\ntest2")
print("test3\rt4")
print("test5\bt6")
print("test5\tt6\tt7t7\tt8")
p8rs.test_scr("Escape nrbt")

cls()
print("\^wTest\^-wTest\^w", 0, 0)
print("normal")
print("\^=Test\^-=Test")
print("\^=\^t\^wTest\^-=Test")
print("\^=\^wTest\^-=Test")
print("\^pTest\^-pTest")
p8rs.test_scr("Escape ^")