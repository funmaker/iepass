pico-8 cartridge // http://www.pico-8.com
version 43

__lua__
print("start", 0, 6)
for i=1,30 do
	print("line " .. i)
end
p8rs.test_scr("Scrolling")
cls()

print("Start", 10, 9)
for i=1,30 do
	print("line " .. i)
end
p8rs.test_scr("Scrolling with offset")
