pico-8 cartridge // http://www.pico-8.com
version 43

__lua__
tab = {"one", "two", "three", "four", "five", "six"}
p8rs.test("init", tab)
p8rs.test("init #", #tab)
p8rs.test("init count", count(tab))
p8rs.test("init nils", count(tab, nil))
tab[5] = nil
p8rs.test("nilled", tab)
p8rs.test("nilled #", #tab)
p8rs.test("nilled count", count(tab))
p8rs.test("nilled nils", count(tab, nil))
tab[5] = "new five"
p8rs.test("filled", tab)
p8rs.test("filled #", #tab)
p8rs.test("filled count", count(tab))
p8rs.test("filled nils", count(tab, nil))
tab[8] = "after gap"
p8rs.test("gap", tab)
p8rs.test("gap #", #tab)
p8rs.test("gap count", count(tab))
p8rs.test("gap nils", count(tab, nil))
tab[7] = "bridge"
p8rs.test("bridged", tab)
p8rs.test("bridged #", #tab)
p8rs.test("bridged count", count(tab))
p8rs.test("bridged nils", count(tab, nil))
tab[9] = nil
p8rs.test("nil term", tab)
p8rs.test("nil term #", #tab)
p8rs.test("nil term count", count(tab))
p8rs.test("nil term nils", count(tab, nil))
tab = {"one", "two", "three", "four", "five", nil}
p8rs.test("nil term lit", tab)
p8rs.test("nil term lit #", #tab)
p8rs.test("nil term lit count", count(tab))
p8rs.test("nil term lit nils", count(tab, nil))
