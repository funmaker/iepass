pico-8 cartridge // http://www.pico-8.com
version 43

__lua__
p8rs.test("table", #{ 1, 2, 3 })
p8rs.test("table empty", #{})
p8rs.test("string", #"Hello World!")
p8rs.test("string empty", #"")

function test(label, a, b, c, d)
  p8rs.test(
    label,
    #{ a, b, c, d },
    #{ [1] = a, [2] = b, [3] = c, [4] = d },
    #pack(a, b, c, d)
  )
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
