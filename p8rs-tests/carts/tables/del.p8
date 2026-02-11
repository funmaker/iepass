pico-8 cartridge // http://www.pico-8.com
version 43

__lua__
tab = {1, 2, 3, 4, 2, 2, 5}

p8rs.test("del 2 ret", del(tab, 2))
p8rs.test("del 2", tab)
p8rs.test("del 2 #", #tab)

p8rs.test("del 2 x2 ret", del(tab, 2))
p8rs.test("del 2 x2", tab)
p8rs.test("del 2 x2 #", #tab)

p8rs.test("del 2 x3 ret", del(tab, 2))
p8rs.test("del 2 x3", tab)
p8rs.test("del 2 x3 #", #tab)

tab = {1, 2, 3, {nested = "table"}, 4, 5, 6}
p8rs.test("del inline ret", del(tab, {nested = "table"}))
p8rs.test("del inline", tab)
p8rs.test("del inline #", #tab)

tab = {1, 2, 3, {nested = "table"}, 4, 5, 6}
p8rs.test("del nested ret", del(tab, tab[4]))
p8rs.test("del nested", tab)
p8rs.test("del nested #", #tab)

tab = {1, 2, 3, "before nil", nil, "after nil", 4, 5, 6}
p8rs.test("del before nil ret", del(tab, "before nil"))
p8rs.test("del before nil", tab)
p8rs.test("del before nil #", #tab)

tab = {1, 2, 3, "before nil", nil, "after nil", 4, 5, 6}
p8rs.test("del nil ret", del(tab, nil))
p8rs.test("del nil", tab)
p8rs.test("del nil #", #tab)

tab = {1, 2, 3, "before nil", nil, "after nil", 4, 5, 6}
p8rs.test("del after nil ret", del(tab, "after nil"))
p8rs.test("del after nil", tab)
p8rs.test("del after nil #", #tab)

tab = {1, 2, 3, "before gap", [6] = "after gap", [7] = 4, [8] = 5, [9] = 6}
p8rs.test("del before gap ret", del(tab, "before gap"))
p8rs.test("del before gap", tab)
p8rs.test("del before gap #", #tab)

tab = {1, 2, 3, "before gap", [6] = "after gap", [7] = 4, [8] = 5, [9] = 6}
p8rs.test("del gap ret", del(tab, nil))
p8rs.test("del gap", tab)
p8rs.test("del gap #", #tab)

tab = {1, 2, 3, "after gap", [6] = "after gap", [7] = 4, [8] = 5, [9] = 6}
p8rs.test("del after gap ret", del(tab, "after gap"))
p8rs.test("del after gap", tab)
p8rs.test("del after gap #", #tab)

tab = {1, 2, 3, [0] = "zero"}
p8rs.test("del zero ret", del(tab, "zero"))
p8rs.test("del zero", tab)
p8rs.test("del zero #", #tab)

tab = {1, 2, 3, [1.5] = "between"}
p8rs.test("del between ret", del(tab, "between"))
p8rs.test("del between", tab)
p8rs.test("del between #", #tab)

tab = {1, 2, 3, named = "named field"}
p8rs.test("del named field ret", del(tab, "named field"))
p8rs.test("del named field", tab)
p8rs.test("del named field #", #tab)

tab = {1, 2, 3, "before nil", nil, "after nil", 4, 5, 6}
p8rs.test("del less args ret", del(tab))
p8rs.test("del less args", tab)
p8rs.test("del less args #", #tab)

p8rs.test("del no args ret", del())
p8rs.test("del no args", tab)
p8rs.test("del no args #", #tab)
