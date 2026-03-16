pico-8 cartridge // http://www.pico-8.com
version 43

__lua__
tab = {1, 2, true, false, "string", { 10, 20, nested = "table" }, nil, "after nil", named = "named field"}

local i = 1
for val in all(tab) do
  p8rs.test("val " .. i, val)
  i += 1
end

i = 1
for val in all("Hello World!") do
  p8rs.test("str " .. i, val)
  i += 1
end

i = 1
for val in all(nil) do
  p8rs.test("nil " .. i, val)
  i += 1
end

p8rs.test_err("true", function() all(true) end)
p8rs.test_err("false", function() all(false) end)
p8rs.test_err("function", function() all(function() end) end)
p8rs.test_err("thread", function() all(cocreate(function() end)) end)
