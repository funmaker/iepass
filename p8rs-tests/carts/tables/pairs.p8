pico-8 cartridge // http://www.pico-8.com
version 43

__lua__
tab = {1, 2, true, false, "string", { 10, 20, nested = "table" }, nil, "after nil", 2, [15] = "after gap", [16] = 2, [0] = "zero", [1.5] = "between", named = "named field"}
new = {}

for key, val in pairs(tab) do
  new[key] = val
end

p8rs.test("orig", tab)
p8rs.test("copied", new)
