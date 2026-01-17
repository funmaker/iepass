pico-8 cartridge // http://www.pico-8.com
version 43

__lua__
p8rs.test("global start", "global start")

for i=1,10 do
  p8rs.test("frame", i)
  flip()
end

p8rs.test("global end", "global end")
