pico-8 cartridge // http://www.pico-8.com
version 43

__lua__
tab = {}
p8rs.test("init", tab)
ret = add(tab, 39)
p8rs.test("num", tab)
p8rs.test("num ret", ret)
ret = add(tab, "string")
p8rs.test("string", tab)
p8rs.test("string ret", ret)
ret = add(tab, true)
p8rs.test("bool", tab)
p8rs.test("bool ret", ret)
ret = add(tab, nil)
p8rs.test("nil", tab)
p8rs.test("nil ret", ret)
ret = add(tab, { 1, 2, 3, other = "table" })
p8rs.test("nested", tab)
p8rs.test("nested ret", ret)
ret = add(tab)
p8rs.test("less args", tab)
p8rs.test("less args ret", ret)
ret = add(tab, "inserted", 2)
p8rs.test("inserted", tab)
p8rs.test("inserted ret", ret)
badtab = 123
ret = add(badtab, "bad type", 2)
p8rs.test("bad type", badtab)
p8rs.test("bad type ret", ret)
ret = add(tab, "kek", 0)
p8rs.test("oob", tab)
p8rs.test("oob ret", ret)