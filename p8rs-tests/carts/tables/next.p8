pico-8 cartridge // http://www.pico-8.com
version 43

__lua__
tab = {1, 2, true, false, "string", { 10, 20, nested = "table" }, nil, "after nil", 2, [11] = "after gap", [12] = 2, [0] = "zero", [1.5] = "between", named = "named field"}

p8rs.test("normal", next(tab))
p8rs.test("after 1", next(tab, 1))
p8rs.test("after nil", next(tab, 6))
p8rs.test("empty", next({}))
p8rs.test("explicit", next({ [1] = "one", [2] = "two", [3] = nil, [4] = "after nil", [6] = "after gap" }))
p8rs.test("named", next({named = "field"}))
p8rs.test("after named", next({named = "field"}, "named"))
p8rs.test("after nil", next({named = "field"}, nil))
p8rs.test("bool", next({[true] = "bool"}))
