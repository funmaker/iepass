pico-8 cartridge // http://www.pico-8.com
version 43

__lua__
function check(name, a1, a2, a3, a4, a5, a6, a7, a8, a9)
  p8rs.test(name .. " 1", a1)
  p8rs.test(name .. " 2", a2)
  p8rs.test(name .. " 3", a3)
  p8rs.test(name .. " 4", a4)
  p8rs.test(name .. " 5", a5)
  p8rs.test(name .. " 6", a6)
  p8rs.test(name .. " 7", a7)
  p8rs.test(name .. " 8", a8)
  p8rs.test(name .. " 9", a9)
end

check("normal", unpack({1, 2, false, "string", { 10, 20, nested = "table" }, nil, "after nil", 2, [15] = "after gap", [16] = 2, [0] = "zero", [1.5] = "between", named = "named field"}))
check("single", unpack({"string"}))
check("empty", unpack({}))
check("nil start", unpack({nil, nil, 1, 2, 3}))
check("nil mid", unpack({1, nil, nil, 4, "string"}))
check("nil trailing", unpack({1, 2, 3, 4, "string", nil, nil}))
check("nil all", unpack({nil, 1, 2, 3, nil, 4, 5, "string", nil}))
check("weird 1", unpack({nil, 1, nil, 4, 5, "string", nil, nil}))
check("weird 2", unpack({nil, 1, nil, 4, "string", nil, nil}))
check("range", unpack({1, 2, true, false, "string", { 10, 20, nested = "table" }, nil, "after nil", 2, [15] = "after gap", [16] = 2, [0] = "zero", [1.5] = "between", named = "named field"}, 2, 5))
check("range nils", unpack({nil, nil, 1, 2, 3, nil, nil, 4, "string", nil, nil}, 2, 10))
check("range outside", unpack({nil, 1, nil, 2, nil}, -1, 7))
