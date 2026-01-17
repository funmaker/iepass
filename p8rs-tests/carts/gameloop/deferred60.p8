pico-8 cartridge // http://www.pico-8.com
version 43

__lua__
p8rs.test("global start", "global start")

local init = 0;
function _init()
  init += 1
  p8rs.test("init", init)
end

local update = 0;
function _update()
  update += 1
  p8rs.test("update", update)

  if update > 10 then stop("update done") end
end

local update60 = 0;
function _update60()
  update60 += 1
  p8rs.test("update 60", update60)

  if update60 > 10 then stop("update 60 done") end
end

local draw = 0;
function _draw()
  draw += 1
  p8rs.test("draw", draw)

  if draw > 10 then stop("draw done") end
end

p8rs.test("global end", "global end")
