pico-8 cartridge // http://www.pico-8.com
version 43

__lua__
for y=0,127 do
  for x=0,127 do
    sset(x, y, (x + y) % 16)
  end
end
local p1 = sget(4, 4)
local p2 = sget(10, 20)
local p3 = sget(64, 64)
local p4 = sget(120, 2)
p8rs.test_mem("Basic", 0x0000, 0x2000);
p8rs.test("Basic - Return", { p1, p2, p3, p4 })

cls()
sset(4, 4)
sset(10, 20, 8)
sset(64, 64)
color(11)
sset(120, 2)
local p1 = sget(4, 4)
local p2 = sget(10, 20)
local p3 = sget(64, 64)
local p4 = sget(120, 2)
p8rs.test_mem("Color", 0x0000, 0x2000);
p8rs.test("Color - Return", { p1, p2, p3, p4 })

cls()
sset()
sset(11)
sset(10, 20)
sset(4, 4, 8)
local p1 = sget()
local p2 = sget(64)
local p3 = sget(10, 20)
local p4 = sget(4, 4)
p8rs.test_mem("Overflow", 0x0000, 0x2000);
p8rs.test("Overflow - Return", { p1, p2, p3, p4 })

cls()
sset(4, 4, 8)
sset(10, 20)
sset(64)
sset()
local p1 = sget(4, 4)
local p2 = sget(10, 20)
local p3 = sget(64)
local p4 = sget()
p8rs.test_mem("Less args", 0x0000, 0x2000);
p8rs.test("Less args - Return", { p1, p2, p3, p4 })

cls()
pal({ 8, 9, 10, 11, 12, 13, 14, 15, 0, 1, 2, 3, 4, 5, 6, 7 })
for y=0,127 do
  for x=0,127 do
    sset(x, y, (x + y) % 16)
  end
end
local p1 = sget(4, 4)
local p2 = sget(10, 20)
local p3 = sget(64, 64)
local p4 = sget(120, 2)
p8rs.test_scr("Palette");
p8rs.test("Palette - Return", { p1, p2, p3, p4 })

cls()
camera(-10, -15)
for y=0,127 do
  for x=0,127 do
    sset(x, y, (x + y) % 16)
  end
end
local p1 = sget(4, 4)
local p2 = sget(10, 20)
local p3 = sget(64, 64)
local p4 = sget(120, 2)
p8rs.test_mem("Camera", 0x0000, 0x2000);
p8rs.test("Camera - Return", { p1, p2, p3, p4 })

cls()
camera(-10, -15)
for y=0,127 do
  for x=0,127 do
    sset(x, y, (x + y) % 16)
  end
end
local p1 = sget(4, 4)
local p2 = sget(10, 20)
local p3 = sget(64, 64)
local p4 = sget(120, 2)
p8rs.test_mem("Clip", 0x0000, 0x2000);
p8rs.test("Clip - Return", { p1, p2, p3, p4 })

cls()
camera(-10, -15)
clip(15, 20, 96, 100)
fillp(✽)
for y=0,31 do
  for x=0,127 do
    sset(x, y, (x + y) % 16)
  end
end
fillp(0b0011001111001100)
for y=32,63 do
  for x=0,127 do
    sset(x, y, ((x + y) % 16) | 0xb0)
  end
end
fillp(♥)
for y=64,95 do
  for x=0,127 do
    sset(x, y, (x + y) % 16)
  end
end
fillp(…)
for y=96,127 do
  for x=0,127 do
    sset(x, y, (x + y) % 16)
  end
end
local p1 = sget(4, 4)
local p2 = sget(10, 20)
local p3 = sget(64, 64)
local p4 = sget(120, 2)
p8rs.test_mem("Fill Pattern", 0x0000, 0x2000);
p8rs.test("Fill Pattern - Return", { p1, p2, p3, p4 })
