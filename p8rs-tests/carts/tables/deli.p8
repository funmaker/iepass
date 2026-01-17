pico-8 cartridge // http://www.pico-8.com
version 43

__lua__
tab = {"one", "two", "three", "four", "five"}
ret = deli(tab, 2)
p8rs.test("deli 2", tab)
p8rs.test("deli 2 ret", ret)
p8rs.test("deli 2 #", #tab)
ret = deli(tab, -2)
p8rs.test("deli -2", tab)
p8rs.test("deli -2 ret", ret)
p8rs.test("deli -2 #", #tab)
ret = deli(tab, 0)
p8rs.test("deli 0", tab)
p8rs.test("deli 0 ret", ret)
p8rs.test("deli 0 #", #tab)
ret = deli(tab, 6)
p8rs.test("deli 6", tab)
p8rs.test("deli 6 ret", ret)
p8rs.test("deli 6 #", #tab)
ret = deli(tab)
p8rs.test("less args", tab)
p8rs.test("less args ret", ret)
p8rs.test("less args #", #tab)
ret = deli()
p8rs.test("no args", tab)
p8rs.test("no args ret", ret)
p8rs.test("no args #", #tab)
tab = {"one", "two", "three", nil, "five", [7] = "seven"}
ret = deli(tab)
p8rs.test("after nil", tab)
p8rs.test("after nil ret", ret)
p8rs.test("after nil #", #tab)
ret = deli(tab, 7)
p8rs.test("after gap", tab)
p8rs.test("after gap ret", ret)
p8rs.test("after gap #", #tab)

