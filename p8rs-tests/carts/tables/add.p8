pico-8 cartridge // http://www.pico-8.com
version 43

__lua__
tab = {}
p8rs.test("init", tab)

p8rs.test("num ret", add(tab, 39))
p8rs.test("num", tab)

p8rs.test("string ret", add(tab, "string"))
p8rs.test("string", tab)

p8rs.test("bool ret", add(tab, true))
p8rs.test("bool", tab)

p8rs.test("nil ret", add(tab, nil))
p8rs.test("nil", tab)

p8rs.test("nested ret", add(tab, { 1, 2, 3, other = "table" }))
p8rs.test("nested", tab)

p8rs.test("less args ret", add(tab))
p8rs.test("less args", tab)

p8rs.test("inserted ret", add(tab, "inserted", 2))
p8rs.test("inserted", tab)

badtab = 123
p8rs.test("bad type ret", add(badtab, "bad type", 2))
p8rs.test("bad type", badtab)

p8rs.test_err("oob 0 ret", function() add(tab, "oob", 0) end)
p8rs.test("oob 0", tab)

p8rs.test_err("oob -2 ret", function() add(tab, "oob", -2) end)
p8rs.test("oob -2", tab)

p8rs.test_err("oob 50 ret", function() add(tab, "oob", 50) end)
p8rs.test("oob 50", tab)

p8rs.test_err("bad offset 0 ret", function() add(tab, "bad offset", "bad offset") end)
p8rs.test("bad offset 0", tab)


function test(label, a, b, c, d)
  local tab_a = {a, b, c, d}
  add(tab_a, "kek")
  local tab_s = {a, b, c, d}
  add(tab_s, "kek", 1)

  p8rs.test(label, tab_a, tab_s)
end

test("nils ____", nil, nil, nil, nil)
test("nils ___4", nil, nil, nil,   4)
test("nils __3_", nil, nil,   3, nil)
test("nils __34", nil, nil,   3,   4)
test("nils _2__", nil,   2, nil, nil)
test("nils _2_4", nil,   2, nil,   4)
test("nils _23_", nil,   2,   3, nil)
test("nils _234", nil,   2,   3,   4)
test("nils 1___",   1, nil, nil, nil)
test("nils 1__4",   1, nil, nil,   4)
test("nils 1_3_",   1, nil,   3, nil)
test("nils 1_34",   1, nil,   3,   4)
test("nils 12__",   1,   2, nil, nil)
test("nils 12_4",   1,   2, nil,   4)
test("nils 123_",   1,   2,   3, nil)
test("nils 1234",   1,   2,   3,   4)
