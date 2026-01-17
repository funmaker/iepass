pico-8 cartridge // http://www.pico-8.com
version 43

__lua__
tab = {1, 2, true, false, "string", { 10, 20, nested = "table" }, nil, "after nil", 2, [11] = "after gap", [12] = 2, [0] = "zero", [1.5] = "between", named = "named field"}

p8rs.test("normal", count(tab))
p8rs.test("empty", count({}))
p8rs.test("explicit", count({ [1] = "one", [2] = "two", [3] = nil, [4] = "after nil", [6] = "after gap" }))
p8rs.test("string", count("abcdef"))
p8rs.test("nil", count(nil))
p8rs.test("no args", count())
p8rs.test("count 2", count(tab, 2))
p8rs.test("count false", count(tab, false))
p8rs.test("count nil", count(tab, nil))
p8rs.test("count nested", count(tab, tab[6]))
p8rs.test("count inline", count(tab, { 10, 20, nested = "table" }))
p8rs.test("count zero", count(tab, "zero"))
p8rs.test("count between", count(tab, "between"))
p8rs.test("count named", count(tab, "named field"))
p8rs.test("count absent", count(tab, "blank"))

