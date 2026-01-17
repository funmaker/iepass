pico-8 cartridge // http://www.pico-8.com
version 43

__lua__
p8rs.test("normal", pack(unpack({1, 2, false, "string", { 10, 20, nested = "table" }, nil, "after nil", 2, [15] = "after gap", [16] = 2, [0] = "zero", [1.5] = "between", named = "named field"})))
p8rs.test("empty", pack(unpack({})))
p8rs.test("nil start", pack(unpack({nil, nil, 1, 2, 3})))
p8rs.test("nil mid", pack(unpack({1, nil, nil, 4, "string"})))
p8rs.test("nil trailing", pack(unpack({1, 2, 3, 4, "string", nil, nil})))
p8rs.test("nil all", pack(unpack({nil, 1, 2, 3, nil, 4, 5, "string", nil})))
p8rs.test("weird 1", pack(unpack({nil, 1, nil, 4, 5, "string", nil, nil})))
p8rs.test("weird 2", pack(unpack({nil, 1, nil, 4, "string", nil, nil})))
p8rs.test("range", pack(unpack({1, 2, true, false, "string", { 10, 20, nested = "table" }, nil, "after nil", 2, [15] = "after gap", [16] = 2, [0] = "zero", [1.5] = "between", named = "named field"}, 2, 5)))
p8rs.test("range nils", pack(unpack({nil, nil, 1, 2, 3, nil, nil, 4, "string", nil, nil}, 2, 10)))
p8rs.test("range outside", pack(unpack({nil, 1, nil, 2, nil}, -1, 7)))
