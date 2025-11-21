pico-8 cartridge // http://www.pico-8.com
version 43

__lua__

local text = "0123456789abcdef"
local longtext = ""
for i = 1, 16 do
   longtext = longtext .. text
end
text = longtext
longtext = ""
for i = 1, 256 do
   longtext = longtext .. text
end
longtext = longtext .. "TestingTestingTestingTesting"
print(#longtext)

local ret = print("asd", 50, 50)


p8rs.test_scr("Test")
p8rs.test("Test", #longtext)


