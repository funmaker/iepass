pico-8 cartridge // http://www.pico-8.com
version 43

__lua__
p8rs.test("normal", pack(1, 2, "string", false, { nested="nested field" }))
p8rs.test("nil", pack(1, 2, nil, 4, 5, nil, nil, nil))
p8rs.test("nil #", #pack(1, 2, nil, 4, 5, nil, nil, nil))
p8rs.test("empty", pack())
p8rs.test("table", pack({1, 2, 3}))

