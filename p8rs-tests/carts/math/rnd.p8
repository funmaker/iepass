pico-8 cartridge // http://www.pico-8.com
version 43

__lua__
for _, s in ipairs({0, 1, 2, 3, 4, 5, -1, 0x0.0001, 0x7FFF.FFFF, 0x8000.0000, 0xFFFF.FFFF}) do
  srand(s)
  p8rs.test_mem(s.." => srand("..s..")", 0x5f00, 0x80)
  p8rs.test(s.." => rnd(10)", rnd(10))
  p8rs.test(s.." => rnd(0.5)", rnd(0.5))
  p8rs.test(s.." => rnd(0xFFFF.FFFF)", rnd(0xFFFF.FFFF))
end

p8rs.test("rnd(0)", rnd(0))
p8rs.test("rnd(1..10)", rnd({1, 2, 3, 4, 5, 6, 7, 8, 9, 10}))
p8rs.test("rnd(★웃♥🐱♪)", rnd({'★','웃','♥','🐱','♪'}))
p8rs.test("rnd({})", rnd({}))

for i = 1,10 do
  p8rs.test("rnd() "..i, rnd())
end
