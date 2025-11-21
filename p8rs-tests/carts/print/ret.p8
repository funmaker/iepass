pico-8 cartridge // http://www.pico-8.com
version 43

__lua__
p8rs.test("Print return - Simple", print("Simple"))
p8rs.test("Print return - Pos", print("Pos", 20, 20))
p8rs.test("Print return - Col", print("Col", 10, 30, 2))
p8rs.test_scr("Print")