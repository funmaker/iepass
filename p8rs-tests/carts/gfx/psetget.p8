pico-8 cartridge // http://www.pico-8.com
version 43

__lua__
for y=0,127 do
  for x=0,127 do
    pset(x, y, (x + y) % 16)
  end
end
local p1 = pget(4, 4)
local p2 = pget(10, 20)
local p3 = pget(64, 64)
local p4 = pget(120, 2)
p8rs.test_scr("Basic");
p8rs.test("Basic - Return", { p1, p2, p3, p4 })

cls()
pset(4, 4)
pset(10, 20, 8)
pset(64, 64)
color(11)
pset(120, 2)
local p1 = pget(4, 4)
local p2 = pget(10, 20)
local p3 = pget(64, 64)
local p4 = pget(120, 2)
p8rs.test_scr("Color");
p8rs.test("Color - Return", { p1, p2, p3, p4 })

cls()
pset(-4, -4, 8)
pset(-10, 200, 11)
pset(500, -200, 20)
pset(50, -5, -10)
local p1 = pget(-4, -4)
local p2 = pget(-10, 200)
local p3 = pget(500, -200)
local p4 = pget(50, -5)
p8rs.test_scr("Overflow");
p8rs.test("Overflow - Return", { p1, p2, p3, p4 })

cls()
pset(4, 4, 8)
pset(10, 20)
pset(64)
pset()
local p1 = pget(4, 4)
local p2 = pget(10, 20)
local p3 = pget(64)
local p4 = pget()
p8rs.test_scr("Less args");
p8rs.test("Less args - Return", { p1, p2, p3, p4 })

cls()
camera(-10, -15)
for y=0,127 do
  for x=0,127 do
    pset(x, y, (x + y) % 16)
  end
end
local p1 = pget(4, 4)
local p2 = pget(10, 20)
local p3 = pget(64, 64)
local p4 = pget(120, 2)
p8rs.test_scr("Camera");
p8rs.test("Camera - Return", { p1, p2, p3, p4 })

cls()
camera(-10, -15)
for y=0,127 do
  for x=0,127 do
    pset(x, y, (x + y) % 16)
  end
end
local p1 = pget(4, 4)
local p2 = pget(10, 20)
local p3 = pget(64, 64)
local p4 = pget(120, 2)
p8rs.test_scr("Clip");
p8rs.test("Clip - Return", { p1, p2, p3, p4 })

cls()
camera(-10, -15)
clip(15, 20, 96, 100)
fillp(✽)
for y=0,31 do
  for x=0,127 do
    pset(x, y, (x + y) % 16)
  end
end
fillp(0b0011001111001100)
for y=32,63 do
  for x=0,127 do
    pset(x, y, ((x + y) % 16) | 0xb0)
  end
end
fillp(♥)
for y=64,95 do
  for x=0,127 do
    pset(x, y, (x + y) % 16)
  end
end
fillp(…)
for y=96,127 do
  for x=0,127 do
    pset(x, y, (x + y) % 16)
  end
end
local p1 = pget(4, 4)
local p2 = pget(10, 20)
local p3 = pget(64, 64)
local p4 = pget(120, 2)
p8rs.test_scr("Fill Pattern");
p8rs.test("Fill Pattern - Return", { p1, p2, p3, p4 })
