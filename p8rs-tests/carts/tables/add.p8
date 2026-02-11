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

p8rs.test("oob ret", add(tab, "kek", 0))
p8rs.test("oob", tab)
