pico-8 cartridge // http://www.pico-8.com
version 43

__lua__
p8rs.test("Global Start", "Global Start")

local init = 0;
function _init()
  init += 1
  p8rs.test("Init", init)
end

local update = 0;
function _update()
  update += 1
  p8rs.test("Update", update)

  if update > 10 then stop("Update done") end
end

local draw = 0;
function _draw()
  draw += 1
  p8rs.test("Draw", draw)

  if draw > 10 then stop("Draw done") end
end

p8rs.test("Global End", "Global End")
