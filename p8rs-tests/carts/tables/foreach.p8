pico-8 cartridge // http://www.pico-8.com
version 43

__lua__
tab = {1, 2, true, false, "string", { 10, 20, nested = "table" }, nil, "after nil", 2, [11] = "after gap", [12] = 2, [0] = "zero", [1.5] = "between", named = "named field"}

i = 1
foreach(tab, function(val)
  p8rs.test("normal " .. i, val)
  i += 1
end)

i = 1
foreach("string\0null", function(val)
  p8rs.test("string " .. i, val)
  i += 1
end)

