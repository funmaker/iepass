pico-8 cartridge // http://www.pico-8.com
version 43
__lua__

p8rs.test("init", time(), t())

local last = time()
for i = 1,127 do
  _set_fps(i)
  flip()
  local now = time()
  p8rs.test(i .. " fps", now, now - last)
  last = now
end
