pico-8 cartridge // http://www.pico-8.com
version 43

__lua__
tab = {1, 2, true, false, "string", { 10, 20, nested = "table" }, nil, "after nil", named = "named field"}

for val in all(tab) do
  p8rs.test("val", val)
end
